pub mod add;
pub mod delete;
pub mod list;

use crate::{
    create_command_group,
    error::{NotifyError, OsakaError},
};

use add::add;
use delete::{clear::clear, remove::remove};
use list::list;

use super::SettingKind;

create_command_group!(blacklist, ["add", "remove", "list", "clear"]);

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
