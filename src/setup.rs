use std::sync::Arc;

use poise::framework;
use rusty18n::I18NWrapper;
use sqlx::{Pool, Postgres};

use crate::{
    error::OsakaError,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
    managers::{
        register_command::{self},
        OsakaManagers,
    },
    OsakaConfig, OsakaData,
};

pub async fn setup(
    ctx: &poise::serenity_prelude::Context,
    framework: &framework::Framework<Arc<OsakaData>, OsakaError>,
    config: OsakaConfig,
    i18n: I18NWrapper<OsakaLocale, OsakaI18N>,
    pool: Pool<Postgres>,
) -> Result<Arc<OsakaData>, OsakaError> {
    let rosu = Arc::new(
        rosu_v2::Osu::builder()
            .client_id(config.osu_client_id)
            .client_secret(&config.osu_client_secret)
            .build()
            .await?,
    );

    let config = Arc::new(config);
    let register_command_manager = register_command::Manager {
        config: config.clone(),
    };

    Ok(register_command_manager
        .register_commands(
            register_command::Context::Serenity(ctx, &framework.options().commands),
            match config.development_guild {
                None => register_command::Kind::Global,
                Some(..) => register_command::Kind::Development,
            },
        )
        .await
        .map(|()| {
            let managers = Arc::new(OsakaManagers::new(
                config.clone(),
                pool.clone(),
                rosu.clone(),
            ));

            Arc::new(OsakaData {
                i18n,
                rosu,
                config,
                pool,
                managers,
            })
        })?)
}
