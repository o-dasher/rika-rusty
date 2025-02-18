use itertools::Itertools;
use rosu_pp::any::PerformanceAttributes;
use sqlx::{types::BigDecimal, Pool, Postgres, QueryBuilder};
use std::{collections::HashSet, sync::Arc};

use rosu_v2::model::{score::Score, GameMode};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::utils::id_locked::{IDLocker, IDLockerError};

use super::beatmap_cache;

#[derive(derive_more::From)]
pub enum SubmissionID {
    ByStoredID(i64),
    ByUsername(String),
}

pub struct ScoreSubmitter {
    locker: IDLocker<String>,
    beatmap_cache_manager: Arc<beatmap_cache::Manager>,
    rosu: Arc<rosu_v2::Osu>,
    pool: Pool<Postgres>,
}

pub struct ReadyScoreSubmitter {
    submitter: Arc<ScoreSubmitter>,
    sender: Sender<(usize, usize)>,
}

#[derive(thiserror::Error, Debug, derive_more::From)]
pub enum SubmissionError {
    #[error("This command does not support this mode.")]
    UnsupportedMode,

    #[error("Invalid user id")]
    InvalidUserID,

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    IdLocker(IDLockerError),

    #[error(transparent)]
    RosuV2(rosu_v2::error::OsuError),

    #[error(transparent)]
    FetchBeatmap(beatmap_cache::Error),

    #[error(transparent)]
    Io(std::io::Error),
}

impl ScoreSubmitter {
    #[must_use]
    pub fn new(
        beatmap_cache_manager: Arc<beatmap_cache::Manager>,
        rosu: Arc<rosu_v2::Osu>,
        pool: Pool<Postgres>,
    ) -> Self {
        Self {
            locker: IDLocker::new(),
            beatmap_cache_manager,
            rosu,
            pool,
        }
    }
}

pub trait ScoreSubmitterDispatcher {
    fn begin_submission(&self) -> (ReadyScoreSubmitter, Receiver<(usize, usize)>);
}

impl ScoreSubmitterDispatcher for Arc<ScoreSubmitter> {
    fn begin_submission(&self) -> (ReadyScoreSubmitter, Receiver<(usize, usize)>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        (
            ReadyScoreSubmitter {
                submitter: Self::clone(self),
                sender,
            },
            receiver,
        )
    }
}

pub struct MinimalStoredScore {
    score_id: BigDecimal,
}

type SubmissionPerformanceInformation<'a> = Vec<(PerformanceAttributes, (&'a Score, &'a u64))>;

impl ReadyScoreSubmitter {
    async fn get_submission_user_id(&self, osu_id: SubmissionID) -> Result<u32, SubmissionError> {
        Ok(match osu_id {
            SubmissionID::ByStoredID(id) => u32::try_from(id)
                .ok()
                .ok_or(SubmissionError::InvalidUserID)?,
            SubmissionID::ByUsername(username) => self.submitter.rosu.user(username).await?.user_id,
        })
    }

    async fn get_performance_information<'a>(
        &'a self,
        new_scores: &'a [(u64, &Score)],
    ) -> Result<SubmissionPerformanceInformation<'a>, SubmissionError> {
        let mut performance_information = vec![];

        for (i, (score_id, score)) in new_scores.iter().enumerate() {
            let ss = &score.statistics;
            performance_information.push((
                rosu_pp::Performance::new(
                    rosu_pp::Difficulty::new()
                        .mods(score.mods.bits())
                        .calculate(&rosu_pp::Beatmap::from_bytes(
                            &self
                                .submitter
                                .beatmap_cache_manager
                                .get_beatmap_file(score.map_id)
                                .await?,
                        )?),
                )
                .n300(ss.great)
                .n100(ss.ok)
                .n50(ss.meh)
                .n_geki(ss.perfect)
                .n_katu(ss.good)
                .misses(ss.miss)
                .calculate(),
                (*score, score_id),
            ));
            let _ = self.sender.send((i + 1, new_scores.len())).await;
        }

        Ok(performance_information)
    }

    async fn store_new_performance_data(
        &self,
        performance_information: SubmissionPerformanceInformation<'_>,
        raw_osu_user_id: u32,
        mode: GameMode,
    ) -> Result<(), SubmissionError> {
        // Let's not start an transaction if not required.
        if performance_information.is_empty() {
            return Ok(());
        }

        let mode_bits = i16::from(mode as u8);
        let mut tx = self.submitter.pool.begin().await?;

        QueryBuilder::<Postgres>::new(
            "
			INSERT INTO osu_score (score_id, osu_user_id, map_id, mods, mode)
			",
        )
        .push_values(
            &performance_information,
            |mut b, (.., (score, score_id))| {
                b.push_bind(BigDecimal::from(**score_id))
                    .push_bind(i64::from(raw_osu_user_id))
                    .push_bind(i64::from(score.map_id))
                    .push_bind(i64::from(score.mods.bits()))
                    .push_bind(mode_bits);
            },
        )
        .build()
        .execute(&mut *tx)
        .await?;

        QueryBuilder::<Postgres>::new(format!(
            "INSERT INTO {}_performance (score_id, mode, overall{})",
            match mode {
                GameMode::Osu => "osu",
                GameMode::Taiko => "taiko",
                GameMode::Catch => "catch",
                GameMode::Mania => "mania",
            },
            match mode {
                GameMode::Osu => ", aim, speed, flashlight, accuracy",
                GameMode::Taiko => ", accuracy, difficulty",
                GameMode::Mania => ", difficulty",
                GameMode::Catch => "",
            }
        ))
        .push_values(
            &performance_information,
            |mut b, (performance, (.., score_id))| {
                b.push_bind(BigDecimal::from(**score_id))
                    .push_bind(mode_bits)
                    .push_bind(performance.pp());

                match performance {
                    PerformanceAttributes::Osu(o) => b
                        .push_bind(o.pp_aim)
                        .push_bind(o.pp_speed)
                        .push_bind(o.pp_flashlight)
                        .push_bind(o.pp_acc),
                    PerformanceAttributes::Taiko(t) => {
                        b.push_bind(t.pp_acc).push_bind(t.pp_difficulty)
                    }
                    PerformanceAttributes::Mania(m) => b.push_bind(m.pp_difficulty),
                    PerformanceAttributes::Catch(..) => &mut b,
                };
            },
        )
        .build()
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    fn get_new_submitted_scores(
        stored_scores: Vec<MinimalStoredScore>,
        api_scores: &[Score],
    ) -> Vec<(u64, &Score)> {
        let existing_scores: HashSet<_> = stored_scores.into_iter().map(|s| s.score_id).collect();
        api_scores
            .iter()
            .filter_map(|s| {
                let is_new = !existing_scores.contains(&s.id.into());
                is_new.then_some((s.id, s))
            })
            .collect_vec()
    }

    pub async fn submit_scores(
        self,
        osu_id: impl Into<SubmissionID> + Send,
        mode: GameMode,
    ) -> Result<(), SubmissionError> {
        let Self { submitter, .. } = &self;

        // We are locking any submission calls from this user.
        let raw_osu_user_id = self.get_submission_user_id(osu_id.into()).await?;
        let locker_guard = submitter.locker.lock(raw_osu_user_id.to_string())?;

        let stored_osu_id = i64::from(raw_osu_user_id);
        let osaka_osu_scores = sqlx_conditional_queries::conditional_query_as!(
            MinimalStoredScore,
            "
			SELECT s.score_id FROM osu_score s
			JOIN {#mode}_performance pp ON s.score_id = pp.score_id
			WHERE s.osu_user_id = {stored_osu_id}
			",
            #mode = match mode {
                GameMode::Osu => "osu",
                GameMode::Mania => "mania",
                GameMode::Taiko => "taiko",
                GameMode::Catch => "catch"
            }
        )
        .fetch_all(&submitter.pool)
        .await?;

        let osu_scores = submitter
            .rosu
            .user_scores(raw_osu_user_id)
            .limit(100)
            .mode(mode)
            .await?;

        self.store_new_performance_data(
            self.get_performance_information(&Self::get_new_submitted_scores(
                osaka_osu_scores,
                &osu_scores,
            ))
            .await?,
            raw_osu_user_id,
            mode,
        )
        .await?;

        locker_guard.unlock()?;

        Ok(())
    }
}
