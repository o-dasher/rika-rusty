#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::clone_on_ref_ptr
)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::too_many_lines
)]

use std::sync::Arc;

use i18n::{osaka_i_18_n::OsakaI18N, pt_br::pt_br, OsakaLocale};
use poise::{
    serenity_prelude::{futures::TryFutureExt, GatewayIntents},
    Context, FrameworkOptions,
};
use poise_i18n::{apply_translations, PoiseI18NMeta};
use rusty18n::I18NWrapper;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolOptions, Pool, Postgres};
use tracing::error;

pub mod commands;
pub mod error;
pub mod i18n;
pub mod managers;
pub mod osaka_sqlx;
pub mod responses;
pub mod setup;
pub mod utils;

pub type OsakaContext<'a> = Context<'a, OsakaData, error::Osaka>;
pub type OsakaResult = Result<(), error::Osaka>;

#[derive(Serialize, Deserialize)]
pub struct OsakaConfig {
    pub bot_token: String,
    pub development_guild: Option<u64>,

    pub osu_client_id: u64,
    pub osu_client_secret: String,

    pub database_url: String,
}

pub struct OsakaData {
    pub i18n: I18NWrapper<OsakaLocale, OsakaI18N>,
    pub rosu: Arc<rosu_v2::Osu>,
    pub config: OsakaConfig,
    pub pool: Pool<Postgres>,
    pub managers: managers::Osaka,
}

impl<'a> PoiseI18NMeta<'a, OsakaLocale, OsakaI18N> for OsakaContext<'a> {
    fn locales(&self) -> &'a I18NWrapper<OsakaLocale, OsakaI18N> {
        &self.data().i18n
    }
}

#[tokio::main]
async fn main() -> OsakaResult {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_target(true)
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();

    let config = envy::from_env::<OsakaConfig>()?;

    let pool = PoolOptions::<Postgres>::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut commands = vec![
        commands::booru::booru(),
        commands::user::user(),
        commands::fun::fun(),
        commands::gif::gif(),
        commands::owner::owner(),
        commands::osu::osu(),
    ];

    let i18n =
        I18NWrapper::<OsakaLocale, OsakaI18N>::new(vec![(OsakaLocale::BrazilianPortuguese, pt_br)]);
    apply_translations(&mut commands, &i18n);

    poise::Framework::builder()
        .options(FrameworkOptions {
            commands,
            on_error: |err| Box::pin(error::handle(err).unwrap_or_else(|e| error!("{}", e))),
            ..Default::default()
        })
        .token(&config.bot_token)
        .intents(GatewayIntents::non_privileged())
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move { setup::setup(ctx, framework, config, i18n, pool).await })
        })
        .run()
        .await?;

    Ok(())
}
