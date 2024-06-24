use std::sync::Arc;

use poise::serenity_prelude::{self, GuildId};
use strum::Display;

use crate::{error::OsakaError, OsakaConfig, OsakaContext, OsakaData};

pub enum RegisterKind {
    Development,
    Local(GuildId),
    Global,
}

#[derive(Debug, Display, thiserror::Error, derive_more::From)]
pub enum RegisterError {
    Serenity(serenity_prelude::Error),
    NoDevelopmentGuildSet,
}

pub struct RegisterCommandManager {
    pub config: Arc<OsakaConfig>,
}

pub enum RegisterContext<'a> {
    Serenity(
        &'a poise::serenity_prelude::Context,
        &'a [poise::Command<OsakaData, OsakaError>],
    ),
    Poise(&'a OsakaContext<'a>),
}

impl RegisterCommandManager {
    pub async fn register_commands<'a>(
        &self,
        ctx: RegisterContext<'a>,
        register_kind: RegisterKind,
    ) -> Result<(), RegisterError> {
        let (http, commands) = match ctx {
            RegisterContext::Serenity(ctx, commands) => (ctx, commands),
            RegisterContext::Poise(ctx) => (
                ctx.serenity_context(),
                ctx.framework().options().commands.as_slice(),
            ),
        };

        match register_kind {
            RegisterKind::Development => match self.config.development_guild {
                Some(guild_id) => {
                    poise::builtins::register_in_guild(http, commands, GuildId(guild_id)).await?
                }
                None => return Err(RegisterError::NoDevelopmentGuildSet),
            },
            RegisterKind::Local(guild_id) => {
                poise::builtins::register_in_guild(http, commands, guild_id).await?
            }
            RegisterKind::Global => poise::builtins::register_globally(http, commands).await?,
        }

        Ok(())
    }
}
