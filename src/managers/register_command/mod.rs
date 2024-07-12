use poise::serenity_prelude::{self, GuildId};
use strum::Display;

use crate::{
    error::{self},
    OsakaConfig, OsakaContext, OsakaData,
};

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

pub struct Manager();

pub enum Context<'a> {
    Serenity(
        &'a poise::serenity_prelude::Context,
        &'a [poise::Command<OsakaData, error::Osaka>],
    ),
    Poise(&'a OsakaContext<'a>),
}

impl Manager {
    pub async fn register_commands<'a>(
        &self,
        ctx: Context<'a>,
        register_kind: Kind,
        config: &OsakaConfig,
    ) -> Result<(), Error> {
        let (http, commands) = match ctx {
            Context::Serenity(ctx, commands) => (ctx, commands),
            Context::Poise(ctx) => (
                ctx.serenity_context(),
                ctx.framework().options().commands.as_slice(),
            ),
        };

        match register_kind {
            Kind::Development => match config.development_guild {
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
