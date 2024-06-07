pub mod add;
pub mod clear;
pub mod list;
pub mod remove;

use crate::{
    create_command_group,
    error::{NotifyError, OsakaError},
};
use add::add;
use clear::clear;
use list::list;
use remove::remove;
use sqlx::types::BigDecimal;

use super::{get_all_setting_kind_db_ids_only_allowing_this_kind, SettingKind};

create_command_group!(blacklist, ["add", "remove", "list", "clear"]);

pub struct ID<T> {
    pub id: T,
}

pub type BigID = ID<BigDecimal>;

pub fn as_some_if<T>(value: T, condition: impl FnOnce(&T) -> bool) -> Option<T> {
    if condition(&value) {
        Some(value)
    } else {
        None
    }
}

pub async fn query_blacklisted_tags(ctx: OsakaContext<'_>, kind: SettingKind) -> Vec<String> {
    let Ok([inserted_guild, inserted_channel, inserted_user]) =
        get_all_setting_kind_db_ids_only_allowing_this_kind(ctx, kind)
    else {
        return Default::default();
    };

    sqlx::query!(
        "
        SELECT * FROM booru_blacklisted_tag t
        JOIN booru_setting s ON s.id = t.booru_setting_id
        WHERE s.id=t.booru_setting_id
        AND s.guild_id=$1 OR s.channel_id=$2 OR s.user_id=$3
        ",
        inserted_guild,
        inserted_channel,
        inserted_user
    )
    .fetch_all(&ctx.data().pool)
    .await
    .map(|v| v.iter().map(|v| v.blacklisted.clone()).collect())
    .unwrap_or_default()
}

pub async fn check_permissions(ctx: OsakaContext<'_>, operation_kind: SettingKind) -> OsakaResult {
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
        Err(NotifyError::MissingPermissions)?;
    };

    Ok(())
}
