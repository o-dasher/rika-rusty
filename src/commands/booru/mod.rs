pub mod blacklist;
pub mod search;
pub mod utils;

use crate::{create_command_group, error::OsakaError};
use blacklist::blacklist;
use poise::{ApplicationContext, ChoiceParameter};
use rusty_booru::generic::client::BooruOption;
use search::search;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use strum::{EnumIter, IntoStaticStr};

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

#[derive(ChoiceParameter, Clone, Copy, EnumIter, Default)]
pub enum SettingKind {
    #[default]
    Guild,
    Channel,
    User,
}

impl SettingKind {
    pub fn get_sqlx_id(self, ctx: OsakaContext) -> Result<BigDecimal, OsakaError> {
        fn acquire(value: impl Into<u64>) -> BigDecimal {
            Into::<u64>::into(value).into()
        }

        match self {
            SettingKind::Guild => ctx
                .guild_id()
                .map(acquire)
                .clone()
                .ok_or(OsakaError::SimplyUnexpected),
            SettingKind::Channel => Ok(acquire(ctx.channel_id())),
            SettingKind::User => Ok(acquire(ctx.author().id)),
        }
    }

    pub fn get_all_sqlx_ids(ctx: OsakaContext) -> Result<[Option<BigDecimal>; 3], OsakaError> {
        Ok(
            [SettingKind::Guild, SettingKind::Channel, SettingKind::User]
                .map(|s| s.get_sqlx_id(ctx))
                .map(Result::ok),
        )
    }
}
