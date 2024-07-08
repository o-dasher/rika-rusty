use itertools::Itertools;
use rosu_pp::any::PerformanceAttributes;
use sqlx::{types::BigDecimal, Pool, Postgres, QueryBuilder};
use std::{collections::HashSet, sync::Arc};

use rosu_v2::model::{score::Score, GameMode};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};

use crate::utils::id_locked::{IDLocker, IDLockerError};

use super::beatmap_cache;

#[derive(derive_more::From)]
pub enum SubmissionID {
    ByStoredID(i64),
    ByUsername(String),
}

pub struct ScoreSubmitter {
    beatmap_cache: Arc<beatmap_cache::Manager>,
    pool: Pool<Postgres>,
    rosu: Arc<rosu_v2::Osu>,
    locker: IDLocker<String>,
}

pub struct ReadyScoreSubmitter {
    submitter: Arc<RwLock<ScoreSubmitter>>,
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
        beatmap_cache: Arc<beatmap_cache::Manager>,
        pool: Pool<Postgres>,
        rosu: Arc<rosu_v2::Osu>,
    ) -> Self {
        Self {
            locker: IDLocker::new(),
            beatmap_cache,
            pool,
            rosu,
        }
    }
}

pub trait ScoreSubmitterTrait {
    fn begin_submission(&self) -> (ReadyScoreSubmitter, Receiver<(usize, usize)>);
}

impl ScoreSubmitterTrait for Arc<RwLock<ScoreSubmitter>> {
    fn begin_submission(&self) -> (ReadyScoreSubmitter, Receiver<(usize, usize)>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        (
            ReadyScoreSubmitter {
                submitter: self.clone(),
                sender,
            },
            receiver,
        )
    }
}

impl ReadyScoreSubmitter {
    pub async fn submit_scores(
        self,
        osu_id: impl Into<SubmissionID> + Send,
        mode: GameMode,
    ) -> Result<(), SubmissionError> {
        pub struct MinimalStoredScore {
            score_id: BigDecimal,
        }

        let submitter = self.submitter.read().await;

        // This cast should be safe since all u8 values fit on i16
        let mode_bits = mode as i16;

        let osu_id = match osu_id.into() {
            SubmissionID::ByStoredID(id) => {
                u32::try_from(id).map_err(|_| SubmissionError::InvalidUserID)?
            }
            SubmissionID::ByUsername(username) => submitter.rosu.user(username).await?.user_id,
        };

        let locker_guard = submitter.locker.lock(osu_id.to_string()).await?;
        let osu_scores = submitter
            .rosu
            .user_scores(osu_id)
            .limit(100)
            .mode(mode)
            .await?;

        let stored_osu_id = i64::from(osu_id);

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

        let existing_scores: HashSet<_> =
            osaka_osu_scores.into_iter().map(|s| s.score_id).collect();

        let new_scores = osu_scores
            .iter()
            .filter_map(|s| {
                let is_new = !existing_scores.contains(&s.id.into());
                is_new.then_some((s.id, s))
            })
            .collect_vec();

        if new_scores.is_empty() {
            return Ok(());
        }

        let mut performance_information: Vec<(PerformanceAttributes, (&Score, &u64))> = vec![];

        for (i, (score_id, score)) in new_scores.iter().enumerate() {
            let ss = &score.statistics;
            performance_information.push((
                rosu_pp::Performance::new(
                    rosu_pp::Difficulty::new()
                        .mods(score.mods.bits())
                        .calculate(&rosu_pp::Beatmap::from_bytes(
                            &submitter
                                .beatmap_cache
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

        tracing::info!("Beginning unsafe transaction!");
        let mut tx = submitter.pool.begin().await?;

        tracing::info!("Storing scores...");
        QueryBuilder::<Postgres>::new(
            "
			INSERT INTO osu_score (score_id, osu_user_id, map_id, mods, mode)
			",
        )
        .push_values(
            &performance_information,
            |mut b, (.., (score, score_id))| {
                b.push_bind(BigDecimal::from(**score_id))
                    .push_bind(i64::from(osu_id))
                    .push_bind(i64::from(score.map_id))
                    .push_bind(i64::from(score.mods.bits()))
                    .push_bind(mode_bits);
            },
        )
        .build()
        .execute(&mut *tx)
        .await?;

        tracing::info!("Storing performance...");
        QueryBuilder::<Postgres>::new(format!(
            "INSERT INTO {mode}_performance (score_id, mode, overall{})",
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

        locker_guard.unlock().await?;
        drop(submitter);

        Ok(())
    }
}
