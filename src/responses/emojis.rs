use std::fmt::Display;

use poise::serenity_prelude::ReactionType;
use strum::IntoStaticStr;

#[derive(IntoStaticStr, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum OsakaMoji {
    ZanyFace,
    ArrowForward,
    ArrowBackward,
    X,
}

impl From<OsakaMoji> for char {
    fn from(value: OsakaMoji) -> Self {
        match value {
            OsakaMoji::ArrowForward => '▶',
            OsakaMoji::ArrowBackward => '◀',
            OsakaMoji::X => '❌',
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
        write!(f, "{}", {
            match self {
                OsakaMoji::X => char::from(self.clone()).to_string(),
                OsakaMoji::ZanyFace | OsakaMoji::ArrowForward | OsakaMoji::ArrowBackward => {
                    format!(":{}:", Into::<&'static str>::into(self))
                }
            }
        })
    }
}
