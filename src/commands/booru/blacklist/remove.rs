use crate::commands::booru::{self, autocomplete_tag, SettingKind};
use crate::error::NotifyError;
use crate::responses::emojis::OsakaMoji;
use crate::responses::markdown::mono;
use crate::responses::templates::cool_text;
use crate::OsakaData;
use crate::{OsakaContext, OsakaResult};
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;
use rusty18n::I18NAccessible;

#[command(slash_command)]
pub async fn remove(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag"] tag: String,
) -> OsakaResult {
    let i18n = ctx.i18n();
    let OsakaData { pool, .. } = ctx.data();

    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_owner_insert_options_some_owner(ctx, kind)?;

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
            t!(i18n.booru.blacklist.remove.failed).access(mono(tag.clone())),
        ))?;
    }

    ctx.say(cool_text(
        OsakaMoji::ZanyFace,
        &t!(i18n.booru.blacklist.remove.removed).access(mono(tag)),
    ))
    .await?;

    Ok(())
}
