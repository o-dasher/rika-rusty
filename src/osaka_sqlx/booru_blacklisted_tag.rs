use crate::{osaka_sqlx::Fall, OsakaContext};
use sqlx_conditional_queries_layering::{create_conditional_query_as, merge_sql_query_as};

use crate::get_id_kind_query;

use super::booru_setting::SettingKind;

pub struct BooruBlacklistedTag {
    pub blacklisted: String,
    pub booru_setting_id: i64,
}

create_conditional_query_as!(
    blacklist_query,
    #blacklist_query = match Fall::Through { _ =>
    r#"
    SELECT t.* FROM booru_blacklisted_tag t
    JOIN booru_setting s ON s.id=t.booru_setting_id
    "#
});

impl BooruBlacklistedTag {
    fn map_to_string(tags: Result<Vec<Self>, sqlx::Error>) -> Vec<String> {
        tags.map(|v| v.iter().map(|v| v.blacklisted.clone()).collect())
            .unwrap_or_default()
    }

    pub async fn query_autocomplete_for_kind(
        ctx: OsakaContext<'_>,
        kind: SettingKind,
        searching: &str,
    ) -> Vec<String> {
        let inserted_discord_id = kind.get_sqlx_id(ctx).unwrap_or_default();

        get_id_kind_query!(kind);
        merge_sql_query_as!(blacklist, id_kind);

        Self::map_to_string(
            blacklist_with_id_kind_query!(
                BooruBlacklistedTag,
                r#"
                {#blacklist_query}
                AND t.blacklisted ILIKE CONCAT('%', {searching}::TEXT, '%')
                AND s.{#id_kind}_id={inserted_discord_id}
                "#,
            )
            .fetch_all(&ctx.data().pool)
            .await,
        )
    }

    pub async fn fetch_all(ctx: OsakaContext<'_>) -> Vec<String> {
        Self::map_to_string(
            blacklist_query!(BooruBlacklistedTag, r#"{#blacklist_query}"#)
                .fetch_all(&ctx.data().pool)
                .await,
        )
    }

    pub async fn fetch_all_for_kind(ctx: OsakaContext<'_>, kind: SettingKind) -> Vec<String> {
        let inserted_discord_id = kind.get_sqlx_id(ctx).unwrap_or_default();

        get_id_kind_query!(kind);
        merge_sql_query_as!(blacklist, id_kind);

        Self::map_to_string(
            blacklist_with_id_kind_query!(
                BooruBlacklistedTag,
                r#"
                {#blacklist_query}
                AND s.{#id_kind}_id={inserted_discord_id}
                "#,
            )
            .fetch_all(&ctx.data().pool)
            .await,
        )
    }
}
