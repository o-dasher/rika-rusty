pub mod add;
pub mod list;
pub mod remove;

use crate::{
    create_command_group,
    error::{NotifyError, OsakaError},
};
use add::add;
use list::list;
use remove::remove;
use sqlx::types::BigDecimal;

use super::SettingKind;

create_command_group!(blacklist, ["add", "remove", "list"]);

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
