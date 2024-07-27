use crate::commands::osu::Context;
use poise_i18n::PoiseI18NTrait;
use rosu_v2::model::GameMode;
use rusty18n::t_prefix;

use crate::{
    commands::osu::Mode,
    managers::osu::submit::ScoreSubmitterDispatcher,
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
};
use std::convert::From;

use crate::{error, managers, OsakaContext, OsakaData, OsakaResult};

#[poise::command(slash_command)]
pub async fn submit(ctx: OsakaContext<'_>, mode: Mode) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($i18n.osu.submit);

    let OsakaData {
        managers:
            managers::Osaka {
                osu_manager: managers::osu::Manager { submit_manager, .. },
                ..
            },
        ..
    } = ctx.data();

    let (ready_submitter, mut receiver) = submit_manager.begin_submission();

    let osu_user_id = ctx.get_linked_user().await?;
    let task = tokio::spawn(ready_submitter.submit_scores(osu_user_id, GameMode::from(mode)));

    let msg = ctx
        .say(cool_text(OsakaMoji::ZanyFace, t!(started).as_str()))
        .await?;

    while let Some((cur, of)) = receiver.recv().await {
        msg.edit(ctx, |b| {
            b.content(cool_text(
                OsakaMoji::ChocolateBar,
                &t!(processing).with((mono(cur), mono(of))),
            ))
        })
        .await?;
    }

    msg.edit(ctx, |b| {
        b.content(cool_text(OsakaMoji::ZanyFace, t!(finished).as_str()))
    })
    .await?;

    task.await.ok().ok_or(error::Osaka::SimplyUnexpected)??;

    Ok(())
}
