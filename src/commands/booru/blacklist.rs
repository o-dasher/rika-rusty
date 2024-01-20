use eager::eager_macro_rules;
use poise::{command, serenity_prelude::GuildId, ChoiceParameter};
use sqlx::types::BigDecimal;

use crate::{error::OsakaError, OsakaContext, OsakaData, OsakaResult};

#[derive(ChoiceParameter, Clone, Copy)]
enum SettingKind {
    Guild,
    Channel,
    User,
}

pub struct ID<T> {
    id: T,
}

pub type BigID = ID<BigDecimal>;

#[command(slash_command)]
pub async fn blacklist<'a>(ctx: OsakaContext<'a>, kind: SettingKind, tag: String) -> OsakaResult {
    let OsakaData { pool, .. } = ctx.data();

    let get_guild_id = || {
        ctx.guild_id()
            .ok_or(OsakaError::SimplyUnexpected)
            .map(|GuildId(id)| id)
    };

    let to_id = BigDecimal::from(match kind {
        SettingKind::Guild => get_guild_id()?.clone(),
        SettingKind::Channel => *ctx.channel_id().as_u64(),
        SettingKind::User => *ctx.author().id.as_u64(),
    });

    let guild_id_stored = get_guild_id().map(BigDecimal::from).ok();

    sqlx_conditional_queries::conditional_query_as!(
        BigID,
        "
        INSERT INTO discord_{#id_kind} (id{#args}) VALUES ({to_id}{#binds})
        ON CONFLICT DO NOTHING RETURNING id
        ",
        #(id_kind, args, binds) = match kind {
            SettingKind::Guild => ("guild", "", ""),
            SettingKind::Channel => ("channel", ", guild_id", ", {guild_id_stored}"),
            SettingKind::User => ("user", "", "")
        },
    )
    .fetch_one(pool)
    .await?;

    let booru_setting_insertion = sqlx_conditional_queries::conditional_query_as!(
        ID,
        "
        INSERT INTO booru_setting ({#id_kind}_id) VALUES ({to_id})
        ON CONFLICT DO NOTHING RETURNING id
        ",
        #id_kind = match kind {
            SettingKind::Guild => "guild",
            SettingKind::Channel => "channel",
            SettingKind::User =>"user"
        }
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        "
        INSERT INTO booru_blacklisted_tag
        (booru_setting_id, blacklisted) VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        ",
        booru_setting_insertion.id,
        tag
    )
    .execute(pool)
    .await?;

    Ok(())
}
