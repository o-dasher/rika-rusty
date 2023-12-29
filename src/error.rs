use std::error::Error;

use poise::{serenity_prelude, FrameworkError};
use poise_i18n::PoiseI18NTrait;
use rusty18n::{t, I18NAccessible};

use crate::OsakaData;
use log::error;

pub type BoxedError = Box<dyn Error + Send + Sync>;

#[derive(thiserror::Error, derive_more::From, Debug)]
pub enum OsakaError {
    #[error(transparent)]
    Serenity(serenity_prelude::SerenityError),

    #[error(transparent)]
    Unexpected(BoxedError),

    #[error("Something really sketchy happened!")]
    SimplyUnexpected,
}

impl OsakaError {
    pub fn unexpect<E: Into<BoxedError>>(err: E) -> OsakaError {
        OsakaError::Unexpected(err.into())
    }
}

pub async fn on_error(
    err: poise::FrameworkError<'_, OsakaData, OsakaError>,
) -> Result<(), OsakaError> {
    match err {
        FrameworkError::Command { error, ctx } => {
            let i18n = ctx.i18n();

            let response = match error {
                OsakaError::Unexpected(..)
                | OsakaError::Serenity(..)
                | OsakaError::SimplyUnexpected => {
                    log::error!("{error}");
                    t!(i18n.errors.unexpected)
                }
            };

            ctx.send(|r| r.content(response)).await?;
        }
        e => poise::builtins::on_error(e)
            .await
            .map_err(OsakaError::unexpect)?,
    }
    Ok(())
}
