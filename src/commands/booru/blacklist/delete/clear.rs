use std::vec;

use crate::{
    commands::booru::{
        blacklist::delete::{provide_delete_feedback, DeleteOperation},
        SettingKind,
    },
    OsakaContext, OsakaResult,
};
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t_prefix;

#[command(slash_command)]
pub async fn clear(ctx: OsakaContext<'_>, kind: SettingKind) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($i18n.booru.blacklist.clear);

    provide_delete_feedback(ctx, kind, DeleteOperation::Clear, |cleared| {
        if cleared {
            t!(cleared).to_string()
        } else {
            t!(failed).to_string()
        }
    })
    .await?;

    Ok(())
}
