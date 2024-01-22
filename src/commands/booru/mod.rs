pub mod search;
pub mod blacklist;

use crate::create_command_group;
use poise::ChoiceParameter;
use search::search;
use blacklist::blacklist;

create_command_group!(booru, ["search", "blacklist"]);

#[derive(ChoiceParameter, Clone, Copy)]
pub enum SettingKind {
    Guild,
    Channel,
    User,
}

