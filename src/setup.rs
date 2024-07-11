use std::sync::Arc;

use poise::framework;
use rusty18n::I18NWrapper;
use sqlx::{Pool, Postgres};

use crate::{
    error,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
    managers::{self, register_command},
    OsakaConfig, OsakaData,
};

pub async fn setup(
    ctx: &poise::serenity_prelude::Context,
    framework: &framework::Framework<OsakaData, error::Osaka>,
    config: OsakaConfig,
    i18n: I18NWrapper<OsakaLocale, OsakaI18N>,
    pool: Pool<Postgres>,
) -> Result<OsakaData, error::Osaka> {
    let rosu = Arc::new(
        rosu_v2::Osu::builder()
            .client_id(config.osu_client_id)
            .client_secret(&config.osu_client_secret)
            .build()
            .await?,
    );

    let register_command_manager = register_command::Manager();

    Ok(register_command_manager
        .register_commands(
            register_command::Context::Serenity(ctx, &framework.options().commands),
            match config.development_guild {
                None => register_command::Kind::Global,
                Some(..) => register_command::Kind::Development,
            },
            &config,
        )
        .await
        .map(|()| OsakaData {
            i18n,
            rosu: Arc::clone(&rosu),
            config,
            pool: pool.clone(),
            managers: managers::Osaka::new(rosu, pool),
        })?)
}
