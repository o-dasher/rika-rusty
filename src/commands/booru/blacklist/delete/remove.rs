use crate::osaka_sqlx::{
    booru_blacklisted_tag::BooruBlacklistedTag, booru_setting::SettingKind, Jib,
};
use sqlx_conditional_queries_layering::create_conditional_query_as;
use std::{str::FromStr, vec};

use crate::{
    commands::booru::{
        blacklist::delete::{provide_delete_feedback, DeleteOperation},
        utils::poise::OsakaBooruTag,
    },
    error::OsakaError,
    get_blacklist_for_kind_query, get_blacklist_query, get_conditional_id_kind_query,
    responses::markdown::mono,
    OsakaContext, OsakaData, OsakaResult,
};
use poise::{command, ApplicationContext};
use poise_i18n::PoiseI18NTrait;
use rusty18n::t_prefix;

pub async fn autocomplete_tag_remove<'a>(
    ctx: ApplicationContext<'a, OsakaData, OsakaError>,
    searching: &str,
) -> Vec<String> {
    let kind = ctx
        .args
        .iter()
        .find(|v| v.name == "kind")
        .map(|v| {
            SettingKind::from_str(
                v.value
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
                    .as_str(),
            )
        })
        .unwrap_or(Ok(Default::default()))
        .unwrap_or(Default::default());

    let ctx = poise::Context::Application(ctx);
    if searching.is_empty() {
        return BooruBlacklistedTag::fetch_all_for_kind(ctx, kind).await;
    }

    let inserted_discord_id = kind.get_sqlx_id(ctx).unwrap_or_default();

    get_blacklist_query!();
    get_conditional_id_kind_query!(kind);
    get_blacklist_for_kind_query!();

    let Ok(completions) = blacklist_for_kind_query!(
        BooruBlacklistedTag,
        "
        {#blacklist_query}
        AND t.blacklisted ILIKE CONCAT('%', {searching}::TEXT, '%')
        AND s.{#id_kind}_id={inserted_discord_id}
        ",
    )
    .fetch_all(&ctx.data().pool)
    .await
    else {
        return Default::default();
    };

    completions.iter().map(|v| v.blacklisted.clone()).collect()
}

#[command(slash_command)]
pub async fn remove(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag_remove"] tag: OsakaBooruTag,
) -> OsakaResult {
    provide_delete_feedback(
        ctx,
        kind,
        DeleteOperation::Remove(tag.0.clone()),
        |cleared| {
            let i18n = ctx.i18n();
            t_prefix!($i18n.booru.blacklist.remove);

            let tag = tag.0.clone();
            if cleared {
                t!(failed).with(mono(tag))
            } else {
                t!(removed).with(mono(tag))
            }
        },
    )
    .await
}
