use super::{emojis::OsakaMoji, markdown::bold};

pub fn cool_text(emoji: OsakaMoji, text: &str) -> String {
    bold(format!("{emoji} | {text}"))
}

pub fn something_wrong(text: &str) -> String {
    cool_text(OsakaMoji::X, text)
}
