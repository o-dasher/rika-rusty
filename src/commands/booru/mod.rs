pub mod blacklist;
pub mod search;

use crate::{create_command_group, error::OsakaError, OsakaData};
use blacklist::blacklist;
use itertools::Itertools;
use poise::{ApplicationContext, ChoiceParameter};
use rusty_booru::generic::client::{GenericClient, BooruOption};
use search::search;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use strum::IntoStaticStr;

create_command_group!(booru, ["search", "blacklist"]);

#[derive(ChoiceParameter, Clone, Copy)]
pub enum SettingKind {
    Guild,
    Channel,
    User,
}

pub struct BooruContext<'a>(OsakaContext<'a>);

impl<'a> BooruContext<'a> {
    fn acquire(value: impl Into<u64>) -> BigDecimal {
        Into::<u64>::into(value).into()
    }

    pub fn guild(&self) -> Option<BigDecimal> {
        self.0.guild_id().map(Self::acquire)
    }

    pub fn channel(&self) -> BigDecimal {
        Self::acquire(self.0.channel_id())
    }

    pub fn user(&self) -> BigDecimal {
        Self::acquire(self.0.author().id)
    }
}

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
