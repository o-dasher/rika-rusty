use crate::{responses, OsakaData};
use chrono::OutOfRangeError;
use log::error;
use poise::{serenity_prelude, FrameworkError};
use poise_i18n::PoiseI18NTrait;
use rusty18n::{t, I18NAccessible};
use strum::Display;

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

    #[error(transparent)]
    Booru(rusty_booru::shared::Error),

    #[error(transparent)]
    DurationOutOfRange(OutOfRangeError),

    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    Notify(NotifyError),

    #[error("Something really sketchy happened!")]
    SimplyUnexpected,
}

#[derive(Debug, Display, thiserror::Error)]
pub enum NotifyError {
    Warn(String),
    MissingPermissions,
}

pub async fn on_error(
    err: poise::FrameworkError<'_, OsakaData, OsakaError>,
) -> Result<(), OsakaError> {
    match err {
        FrameworkError::Command { error, ctx } => {
            let i18n = ctx.i18n();

            enum ErrorResponse {
                Say(String),
            }

            let response = match error {
                OsakaError::Serenity(..)
                | OsakaError::Sqlx(..)
                | OsakaError::Migrate(..)
                | OsakaError::Envy(..)
                | OsakaError::Url(..)
                | OsakaError::Booru(..)
                | OsakaError::Reqwest(..)
                | OsakaError::DurationOutOfRange(..)
                | OsakaError::SimplyUnexpected => {
                    log::error!("{error}");
                    ErrorResponse::Say(t!(i18n.errors.unexpected).clone())
                }
                OsakaError::Notify(e) => match e {
                    NotifyError::Warn(warn) => ErrorResponse::Say(warn),
                    NotifyError::MissingPermissions => {
                        ErrorResponse::Say(t!(i18n.errors.user_missing_permissions).clone())
                    }
                },
            };

            match response {
                ErrorResponse::Say(say) => {
                    ctx.say(&responses::templates::something_wrong(&say))
                        .await?;
                }
            }
        }
        e => poise::builtins::on_error(e).await?,
    }
    Ok(())
}
