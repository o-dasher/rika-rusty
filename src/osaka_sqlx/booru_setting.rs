use poise::ChoiceParameter;
use sqlx::types::BigDecimal;
use strum::EnumIter;

use crate::{error::OsakaError, OsakaContext};

#[macro_export]
macro_rules! get_conditional_id_kind_query {
    ($kind:ident) => {
        sqlx_conditional_queries_layering::create_conditional_query_as!(
            conditional_id_kind_query,
            #id_kind = match $kind {
                SettingKind::Guild => "guild",
                SettingKind::Channel => "channel",
                SettingKind::User => "user"
            }
        );
    };
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
