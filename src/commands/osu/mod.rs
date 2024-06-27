use crate::{commands::osu::link::link, create_command_group};

pub mod link;
pub mod submit;

create_command_group!(osu, ["link"]);
