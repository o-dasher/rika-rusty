pub mod add;
pub mod remove;

use crate::{
    commands::booru::blacklist::remove::remove,
    create_command_group,
    error::{NotifyError, OsakaError},
};
use add::add;
use sqlx::types::BigDecimal;

use super::SettingKind;

create_command_group!(blacklist, ["add", "remove"]);

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
