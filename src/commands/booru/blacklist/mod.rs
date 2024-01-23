pub mod add;

use crate::{
    create_command_group,
    error::{NotifyError, OsakaError},
    OsakaData,
};
use add::add;
use sqlx::{types::BigDecimal, Postgres, Transaction};

use super::{BooruContext, SettingKind};

create_command_group!(blacklist, ["add"]);

pub struct ID {
    pub id: BigDecimal,
}

pub fn as_some_if<T>(value: T, condition: impl FnOnce(&T) -> bool) -> Option<T> {
    if condition(&value) {
        Some(value)
    } else {
        None
    }
}

pub async fn try_begin_blacklist_storing<'a>(
    ctx: OsakaContext<'a>,
    operation_kind: SettingKind,
) -> Result<(i32, Transaction<'_, Postgres>), OsakaError> {
    let booru_ctx = BooruContext(ctx);
    let OsakaData { pool, .. } = ctx.data();

    let perms = ctx
        .author_member()
        .await
        .ok_or(OsakaError::SimplyUnexpected)?
        .permissions
        .ok_or(OsakaError::SimplyUnexpected)?;

    let authorized = match operation_kind {
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

    let to_id = BigDecimal::from(match operation_kind {
        SettingKind::Guild => guild_id.clone().ok_or(OsakaError::SimplyUnexpected)?,
        SettingKind::Channel => channel_id.clone(),
        SettingKind::User => user_id.clone(),
    });

    let mut tx = pool.begin().await?;

    sqlx_conditional_queries::conditional_query_as!(
        ID,
        "
        INSERT INTO discord_{#id_kind} (id{#args}) VALUES ({to_id}{#binds})
        ON CONFLICT (id) DO UPDATE SET id={to_id} RETURNING id
        ",
        #(id_kind, args, binds) = match operation_kind {
            SettingKind::Guild => ("guild", "", ""),
            SettingKind::Channel => ("channel", ", guild_id", ", {guild_id}"),
            SettingKind::User => ("user", "", "")
        },
    )
    .fetch_one(&mut *tx)
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
    .fetch_one(&mut *tx)
    .await?;

    Ok((booru_setting_insertion.id, tx))
}
