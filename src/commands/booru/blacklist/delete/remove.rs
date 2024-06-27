use crate::{
    error,
    osaka_sqlx::{booru_blacklisted_tag::BooruBlacklistedTag, booru_setting::SettingKind},
};
use std::{str::FromStr, sync::Arc, vec};

use crate::{
    commands::booru::{
        blacklist::delete::{provide_delete_feedback, Operation},
        utils::poise::OsakaBooruTag,
    },
    responses::markdown::mono,
    OsakaContext, OsakaData, OsakaResult,
};
use poise::{command, ApplicationContext};
use poise_i18n::PoiseI18NTrait;
use rusty18n::t_prefix;

pub async fn autocomplete_tag<'a>(
    ctx: ApplicationContext<'a, Arc<OsakaData>, error::Osaka>,
    searching: &str,
) -> Vec<String> {
    let kind = ctx
        .args
        .iter()
        .find(|v| v.name == "kind")
        .map_or_else(
            || Ok(SettingKind::default()),
            |v| {
                SettingKind::from_str(
                    v.value
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                        .as_str(),
                )
            },
        )
        .unwrap_or_else(|_| SettingKind::default());

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
    #[autocomplete = "autocomplete_tag"] tag: OsakaBooruTag,
) -> OsakaResult {
    provide_delete_feedback(ctx, kind, Operation::Remove(tag.0.clone()), |cleared| {
        let i18n = ctx.i18n();
        t_prefix!($i18n.booru.blacklist.remove);

        let tag = tag.0.clone();
        if cleared {
            t!(failed).with(mono(tag))
        } else {
            t!(removed).with(mono(tag))
        }
    })
    .await
}
