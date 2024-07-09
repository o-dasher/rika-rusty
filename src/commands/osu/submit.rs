use crate::{
    managers::osu::submit::{ReadyScoreSubmitterInjection, ScoreSubmitterTrait},
    osaka_sqlx::discord,
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
};
use poise::ChoiceParameter;
use rosu_v2::model::GameMode;
use std::sync::Arc;

use crate::{
    error, managers, osaka_sqlx::booru_setting::SettingKind, OsakaContext, OsakaData, OsakaResult,
};

#[derive(ChoiceParameter, Default, Clone, Copy)]
#[repr(u8)]
pub enum OsuMode {
    #[default]
    #[name = "osu"]
    Osu = 0,

    #[name = "taiko"]
    Taiko = 1,

    #[name = "catch"]
    Catch = 2,

    #[name = "mania"]
    Mania = 3,
}

impl From<OsuMode> for GameMode {
    fn from(value: OsuMode) -> Self {
        match value {
            OsuMode::Osu => Self::Osu,
            OsuMode::Taiko => Self::Taiko,
            OsuMode::Catch => Self::Catch,
            OsuMode::Mania => Self::Mania,
        }
    }
}

#[poise::command(slash_command)]
pub async fn submit(ctx: OsakaContext<'_>, mode: OsuMode) -> OsakaResult {
    let OsakaData {
        pool,
        managers,
        rosu,
        ..
    } = ctx.data();

    let managers::Osaka { osu_manager, .. } = managers;
    let managers::osu::Manager {
        submit_manager,
        beatmap_cache_manager,
        ..
    } = osu_manager;

    let user = sqlx::query_as!(
        discord::User,
        "SELECT * FROM discord_user WHERE id=$1",
        SettingKind::User.get_sqlx_id(ctx)?
    )
    .fetch_one(pool)
    .await
    .map_err(|_| error::Osaka::SimplyUnexpected)?;

    let osu_id = user.osu_user_id.ok_or(error::Osaka::SimplyUnexpected)?;
    let (ready_submitter, mut receiver) =
        submit_manager.begin_submission(ReadyScoreSubmitterInjection::new(
            Arc::clone(beatmap_cache_manager),
            Arc::clone(rosu),
            pool.clone(),
        ));

    let task = tokio::spawn(ready_submitter.submit_scores(osu_id, mode.into()));

    let msg = ctx
        .say(cool_text(OsakaMoji::ZanyFace, "Started score submission!"))
        .await?;

    while let Some((cur, of)) = receiver.recv().await {
        msg.edit(ctx, |b| {
            b.content(cool_text(
                OsakaMoji::ChocolateBar,
                &format!("Processing score {} out of {}", mono(cur), mono(of)),
            ))
        })
        .await?;
    }

    msg.edit(ctx, |b| {
        b.content(cool_text(OsakaMoji::ZanyFace, "Finished score submission"))
    })
    .await?;

    task.await.map_err(|_| error::Osaka::SimplyUnexpected)??;

    Ok(())
}
