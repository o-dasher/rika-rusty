use std::future::Future;

use crate::{
    commands::osu::{link::link, submit::submit},
    create_command_group, error,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
    osaka_sqlx::{booru_setting::SettingKind, discord},
};
use poise::ChoiceParameter;
use rosu_v2::prelude::GameMode;
use rusty18n::{t, I18NAccess, I18NWrapper};
use strum::Display;

pub mod link;
pub mod recommend;
pub mod submit;

create_command_group!(osu, ["link", "submit"]);

#[derive(ChoiceParameter, Default, Clone, Copy)]
#[repr(u8)]
pub enum Mode {
    #[default]
    #[name = "osu"]
    Osu = 0,

    #[name = "taiko"]
    Taiko = 1,

    #[name = "catch"]
    Catch = 2,

    #[name = "mania"]
    Mania = 3,
}

impl From<Mode> for GameMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Osu => Self::Osu,
            Mode::Taiko => Self::Taiko,
            Mode::Catch => Self::Catch,
            Mode::Mania => Self::Mania,
        }
    }
}

#[derive(thiserror::Error, Debug, Display, derive_more::From)]
pub enum Error {
    NotLinked,
}

impl error::Translated for Error {
    fn get_response<'a>(
        self,
        i18n: &'a I18NAccess<I18NWrapper<OsakaLocale, OsakaI18N>>,
    ) -> &'a str {
        match self {
            Self::NotLinked => t!(i18n.osu.errors.not_linked).as_str(),
        }
    }
}

pub trait Context {
    fn get_linked_user(&self) -> impl Future<Output = Result<i64, error::Osaka>> + Send;
}

impl Context for OsakaContext<'_> {
    async fn get_linked_user(&self) -> Result<i64, error::Osaka> {
        sqlx::query_as!(
            discord::User,
            "SELECT * FROM discord_user WHERE id=$1",
            SettingKind::User.get_sqlx_id(*self)?
        )
        .fetch_one(&self.data().pool)
        .await
        .ok()
        .ok_or(Error::NotLinked)?
        .osu_user_id
        .ok_or(Error::NotLinked)
        .map_err(error::Osaka::from)
    }
}
