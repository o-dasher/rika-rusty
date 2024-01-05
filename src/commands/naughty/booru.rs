use crate::{error::OsakaError, utils::pagination::Paginator, OsakaContext, OsakaResult};
use poise::{command, ChoiceParameter};
use rusty_booru::{
    generic::client::{BooruOption, GenericClient},
    shared,
};

const PREFETCHED_IMAGES_COUNT: u32 = 10;

#[derive(ChoiceParameter, Default)]
enum SortChoice {
    Id,
    Score,
    #[default]
    Rating,
    User,
    Height,
    Width,
    Source,
    Updated,
    Random,
}

impl From<SortChoice> for shared::Sort {
    fn from(value: SortChoice) -> shared::Sort {
        match value {
            SortChoice::Id => shared::Sort::Id,
            SortChoice::Score => shared::Sort::Score,
            SortChoice::Rating => shared::Sort::Rating,
            SortChoice::User => shared::Sort::User,
            SortChoice::Height => shared::Sort::Height,
            SortChoice::Width => shared::Sort::Width,
            SortChoice::Source => shared::Sort::Source,
            SortChoice::Updated => shared::Sort::Updated,
            SortChoice::Random => shared::Sort::Random,
        }
    }
}

#[derive(ChoiceParameter, Default)]
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

macro_rules! default_arg {
    ($var:tt) => {
        let $var = $var.unwrap_or_default();
    };
}

#[command(slash_command)]
pub async fn search<'a>(
    ctx: OsakaContext<'a>,
    tags: String,
    booru: Option<BooruChoice>,
    sort: Option<SortChoice>,
) -> OsakaResult {
    default_arg!(sort);
    default_arg!(booru);

    let built_tags = tags.split(" ");
    let mut client = GenericClient::query()
        .limit(PREFETCHED_IMAGES_COUNT)
        .sort(sort.into());

    for tag in built_tags {
        client = client.tag(tag);
    }

    let result_query = client.validate()?.get(booru.into()).await?;

    Paginator::new(ctx, result_query.len())
        .paginate(|idx, r| {
            dbg!(idx);

            let queried = result_query.get(idx).ok_or(OsakaError::SimplyUnexpected)?;

            let Some(file_url) = &queried.file_url else {
                return Err(OsakaError::SimplyUnexpected);
            };

            dbg!(file_url);

            Ok(r.embed(|e| {
                e.image(file_url)
                    .description(format!("Tags: {}", queried.tags))
            })
            .components(|b| {
                if let Some(source) = &queried.source {
                    if !source.is_empty() {
                        b.create_action_row(|b| {
                            b.create_button(|b| {
                                b.label("Source")
                                    .url(source)
                                    .style(poise::serenity_prelude::ButtonStyle::Link)
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
