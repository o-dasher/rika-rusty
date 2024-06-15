pub mod add;
pub mod delete;
pub mod list;
pub mod utils;

use crate::{
    create_command_group,
    error::{NotifyError, OsakaError},
    get_conditional_id_kind_query,
    utils::sqlx::Jib,
};

use add::add;
use delete::{clear::clear, remove::remove};
use list::list;
use sqlx::types::BigDecimal;
use sqlx_conditional_queries_layering::create_conditional_query_as;

use super::SettingKind;

create_command_group!(blacklist, ["add", "remove", "list", "clear"]);

pub struct BooruBlacklistedTag {
    pub blacklisted: String,
    pub booru_setting_id: BigDecimal,
}

#[macro_export]
macro_rules! get_blacklist_query {
    () => {
        create_conditional_query_as!(
            blacklist_query,
            #blacklist_query = match Jib::Jab { Jab =>
            "
            SELECT t.* FROM booru_blacklisted_tag t
            JOIN booru_setting s ON s.id = t.booru_setting_id
            WHERE s.id=t.booru_setting_id
            "
        });
    };
}

#[macro_export]
macro_rules! get_blacklist_for_kind_query {
    () => {
        blacklist_query_feed_existing_query!(conditional_id_kind_query, blacklist_for_kind_query);
    };
}

pub async fn query_blacklisted_tags(
    ctx: OsakaContext<'_>,
    kind: Option<SettingKind>,
) -> Vec<String> {
    let pool = ctx.data().pool.clone();

    get_blacklist_query!();
    let tags = match kind {
        Some(kind) => {
            let inserted_discord_id = kind.get_sqlx_id(ctx).unwrap_or_default();

            get_conditional_id_kind_query!(kind);
            get_blacklist_for_kind_query!();

            blacklist_for_kind_query!(
                BooruBlacklistedTag,
                "
                {#blacklist_query}
                AND s.{#id_kind}_id={inserted_discord_id}
                ",
            )
            .fetch_all(&pool)
            .await
        }
        None => {
            blacklist_query!(BooruBlacklistedTag, "{#blacklist_query}")
                .fetch_all(&pool)
                .await
        }
    };

    tags.map(|v| v.iter().map(|v| v.blacklisted.clone()).collect())
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
