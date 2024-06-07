use std::vec;

use crate::{
    commands::booru::{self, SettingKind},
    error::NotifyError,
    responses::{emojis::OsakaMoji, templates::cool_text},
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

    // Possible conditional query?
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

    if result.rows_affected() < 1 {
        Err(NotifyError::Warn(t!(failed).to_string()))?;
    }

    ctx.say(cool_text(OsakaMoji::ZanyFace, &t!(cleared).to_string()))
        .await?;

    Ok(())
}
