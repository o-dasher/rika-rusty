use poise::command;
use poise::serenity_prelude::Permissions;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;
use rusty18n::I18NAccessible;
use sqlx::types::BigDecimal;

use crate::error::NotifyError;
use crate::responses::emojis::OsakaMoji;
use crate::responses::markdown::mono;
use crate::responses::templates::cool_text;
use crate::{
    commands::booru::{BooruContext, SettingKind},
    error::OsakaError,
    OsakaContext, OsakaData, OsakaResult,
};

pub struct ID {
    pub id: BigDecimal,
}

fn as_some_if<T>(value: T, condition: impl FnOnce(&T) -> bool) -> Option<T> {
    if condition(&value) {
        Some(value)
    } else {
        None
    }
}

#[command(slash_command)]
pub async fn blacklist<'a>(ctx: OsakaContext<'a>, kind: SettingKind, tag: String) -> OsakaResult {
    let i18n = ctx.i18n();
    let booru_ctx = BooruContext(ctx);
    let OsakaData { pool, .. } = ctx.data();

    let perms = ctx
        .author_member()
        .await
        .ok_or(OsakaError::SimplyUnexpected)?
        .permissions
        .ok_or(OsakaError::SimplyUnexpected)?;

    let authorized = match kind {
        SettingKind::Guild => perms.administrator(),
        SettingKind::Channel => perms.administrator(),
        SettingKind::User => true,
    };

    if !authorized {
        return Err(NotifyError::MissingPermissions)?;
    };

    let guild_id = booru_ctx.guild();
    let channel_id = booru_ctx.channel();
    let user_id = booru_ctx.user();

    let to_id = BigDecimal::from(match kind {
        SettingKind::Guild => guild_id.clone().ok_or(OsakaError::SimplyUnexpected)?,
        SettingKind::Channel => channel_id.clone(),
        SettingKind::User => user_id.clone(),
    });

    sqlx_conditional_queries::conditional_query_as!(
        ID,
        "
        INSERT INTO discord_{#id_kind} (id{#args}) VALUES ({to_id}{#binds})
        ON CONFLICT (id) DO UPDATE SET id={to_id} RETURNING id
        ",
        #(id_kind, args, binds) = match kind {
            SettingKind::Guild => ("guild", "", ""),
            SettingKind::Channel => ("channel", ", guild_id", ", {guild_id}"),
            SettingKind::User => ("user", "", "")
        },
    )
    .fetch_one(pool)
    .await?;

    let get_insert_option = |v: Option<BigDecimal>| {
        as_some_if(v, |v| v.as_ref().is_some_and(|v| *v == to_id)).flatten()
    };

    let booru_setting_insertion = sqlx::query!(
        "
        INSERT INTO booru_setting AS s (guild_id, channel_id, user_id) VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET id=s.id RETURNING id 
        ",
        get_insert_option(guild_id),
        get_insert_option(channel_id.into()),
        get_insert_option(user_id.into())
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        "
        INSERT INTO booru_blacklisted_tag
        (booru_setting_id, blacklisted) VALUES ($1, $2)
        ",
        booru_setting_insertion.id,
        tag
    )
    .execute(pool)
    .await?;

    ctx.say(cool_text(
        OsakaMoji::ZanyFace,
        &t!(i18n.booru.blacklist.blacklisted).access(mono(tag)),
    ))
    .await?;

    Ok(())
}
