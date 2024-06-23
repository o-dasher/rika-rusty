use crate::{
    managers::register_command_manager::RegisterError, responses, OsakaContext,
    OsakaData,
};
use chrono::OutOfRangeError;
use poise::{serenity_prelude, FrameworkError};
use poise_i18n::PoiseI18NTrait;
use rusty18n::{t,};
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

    #[error(transparent)]
    RegisterCommand(RegisterError),

    #[error("Something really sketchy happened!")]
    SimplyUnexpected,
}

#[derive(Debug, Display, thiserror::Error)]
pub enum NotifyError {
    Warn(String),
    MissingPermissions,
}

fn get_error_response(ctx: OsakaContext, error: OsakaError) -> String {
    let i18n = ctx.i18n();

    match error {
        OsakaError::Serenity(..)
        | OsakaError::Sqlx(..)
        | OsakaError::Migrate(..)
        | OsakaError::Envy(..)
        | OsakaError::Url(..)
        | OsakaError::Booru(..)
        | OsakaError::Reqwest(..)
        | OsakaError::DurationOutOfRange(..)
        | OsakaError::SimplyUnexpected => t!(i18n.errors.unexpected).clone(),
        OsakaError::RegisterCommand(e) => match e {
            RegisterError::Serenity(e) => get_error_response(ctx, e.into()),
            RegisterError::NoDevelopmentGuildSet => {
                "Failed to register commands no development guild set".to_string()
            }
        },
        OsakaError::Notify(e) => match e {
            NotifyError::Warn(warn) => warn.to_string(),
            NotifyError::MissingPermissions => t!(i18n.errors.user_missing_permissions).clone(),
        },
    }
}

pub async fn on_error(
    err: poise::FrameworkError<'_, OsakaData, OsakaError>,
) -> Result<(), OsakaError> {
    match err {
        FrameworkError::Command { error, ctx } => {
            ctx.say(&responses::templates::something_wrong(&get_error_response(
                ctx, error,
            )))
            .await?;
        }
        e => poise::builtins::on_error(e).await?,
    }
    Ok(())
}
