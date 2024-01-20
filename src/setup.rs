use poise::framework;
use rusty18n::I18NWrapper;
use sqlx::{Pool, Postgres};

use crate::{
    error::OsakaError,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
    OsakaConfig, OsakaData,
};

pub async fn setup(
    ctx: &poise::serenity_prelude::Context,
    framework: &framework::Framework<OsakaData, OsakaError>,
    config: OsakaConfig,
    i18n: I18NWrapper<OsakaLocale, OsakaI18N>,
    pool: Pool<Postgres>,
) -> Result<OsakaData, OsakaError> {
    let registered_commands = &framework.options().commands;

    let data = OsakaData { i18n, pool };

    match config.development_guild {
        None => {
            poise::builtins::register_globally(ctx, registered_commands).await?;
        }
        Some(dev_guild) => {
            poise::builtins::register_in_guild(ctx, registered_commands, dev_guild.into()).await?;
        }
    };

    Ok(data)
}
