use poise::ChoiceParameter;
use strum::EnumIter;

use crate::{error, OsakaContext};

#[macro_export]
macro_rules! get_id_kind_query {
    ($kind:ident) => {
        sqlx_conditional_queries_layering::create_conditional_query_as!(
            $id_kind_query,
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
    pub fn get_sqlx_id(self, ctx: OsakaContext) -> Result<i64, error::Osaka> {
        match self {
            Self::Guild => ctx
                .guild_id()
                .map(Into::into)
                .ok_or(error::Osaka::SimplyUnexpected),
            Self::Channel => Ok(ctx.channel_id().into()),
            Self::User => Ok(ctx.author().id.into()),
        }
    }

    pub fn get_all_sqlx_ids(ctx: OsakaContext) -> Result<[Option<i64>; 3], error::Osaka> {
        Ok([Self::Guild, Self::Channel, Self::User]
            .map(|s| s.get_sqlx_id(ctx))
            .map(Result::ok))
    }
}
