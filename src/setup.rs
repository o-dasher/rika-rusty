use poise::framework;
use rusty18n::I18NWrapper;
use sqlx::{Pool, Postgres};

use crate::{
    error::OsakaError,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
    managers::register_command_manager::{RegisterCommandManager, RegisterKind},
    OsakaConfig, OsakaData,
};

pub async fn setup(
    ctx: &poise::serenity_prelude::Context,
    framework: &framework::Framework<OsakaData, OsakaError>,
    config: OsakaConfig,
    i18n: I18NWrapper<OsakaLocale, OsakaI18N>,
    pool: Pool<Postgres>,
) -> Result<OsakaData, OsakaError> {
    Ok(RegisterCommandManager::register_commands(
        ctx,
        &config,
        &framework.options().commands,
        match config.development_guild {
            None => RegisterKind::Global,
            Some(..) => RegisterKind::Development,
        },
    )
    .await
    .map(|_| OsakaData { i18n, pool, config })?)
}
