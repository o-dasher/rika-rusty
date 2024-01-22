use poise::{command, serenity_prelude::GuildId};
use sqlx::types::BigDecimal;

use crate::{
    commands::booru::SettingKind, error::OsakaError, OsakaContext, OsakaData, OsakaResult,
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
    let OsakaData { pool, .. } = ctx.data();

    let get_guild_id = || {
        ctx.guild_id()
            .ok_or(OsakaError::SimplyUnexpected)
            .map(|GuildId(id)| id)
    };

    let guild_id = get_guild_id()?.clone();
    let channel_id = *ctx.channel_id().as_u64();
    let user_id = *ctx.author().id.as_u64();

    let to_id = BigDecimal::from(match kind {
        SettingKind::Guild => guild_id,
        SettingKind::Channel => channel_id,
        SettingKind::User => user_id,
    });

    let guild_id_stored = get_guild_id().map(BigDecimal::from).ok();
    sqlx_conditional_queries::conditional_query_as!(
        ID,
        "
        INSERT INTO discord_{#id_kind} (id{#args}) VALUES ({to_id}{#binds})
        ON CONFLICT (id) DO UPDATE SET id={to_id} RETURNING id
        ",
        #(id_kind, args, binds) = match kind {
            SettingKind::Guild => ("guild", "", ""),
            SettingKind::Channel => ("channel", ", guild_id", ", {guild_id_stored}"),
            SettingKind::User => ("user", "", "")
        },
    )
    .fetch_one(pool)
    .await?;

    let get_insert_option = |v: u64| as_some_if(BigDecimal::from(v), |v| *v == to_id);
    let booru_setting_insertion = sqlx::query!(
        "
        INSERT INTO booru_setting AS s (guild_id, channel_id, user_id) VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET id=s.id RETURNING id 
        ",
        get_insert_option(guild_id),
        get_insert_option(channel_id),
        get_insert_option(user_id)
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

    ctx.say("BLACKLISTED").await?;

    Ok(())
}
