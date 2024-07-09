use crate::{
    commands::osu::{link::link, submit::submit},
    create_command_group,
};

pub mod link;
pub mod submit;

create_command_group!(osu, ["link", "submit"]);
