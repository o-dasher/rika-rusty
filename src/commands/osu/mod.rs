use crate::{
    commands::osu::{link::link, submit::submit},
    create_command_group, error,
    i18n::{osaka_i_18_n::OsakaI18N, OsakaLocale},
};
use poise::ChoiceParameter;
use rusty18n::{t, I18NAccess, I18NWrapper};
use strum::Display;

pub mod link;
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
            Self::NotLinked => &t!(i18n.osu.errors.not_linked),
        }
    }
}
