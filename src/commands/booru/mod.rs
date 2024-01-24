pub mod blacklist;
pub mod search;

use crate::{create_command_group, error::OsakaError, OsakaData};
use blacklist::blacklist;
use itertools::Itertools;
use poise::{ApplicationContext, ChoiceParameter};
use rusty_booru::generic::client::{BooruOption, GenericClient};
use search::search;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use strum::IntoStaticStr;

use self::blacklist::as_some_if;

create_command_group!(booru, ["search", "blacklist"]);

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

#[derive(ChoiceParameter, Clone, Copy)]
pub enum SettingKind {
    Guild,
    Channel,
    User,
}

pub fn get_owner_insert_option(
    ctx: OsakaContext,
    operation_kind: SettingKind,
) -> Result<BigDecimal, OsakaError> {
    fn acquire(value: impl Into<u64>) -> BigDecimal {
        Into::<u64>::into(value).into()
    }

    match operation_kind {
        SettingKind::Guild => ctx
            .guild_id()
            .map(acquire)
            .clone()
            .ok_or(OsakaError::SimplyUnexpected),
        SettingKind::Channel => Ok(acquire(ctx.channel_id())),
        SettingKind::User => Ok(acquire(ctx.author().id)),
    }
}

pub fn get_all_owner_insert_options(
    ctx: OsakaContext,
) -> Result<[Option<BigDecimal>; 3], OsakaError> {
    Ok(
        [SettingKind::Guild, SettingKind::Channel, SettingKind::User]
            .map(|s| get_owner_insert_option(ctx, s))
            .map(Result::ok),
    )
}

pub fn get_all_owner_insert_options_some_owner(
    ctx: OsakaContext,
    operation_kind: SettingKind,
) -> Result<[Option<BigDecimal>; 3], OsakaError> {
    let owner_id = get_owner_insert_option(ctx, operation_kind)?;

    get_all_owner_insert_options(ctx)
        .map(|v| v.map(|v| as_some_if(v, |v| v.as_ref().is_some_and(|v| *v == owner_id)).flatten()))
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
