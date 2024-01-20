use poise::{
    command,
    serenity_prelude::{guild, GuildId},
    ChoiceParameter,
};

use crate::{error::OsakaError, OsakaContext, OsakaData, OsakaResult};

#[derive(ChoiceParameter)]
enum SettingKind {
    Guild,
    Channel,
    User,
}

#[command(slash_command)]
pub async fn blacklist<'a>(ctx: OsakaContext<'a>, kind: SettingKind, tag: String) -> OsakaResult {
    let OsakaData { pool, .. } = ctx.data();

    let get_guild_id = || {
        ctx.guild_id()
            .ok_or(OsakaError::SimplyUnexpected)
            .map(|GuildId(id)| id)
    };

    let to_id = match kind {
        SettingKind::Guild => get_guild_id()?,
        SettingKind::Channel => *ctx.channel_id().as_u64(),
        SettingKind::User => *ctx.author().id.as_u64(),
    };

    let queries = match kind {
        SettingKind::Guild => (
            sqlx::query!("INSERT IGNORE INTO discord_guild (id) VALUES (?)", to_id),
            sqlx::query!(
                "INSERT IGNORE INTO booru_setting (guild_id) VALUES (?)",
                to_id
            ),
        ),
        SettingKind::Channel => (
            sqlx::query!(
                "INSERT IGNORE INTO discord_channel (id, guild_id) VALUES (?, ?)",
                to_id,
                get_guild_id()?,
            ),
            sqlx::query!(
                "INSERT IGNORE INTO booru_setting (channel_id) VALUES (?)",
                to_id
            ),
        ),
        SettingKind::User => (
            sqlx::query!("INSERT IGNORE INTO discord_user (id) VALUES (?)", to_id),
            sqlx::query!(
                "INSERT IGNORE INTO booru_setting (user_id) VALUES (?)",
                to_id
            ),
        ),
    };

    let (parent_insertion, booru_setting_insertion) = queries;

    parent_insertion.execute(pool).await?;
    let booru_setting_id = booru_setting_insertion
        .execute(pool)
        .await?
        .last_insert_id();

    sqlx::query!(
        "
        INSERT INTO booru_blacklisted_tag
        (booru_setting_id, blacklisted) VALUES (?, ?)
        ",
        booru_setting_id,
        tag
    )
    .execute(pool)
    .await?;

    Ok(())
}
