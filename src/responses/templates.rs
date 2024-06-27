use super::{emojis::OsakaMoji, markdown::bold};

#[must_use]
pub fn cool_text(emoji: OsakaMoji, text: &str) -> String {
    bold(format!("{emoji} | {text}"))
}

#[must_use]
pub fn something_wrong(text: &str) -> String {
    cool_text(OsakaMoji::X, text)
}
