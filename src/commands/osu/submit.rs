use rosu_v2::model::GameMode;

use crate::{
    commands::osu::{self, Mode},
    managers::osu::submit::ScoreSubmitterDispatcher,
    osaka_sqlx::discord,
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
};
use std::convert::From;

use crate::{
    error, managers, osaka_sqlx::booru_setting::SettingKind, OsakaContext, OsakaData, OsakaResult,
};

impl From<Mode> for GameMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Osu => Self::Osu,
            Mode::Taiko => Self::Taiko,
            Mode::Catch => Self::Catch,
            Mode::Mania => Self::Mania,
        }
    }
}

#[poise::command(slash_command)]
pub async fn submit(ctx: OsakaContext<'_>, mode: Mode) -> OsakaResult {
    let OsakaData {
        pool,
        managers:
            managers::Osaka {
                osu_manager: managers::osu::Manager { submit_manager, .. },
                ..
            },
        ..
    } = ctx.data();

    let osu_user_id = sqlx::query_as!(
        discord::User,
        "SELECT * FROM discord_user WHERE id=$1",
        SettingKind::User.get_sqlx_id(ctx)?
    )
    .fetch_one(pool)
    .await
    .ok()
    .ok_or(osu::Error::NotLinked)?
    .osu_user_id
    .ok_or(osu::Error::NotLinked)?;

    let (ready_submitter, mut receiver) = submit_manager.begin_submission();

    let task = tokio::spawn(ready_submitter.submit_scores(osu_user_id, GameMode::from(mode)));

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

    task.await.ok().ok_or(error::Osaka::SimplyUnexpected)??;

    Ok(())
}
