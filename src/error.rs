use poise::{serenity_prelude, FrameworkError};
use poise_i18n::PoiseI18NTrait;
use rusty18n::{t, I18NAccessible};

use crate::OsakaData;
use log::error;

#[derive(thiserror::Error, derive_more::From, Debug)]
pub enum OsakaError {
    #[error(transparent)]
    Serenity(serenity_prelude::SerenityError),

    #[error(transparent)]
    Envy(envy::Error),

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    Migrate(sqlx::migrate::MigrateError),

    #[error(transparent)]
    Url(url::ParseError),

    #[error("Something really sketchy happened!")]
    SimplyUnexpected,
}

pub async fn on_error(
    err: poise::FrameworkError<'_, OsakaData, OsakaError>,
) -> Result<(), OsakaError> {
    match err {
        FrameworkError::Command { error, ctx } => {
            let i18n = ctx.i18n();

            let response = match error {
                OsakaError::Serenity(..)
                | OsakaError::Sqlx(..)
                | OsakaError::Migrate(..)
                | OsakaError::Envy(..)
                | OsakaError::Url(..)
                | OsakaError::SimplyUnexpected => {
                    log::error!("{error}");
                    t!(i18n.errors.unexpected)
                }
            };

            ctx.send(|r| r.content(response)).await?;
        }
        e => poise::builtins::on_error(e).await?,
    }
    Ok(())
}
