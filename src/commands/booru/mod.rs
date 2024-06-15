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

pub fn get_setting_kind_db_id(
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

type OwnerInsertOptions = Result<[Option<BigDecimal>; 3], OsakaError>;
pub fn get_all_setting_kind_db_ids(ctx: OsakaContext) -> OwnerInsertOptions {
    Ok(
        [SettingKind::Guild, SettingKind::Channel, SettingKind::User]
            .map(|s| get_setting_kind_db_id(ctx, s))
            .map(Result::ok),
    )
}
