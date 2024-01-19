use std::vec;

use crate::{
    default_args,
    error::OsakaError,
    responses::{markdown::bold, templates::something_wrong},
    utils::pagination::Paginator,
    OsakaContext, OsakaData, OsakaResult,
};
use itertools::Itertools;
use poise::{command, serenity_prelude::ButtonStyle, ApplicationContext, ChoiceParameter};
use rusty_booru::generic::client::{BooruOption, GenericClient};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

const CLAMP_TAGS_LEN: usize = 75;

#[derive(IntoStaticStr, ChoiceParameter, Debug, Serialize, Deserialize, Default, Clone)]
enum BooruChoice {
    #[default]
    Danbooru,
    Gelbooru,
    Safebooru,
}

impl From<BooruChoice> for BooruOption {
    fn from(value: BooruChoice) -> Self {
        match value {
            BooruChoice::Danbooru => BooruOption::Danbooru,
            BooruChoice::Gelbooru => BooruOption::Gelbooru,
            BooruChoice::Safebooru => BooruOption::Safebooru,
        }
    }
}

pub async fn autocomplete<'a>(
    ctx: ApplicationContext<'a, OsakaData, OsakaError>,
    searching: &str,
) -> Vec<String> {
    if searching.is_empty() {
        return vec![];
    }

    let booru_choice = ctx
        .args
        .get(1)
        .map(|v| {
            serde_json::from_value::<BooruChoice>(v.value.clone().unwrap_or_default())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let search_vec = searching.split_whitespace().collect_vec();

    let mut search_iter = search_vec.iter();
    let prefix_search = search_iter.by_ref().take(search_vec.len() - 1).join(" ");

    match search_iter.next() {
        Some(last_term) => GenericClient::query()
            .get_autocomplete(booru_choice.into(), *last_term)
            .await
            .unwrap_or_default()
            .iter()
            .map(|v| [prefix_search.clone(), v.value.clone()].join(" "))
            .collect_vec(),
        None => vec![prefix_search],
    }
}

#[command(slash_command)]
pub async fn search<'a>(
    ctx: OsakaContext<'a>,
    booru: Option<BooruChoice>,
    #[autocomplete = "autocomplete"] tags: String,
    ephemeral: Option<bool>,
) -> OsakaResult {
    default_args!(booru, ephemeral);

    let built_tags = tags.split(" ");
    let mut query = GenericClient::query();

    for tag in built_tags {
        query.tag(tag);
    }

    let reply_not_found = || {
        ctx.send(|f| {
            f.content(something_wrong("Nothing here but us chickens... maybe something sketchy happened to this booru instance!"))
                .attachment("https://media1.tenor.com/m/mb-bdtZ7toYAAAAd/chicken.gif".into()).ephemeral(ephemeral)
        })
    };

    let query_res = query.get(booru.clone().into()).await;

    match &query_res {
        Ok(value) => {
            if value.is_empty() {
                reply_not_found().await?;
                return Ok(());
            }
        }
        Err(..) => {
            reply_not_found().await?;
            return Ok(());
        }
    }

    let query_res = query_res?;

    let mapped_result = query_res
        .iter()
        .filter_map(|v| {
            if let Some(file_url) = &v.file_url {
                Some((file_url, v))
            } else {
                None
            }
        })
        .collect_vec();

    let paginator = Paginator::new(ctx, mapped_result.len());
    paginator
        .paginate(|idx, r| {
            dbg!(idx);

            let indexed_res = mapped_result.get(idx).ok_or(OsakaError::SimplyUnexpected)?;
            let (file_url, queried) = indexed_res;

            let tag_description = if queried.tags.len() < CLAMP_TAGS_LEN {
                queried.tags.clone()
            } else {
                format!(
                    "{}...",
                    queried
                        .tags
                        .chars()
                        .take(CLAMP_TAGS_LEN)
                        .collect::<String>()
                )
            };

            dbg!(&queried.file_url);

            Ok(r.ephemeral(ephemeral)
                .embed(|e| {
                    e.image(&file_url)
                        .description(
                            vec![
                                ("Score", queried.score.to_string()),
                                ("Rating", queried.rating.to_string()),
                                ("Tags", tag_description),
                            ]
                            .iter()
                            .map(|(label, value)| format!("{}: {value}", bold(label)))
                            .join(" | "),
                        )
                        .footer(|b| {
                            b.text(format!(
                                "{} - {}/{}",
                                booru.to_string(),
                                idx + 1,
                                paginator.amount_pages
                            ))
                        })
                })
                .components(|b| {
                    if let Some(source) = &queried.source {
                        if !source.is_empty() {
                            b.create_action_row(|b| {
                                b.create_button(|b| {
                                    b.label("Source").url(source).style(ButtonStyle::Link)
                                })
                            });
                        }
                    };
                    b
                })
                .to_owned())
        })
        .await?;

    Ok(())
}
