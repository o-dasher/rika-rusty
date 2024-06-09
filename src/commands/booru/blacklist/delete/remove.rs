use std::{str::FromStr, vec};

use crate::{
    commands::booru::{
        blacklist::{
            BooruBlacklistedTag,
            delete::{provide_delete_feedback, DeleteOperation},
            query_blacklisted_tags,
        },
        get_setting_kind_db_id, SettingKind,
    },
    error::OsakaError,
    get_conditional_id_kind_query,
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
        return query_blacklisted_tags(ctx, kind).await;
    }

    let inserted_discord_id = get_setting_kind_db_id(ctx, kind).unwrap_or_default();

    get_conditional_id_kind_query!(kind);
    let Ok(completions) = conditional_id_kind_query!(
        BooruBlacklistedTag,
        "
        SELECT t.blacklisted FROM booru_blacklisted_tag t
        JOIN booru_setting s ON s.id=t.booru_setting_id
        WHERE s.id=t.booru_setting_id
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
    #[autocomplete = "autocomplete_tag_remove"] tag: String,
) -> OsakaResult {
    let tag = tag.trim().to_lowercase();
    let i18n = ctx.i18n();

    t_prefix!($i18n.booru.blacklist.remove);

    provide_delete_feedback(ctx, kind, DeleteOperation::Remove(tag.clone()), |cleared| {
        let tag = tag.clone();
        if cleared {
            t!(failed).with(mono(tag))
        } else {
            t!(removed).with(mono(tag))
        }
    })
    .await?;

    Ok(())
}
