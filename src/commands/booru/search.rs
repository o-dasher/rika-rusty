use crate::commands::booru::autocomplete_tag;
use crate::commands::booru::BooruChoice;
use std::{collections::HashSet, vec};

use crate::{
    commands::booru::BooruContext,
    default_args,
    error::{NotifyError, OsakaError},
    responses::{
        markdown::{bold, mono},
        templates::something_wrong,
    },
    utils::pagination::Paginator,
    OsakaContext, OsakaData, OsakaResult,
};
use itertools::Itertools;
use poise::{command, serenity_prelude::ButtonStyle};
use rusty_booru::generic::client::GenericClient;

const CLAMP_TAGS_LEN: usize = 75;

#[command(slash_command)]
pub async fn search<'a>(
    ctx: OsakaContext<'a>,
    booru: Option<BooruChoice>,
    #[autocomplete = "autocomplete_tag"] tags: String,
    ephemeral: Option<bool>,
) -> OsakaResult {
    ctx.defer().await?;

    let booru_ctx = BooruContext(ctx);
    default_args!(booru, ephemeral);

    let OsakaData { pool, .. } = ctx.data();
    let mut query = GenericClient::query();

    let all_blacklists = sqlx::query!(
        "
        SELECT blacklisted FROM booru_blacklisted_tag t
        JOIN booru_setting s ON t.booru_setting_id = s.id
        WHERE s.guild_id = $1 OR s.user_id = $2 OR s.channel_id = $3
        ",
        booru_ctx.guild(),
        booru_ctx.user(),
        booru_ctx.channel()
    )
    .fetch_all(pool)
    .await?;

    let built_tags = tags.split(" ").map(str::to_string).collect_vec();
    let blacklisted_tags = all_blacklists.iter().map(|v| format!("-{}", v.blacklisted));

    let blacklist_set: HashSet<_> = blacklisted_tags.clone().collect::<HashSet<_>>();
    if let Some(blacklisted_tag) = built_tags.iter().find(|v| blacklist_set.contains(*v)) {
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
        .filter(|v| !v.tags.split(" ").any(|v| blacklist_set.contains(v)))
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
