use poise::serenity_prelude::{self, GuildId};
use strum::Display;

use crate::{error::OsakaError, OsakaConfig, OsakaData};

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

pub struct RegisterCommandManager(());

impl RegisterCommandManager {
    pub async fn register_commands(
        ctx: &poise::serenity_prelude::Context,
        config: &OsakaConfig,
        commands: &[poise::Command<OsakaData, OsakaError>],
        register_kind: RegisterKind,
    ) -> Result<(), RegisterError> {
        match register_kind {
            RegisterKind::Development => match config.development_guild {
                Some(guild_id) => {
                    poise::builtins::register_in_guild(ctx, commands, GuildId(guild_id)).await?
                }
                None => return Err(RegisterError::NoDevelopmentGuildSet),
            },
            RegisterKind::Local(guild_id) => {
                poise::builtins::register_in_guild(ctx, commands, guild_id).await?
            }
            RegisterKind::Global => poise::builtins::register_globally(ctx, commands).await?,
        }

        Ok(())
    }
}
