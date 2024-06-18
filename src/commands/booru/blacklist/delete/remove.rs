use crate::osaka_sqlx::{booru_blacklisted_tag::BooruBlacklistedTag, booru_setting::SettingKind};
use std::{str::FromStr, vec};

use crate::{
    commands::booru::{
        blacklist::delete::{provide_delete_feedback, DeleteOperation},
        utils::poise::OsakaBooruTag,
    },
    error::OsakaError,
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

    BooruBlacklistedTag::query_autocomplete_for_kind(ctx, kind, searching).await
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
