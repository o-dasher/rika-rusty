use paste::paste;
use poise::serenity_prelude::{Content, MessageBuilder};

macro_rules! md {
    ($mode:tt) => {
        pub fn $mode(value: impl Into<Content>) -> String {
            paste! {
                MessageBuilder::new().[<push_ $mode _safe>](value).build()
            }
        }
    };
    ($($mode:tt),+) => {
        $(md!($mode);)+
    };
}

md!(mono, bold);
