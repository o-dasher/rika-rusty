use crate::{
    commands::osu::{link::link, submit::submit},
    create_command_group,
};
use poise::ChoiceParameter;

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
