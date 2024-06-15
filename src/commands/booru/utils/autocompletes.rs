use std::collections::HashSet;

use itertools::Itertools;
use rusty_booru::generic::client::GenericClient;

use crate::{
    commands::booru::{ApplicationContext, BooruChoice},
    error::OsakaError,
    osaka_sqlx::booru_blacklisted_tag::BooruBlacklistedTag,
    OsakaData,
};

pub async fn autocomplete_tag_single<'a>(
    ctx: ApplicationContext<'a, OsakaData, OsakaError>,
    searching: &str,
) -> Vec<String> {
    autocomplete_tag(
        ctx,
        searching.split(" ").collect_vec().first().unwrap_or(&""),
    )
    .await
}

pub async fn autocomplete_tag<'a>(
    ctx: ApplicationContext<'a, OsakaData, OsakaError>,
    searching: &str,
) -> Vec<String> {
    if searching.is_empty() {
        return vec![];
    }

    let booru_choice = ctx
        .args
        .iter()
        .find(|v| v.name == "tag")
        .map(|v| {
            serde_json::from_value::<BooruChoice>(v.value.clone().unwrap_or_default())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let search_vec = searching.split_whitespace().collect_vec();

    let mut search_iter = search_vec.iter();
    let prefix_search = search_iter.by_ref().take(search_vec.len() - 1).join(" ");

    match search_iter.next() {
        Some(last_term) => {
            let blacklisted_tags = HashSet::<String>::from_iter(
                BooruBlacklistedTag::fetch_all(poise::Context::Application(ctx)).await,
            );

            GenericClient::query()
                .get_autocomplete(booru_choice.into(), *last_term)
                .await
                .unwrap_or_default()
                .iter()
                .filter(|v| !blacklisted_tags.contains(&v.value))
                .map(|v| [prefix_search.clone(), v.value.clone()].join(" "))
                .collect_vec()
        }
        None => vec![prefix_search],
    }
}
