pub mod add;
pub mod delete;
pub mod list;

use crate::{
    commands::booru::get_setting_kind_db_id,
    create_command_group,
    error::{NotifyError, OsakaError},
    get_conditional_id_kind_query,
};

use add::add;
use delete::{clear::clear, remove::remove};
use list::list;
use sqlx::types::BigDecimal;

use super::SettingKind;

create_command_group!(blacklist, ["add", "remove", "list", "clear"]);

pub struct ID<T> {
    pub id: T,
}

pub type BigID = ID<BigDecimal>;

pub struct BooruBlacklistedTag {
    blacklisted: String,
}

pub async fn query_blacklisted_tags(ctx: OsakaContext<'_>, kind: SettingKind) -> Vec<String> {
    let inserted_discord_id = get_setting_kind_db_id(ctx, kind).unwrap_or_default();

    get_conditional_id_kind_query!(kind);
    conditional_id_kind_query!(
        BooruBlacklistedTag,
        "
        SELECT t.blacklisted FROM booru_blacklisted_tag t
        JOIN booru_setting s ON s.id = t.booru_setting_id
        WHERE s.id=t.booru_setting_id
        AND s.{#id_kind}_id={inserted_discord_id}
        "
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
