use crate::{
    commands::booru::{self, autocomplete_tag, SettingKind},
    error::NotifyError,
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;

#[command(slash_command)]
pub async fn remove(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag"] tag: String,
) -> OsakaResult {
    let tag = tag.trim().to_lowercase();

    let i18n = ctx.i18n();
    let OsakaData { pool, .. } = ctx.data();

    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_setting_kind_db_ids_only_allowing_this_kind(ctx, kind)?;

    let result = sqlx::query!(
        "
        DELETE FROM booru_blacklisted_tag t
        USING booru_setting s
        WHERE t.blacklisted=$1 AND s.id=t.booru_setting_id
        AND s.guild_id=$2 OR s.channel_id=$3 OR s.user_id=$4
        ",
        tag,
        inserted_guild,
        inserted_channel,
        inserted_user
    )
    .execute(pool)
    .await?;

    if result.rows_affected() < 1 {
        Err(NotifyError::Warn(
            t!(i18n.booru.blacklist.remove.failed).with(mono(tag.clone())),
        ))?;
    }

    ctx.say(cool_text(
        OsakaMoji::ZanyFace,
        &t!(i18n.booru.blacklist.remove.removed).with(mono(tag)),
    ))
    .await?;

    Ok(())
}
