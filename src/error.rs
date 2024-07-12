use crate::{
    commands,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
    managers::{self, osu::submit::SubmissionError},
    responses, OsakaContext, OsakaData,
};
use chrono::OutOfRangeError;
use poise::{serenity_prelude, FrameworkError};
use poise_i18n::PoiseI18NTrait;
use rusty18n::{t_prefix, I18NAccess, I18NWrapper};
use strum::Display;
use tracing::error;

use crate::managers::register_command::Error;

#[derive(thiserror::Error, derive_more::From, Debug)]
pub enum Osaka {
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
    Rosu(rosu_v2::error::OsuError),

    #[error(transparent)]
    Notify(Notify),

    #[error(transparent)]
    OsuCommand(commands::osu::Error),

    #[error(transparent)]
    RegisterCommand(managers::register_command::Error),

    #[error(transparent)]
    Submission(SubmissionError),

    #[error("Something really sketchy happened!")]
    SimplyUnexpected,
}

#[derive(Debug, Display, thiserror::Error)]
pub enum Notify {
    Warn(String),
    MissingPermissions,
}

pub trait Translated {
    fn get_response<'a>(
        self,
        i18n: &'a I18NAccess<I18NWrapper<OsakaLocale, OsakaI18N>>,
    ) -> &'a String;
}

fn get_response(ctx: OsakaContext, error: Osaka) -> String {
    let i18n = ctx.i18n();
    t_prefix!($i18n.errors);

    match error {
        Osaka::Serenity(..)
        | Osaka::Sqlx(..)
        | Osaka::Migrate(..)
        | Osaka::Envy(..)
        | Osaka::Url(..)
        | Osaka::Booru(..)
        | Osaka::Reqwest(..)
        | Osaka::DurationOutOfRange(..)
        | Osaka::Rosu(..)
        | Osaka::Submission(..)
        | Osaka::SimplyUnexpected => {
            error!("{}", error);
            t!(unexpected).clone()
        }
        Osaka::RegisterCommand(e) => match e {
            Error::Serenity(e) => get_response(ctx, e.into()),
            Error::NoDevelopmentGuildSet => t!(register.no_development_guild_set).clone(),
        },
        Osaka::Notify(e) => match e {
            Notify::Warn(warn) => warn,
            Notify::MissingPermissions => t!(user_missing_permissions).clone(),
        },
        Osaka::OsuCommand(e) => e.get_response(&i18n).to_string(),
    }
}

pub async fn handle(err: poise::FrameworkError<'_, OsakaData, Osaka>) -> Result<(), Osaka> {
    match err {
        FrameworkError::Command { error, ctx } => {
            ctx.say(&responses::templates::something_wrong(&get_response(
                ctx, error,
            )))
            .await?;
        }
        e => poise::builtins::on_error(e).await?,
    }
    Ok(())
}
