use super::{emojis::OsakaMoji, markdown::bold};

pub fn cool_text(emoji: OsakaMoji, text: &str) -> String {
    bold(format!("{} | {}", emoji, text))
}
