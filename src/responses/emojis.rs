use std::fmt::Display;

use poise::serenity_prelude::ReactionType;
use strum::IntoStaticStr;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum OsakaMoji {
    ZanyFace,
    ArrowForward,
    ArrowBackward,
}

impl From<OsakaMoji> for char {
    fn from(value: OsakaMoji) -> Self {
        match value {
            OsakaMoji::ArrowForward => '▶',
            OsakaMoji::ArrowBackward => '◀',
            OsakaMoji::ZanyFace => ' ',
        }
    }
}

impl From<OsakaMoji> for ReactionType {
    fn from(value: OsakaMoji) -> Self {
        char::from(value).into()
    }
}

impl Display for OsakaMoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_v: &'static str = self.into();
        write!(f, ":{}:", str_v)
    }
}
