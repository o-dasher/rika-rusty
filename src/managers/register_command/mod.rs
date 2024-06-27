use std::sync::Arc;

use poise::serenity_prelude::{self, GuildId};
use strum::Display;

use crate::{error::OsakaError, OsakaConfig, OsakaContext, OsakaData};

pub enum Kind {
    Development,
    Local(GuildId),
    Global,
}

#[derive(Debug, Display, thiserror::Error, derive_more::From)]
pub enum Error {
    Serenity(serenity_prelude::Error),
    NoDevelopmentGuildSet,
}

pub struct Manager {
    pub config: Arc<OsakaConfig>,
}

impl Manager {
    pub fn new(config: Arc<OsakaConfig>) -> Self {
        Self { config }
    }
}

pub enum Context<'a> {
    Serenity(
        &'a poise::serenity_prelude::Context,
        &'a [poise::Command<Arc<OsakaData>, OsakaError>],
    ),
    Poise(&'a OsakaContext<'a>),
}

impl Manager {
    pub async fn register_commands<'a>(
        &self,
        ctx: Context<'a>,
        register_kind: Kind,
    ) -> Result<(), Error> {
        let (http, commands) = match ctx {
            Context::Serenity(ctx, commands) => (ctx, commands),
            Context::Poise(ctx) => (
                ctx.serenity_context(),
                ctx.framework().options().commands.as_slice(),
            ),
        };

        match register_kind {
            Kind::Development => match self.config.development_guild {
                Some(guild_id) => {
                    poise::builtins::register_in_guild(http, commands, GuildId(guild_id)).await?;
                }
                None => return Err(Error::NoDevelopmentGuildSet),
            },
            Kind::Local(guild_id) => {
                poise::builtins::register_in_guild(http, commands, guild_id).await?;
            }
            Kind::Global => poise::builtins::register_globally(http, commands).await?,
        }

        Ok(())
    }
}
