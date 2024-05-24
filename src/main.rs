use error::OsakaError;
use i18n::{osaka_i_18_n::OsakaI18N, pt_br::pt_br, OsakaLocale};
use poise::{
    serenity_prelude::{futures::TryFutureExt, GatewayIntents},
    Context, FrameworkOptions,
};
use poise_i18n::{apply_translations, PoiseI18NMeta};
use rusty18n::I18NWrapper;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolOptions, Pool, Postgres};

pub mod commands;
pub mod error;
pub mod i18n;
pub mod responses;
pub mod setup;
pub mod utils;

pub type OsakaContext<'a> = Context<'a, OsakaData, OsakaError>;
pub type OsakaResult = Result<(), OsakaError>;

#[derive(Serialize, Deserialize)]
pub struct OsakaConfig {
    pub bot_token: String,
    pub development_guild: Option<u64>,

    pub database_url: String,
}

pub struct OsakaData {
    pub i18n: I18NWrapper<OsakaLocale, OsakaI18N>,
    pub pool: Pool<Postgres>,
}

impl<'a> PoiseI18NMeta<'a, OsakaLocale, OsakaI18N> for OsakaContext<'a> {
    fn locales(&self) -> &'a I18NWrapper<OsakaLocale, OsakaI18N> {
        &self.data().i18n
    }
}

#[tokio::main]
async fn main() -> OsakaResult {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_target(true).pretty().init();

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
    ];

    let i18n =
        I18NWrapper::<OsakaLocale, OsakaI18N>::new(vec![(OsakaLocale::BrazilianPortuguese, pt_br)]);
    apply_translations(&mut commands, &i18n);

    poise::Framework::builder()
        .options(FrameworkOptions {
            commands,
            on_error: |err| Box::pin(error::on_error(err).unwrap_or_else(|e| log::error!("{}", e))),
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
