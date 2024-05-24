use std::{str::FromStr, vec};

use crate::{
    commands::booru::{self, SettingKind},
    error::{NotifyError, OsakaError},
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};
use poise::{command, ApplicationContext};
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;

pub async fn autocomplete_tag_remove<'a>(
    ctx: ApplicationContext<'a, OsakaData, OsakaError>,
    searching: &str,
) -> Vec<String> {
    if searching.is_empty() {
        return Default::default();
    }

    let booru_choice = ctx
        .args
        .iter()
        .find(|v| v.name == "kind")
        .map(|v| {
            <SettingKind as FromStr>::from_str(
                v.value
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
                    .as_str(),
            )
        })
        .unwrap_or(Ok(Default::default()))
        .unwrap_or(Default::default());

    let Ok([inserted_guild, inserted_channel, inserted_user]) =
        booru::get_all_setting_kind_db_ids_only_allowing_this_kind(
            poise::Context::Application(ctx),
            booru_choice,
        )
    else {
        return Default::default();
    };

    let Ok(completions) = sqlx::query!(
        "
        SELECT * FROM booru_blacklisted_tag t
        JOIN booru_setting s ON s.id = t.booru_setting_id
        WHERE s.id=t.booru_setting_id
        AND t.blacklisted ILIKE CONCAT('%', $1::TEXT, '%')
        AND s.guild_id=$2 OR s.channel_id=$3 OR s.user_id=$4
        ",
        searching,
        inserted_guild,
        inserted_channel,
        inserted_user
    )
    .fetch_all(&ctx.data().pool)
    .await
    else {
        return Default::default();
    };

    completions.iter().map(|v| v.blacklisted.clone()).collect()
}

#[command(slash_command)]
pub async fn remove(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag_remove"] tag: String,
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
