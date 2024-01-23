pub mod blacklist;
pub mod search;

use crate::create_command_group;
use blacklist::blacklist;
use poise::ChoiceParameter;
use search::search;
use sqlx::types::BigDecimal;

create_command_group!(booru, ["search", "blacklist"]);

#[derive(ChoiceParameter, Clone, Copy)]
pub enum SettingKind {
    Guild,
    Channel,
    User,
}

pub struct BooruContext<'a>(OsakaContext<'a>);

impl<'a> BooruContext<'a> {
    fn acquire(value: impl Into<u64>) -> BigDecimal {
        Into::<u64>::into(value).clone().into()
    }

    pub fn guild(&self) -> Option<BigDecimal> {
        self.0.guild_id().map(Self::acquire)
    }

    pub fn channel(&self) -> BigDecimal {
        Self::acquire(self.0.channel_id())
    }

    pub fn user(&self) -> BigDecimal {
       Self:: acquire(self.0.author().id)
    }
}
