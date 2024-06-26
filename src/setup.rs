use std::sync::Arc;

use poise::framework;
use rusty18n::I18NWrapper;
use sqlx::{Pool, Postgres};

use crate::{
    error::OsakaError,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
    managers::{
        osu::OsuManager,
        register::{RegisterCommandManager, RegisterContext, RegisterKind},
    },
    OsakaConfig, OsakaData, OsakaManagers,
};

pub async fn setup(
    ctx: &poise::serenity_prelude::Context,
    framework: &framework::Framework<OsakaData, OsakaError>,
    config: OsakaConfig,
    i18n: I18NWrapper<OsakaLocale, OsakaI18N>,
    pool: Pool<Postgres>,
) -> Result<OsakaData, OsakaError> {
    let rosu = rosu_v2::Osu::builder()
        .client_id(config.osu_client_id)
        .client_secret(&config.osu_client_secret)
        .build()
        .await?;

    let config = Arc::new(config);
    let register_command_manager = RegisterCommandManager {
        config: config.clone(),
    };

    Ok(register_command_manager
        .register_commands(
            RegisterContext::Serenity(ctx, &framework.options().commands),
            match config.development_guild {
                None => RegisterKind::Global,
                Some(..) => RegisterKind::Development,
            },
        )
        .await
        .map(|_| OsakaData {
            i18n,
            pool,
            config,
            rosu,
            managers: OsakaManagers {
                register_command_manager,
                osu_manager: OsuManager::new(),
            },
        })?)
}
