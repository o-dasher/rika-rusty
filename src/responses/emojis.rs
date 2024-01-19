use std::fmt::Display;

use poise::serenity_prelude::ReactionType;
use strum::IntoStaticStr;

#[derive(IntoStaticStr, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum OsakaMoji {
    ZanyFace,
    ArrowForward,
    ArrowBackward,
    Close,
}

impl From<OsakaMoji> for char {
    fn from(value: OsakaMoji) -> Self {
        match value {
            OsakaMoji::ArrowForward => '▶',
            OsakaMoji::ArrowBackward => '◀',
            OsakaMoji::Close => '❌',
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
            let as_char = char::from(*self);
            if as_char != ' ' {
                as_char.to_string()
            } else {
                format!(":{}:", self.to_string())
            }
        })
    }
}
