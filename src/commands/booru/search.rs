use crate::{
    default_args, error::OsakaError, responses::markdown::bold, utils::pagination::Paginator,
    OsakaContext, OsakaResult,
};
use itertools::Itertools;
use poise::{command, serenity_prelude::ButtonStyle, ChoiceParameter};
use rusty_booru::generic::client::{BooruOption, GenericClient};

const CLAMP_TAGS_LEN: usize = 75;

#[derive(ChoiceParameter, Default, Clone)]
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

#[command(slash_command)]
pub async fn search<'a>(
    ctx: OsakaContext<'a>,
    tags: String,
    booru: Option<BooruChoice>,
) -> OsakaResult {
    default_args!(booru);

    let built_tags = tags.split(" ");
    let mut client = GenericClient::query();

    for tag in built_tags {
        client.tag(tag);
    }

    let result_query = client.get(booru.clone().into()).await?;

    let mapped_result = result_query
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

            Ok(r.embed(|e| {
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
