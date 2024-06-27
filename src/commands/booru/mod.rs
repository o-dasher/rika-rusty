pub mod blacklist;
pub mod search;
pub mod utils;

use crate::create_command_group;
use blacklist::blacklist;
use poise::{ApplicationContext, ChoiceParameter};
use rusty_booru::generic::client::BooruOption;
use search::search;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

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
            BooruChoice::Danbooru => Self::Danbooru,
            BooruChoice::Gelbooru => Self::Gelbooru,
            BooruChoice::Safebooru => Self::Safebooru,
        }
    }
}
