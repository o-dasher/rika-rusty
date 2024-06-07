use std::vec;

use crate::{
    commands::booru::{self, blacklist::delete::provide_delete_feedback, SettingKind},
    OsakaContext, OsakaData, OsakaResult,
};
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t_prefix;

#[command(slash_command)]
pub async fn clear(ctx: OsakaContext<'_>, kind: SettingKind) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($i18n.booru.blacklist.clear);

    let OsakaData { pool, .. } = ctx.data();

    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_setting_kind_db_ids_only_allowing_this_kind(ctx, kind)?;

    let result = sqlx::query!(
        "
        DELETE FROM booru_blacklisted_tag t
        USING booru_setting s WHERE s.id=t.booru_setting_id
        AND s.guild_id=$1 OR s.channel_id=$2 OR s.user_id=$3
        ",
        inserted_guild,
        inserted_channel,
        inserted_user
    )
    .execute(pool)
    .await?;

    provide_delete_feedback(ctx, result, |cleared| {
        if cleared {
            t!(cleared).to_string()
        } else {
            t!(failed).to_string()
        }
    })
    .await?;

    Ok(())
}
