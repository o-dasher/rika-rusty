use crate::{
    commands::booru::{
        utils::{autocompletes::autocomplete_tag, poise::OsakaBooruTag},
        BooruChoice,
    },
    osaka_sqlx::booru_blacklisted_tag::BooruBlacklistedTag,
};
use std::vec;

use crate::{
    default_args,
    error::{NotifyError, OsakaError},
    responses::{
        markdown::{bold, mono},
        templates::something_wrong,
    },
    utils::pagination::Paginator,
    OsakaContext, OsakaResult,
};
use itertools::Itertools;
use poise::{command, serenity_prelude::ButtonStyle};
use rusty_booru::generic::{client::GenericClient, Rating};

const CLAMP_TAGS_LEN: usize = 75;

#[command(slash_command)]
pub async fn search(
    ctx: OsakaContext<'_>,
    #[autocomplete = "autocomplete_tag"] tag: OsakaBooruTag,
    booru: Option<BooruChoice>,
    ephemeral: Option<bool>,
) -> OsakaResult {
    ctx.defer().await?;
    default_args!(booru, ephemeral);

    let mut query = GenericClient::query();

    let blacklisted_tags = BooruBlacklistedTag::fetch_all(ctx).await;
    let built_tags = tag.0.split(' ').map(str::to_string).collect_vec();

    if let Some(blacklisted_tag) = built_tags.iter().find(|v| blacklisted_tags.contains(v)) {
        Err(NotifyError::Warn(format!(
            "The tag {} is being blacklisted by either yourself, the channel or this server.",
            mono(blacklisted_tag)
        )))?;
    }

    let queried_tags = built_tags.clone();

    for tag in queried_tags {
        query.tag(tag);
    }

    let reply_not_found = || {
        ctx.send(|f| {
            f.content(something_wrong("Nothing here but us chickens... maybe something sketchy happened to this booru instance!"))
                .attachment("https://media1.tenor.com/m/mb-bdtZ7toYAAAAd/chicken.gif".into()).ephemeral(ephemeral)
        })
    };

    if !ctx
        .guild_channel()
        .await
        .map(|c| c.nsfw)
        // We want to allow nsfw search on dm.
        .unwrap_or_default()
    {
        query.rating(Rating::Safe);
    }

    dbg!("Someone is searching: {}", &query.tags);

    let query_res = query.get(booru.clone().into()).await;

    match &query_res {
        Ok(value) => {
            if value.is_empty() {
                reply_not_found().await?;
                return Ok(());
            }
        }
        Err(e) => {
            dbg!("Something bad happened, booru: {}", e);
            reply_not_found().await?;
            return Ok(());
        }
    }

    let query_res = query_res?;

    let mapped_result = query_res
        .iter()
        .filter(|v| {
            !v.tags
                .split(' ')
                .any(|v| blacklisted_tags.contains(&v.to_string()))
        })
        .filter_map(|v| v.file_url.as_ref().map(|file_url| (file_url, v)))
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
                    e.image(file_url)
                        .description(
                            [
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
                                booru,
                                idx + 1,
                                paginator.amount_pages
                            ))
                        })
                })
                .components(|b| {
                    if let Some(source) = &queried.source
                        && !source.is_empty()
                    {
                        b.create_action_row(|b| {
                            b.create_button(|b| {
                                b.label("Source").url(source).style(ButtonStyle::Link)
                            })
                        });
                    };
                    b
                })
                .to_owned())
        })
        .await?;

    Ok(())
}
