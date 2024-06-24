use crate::commands::osu::link::link;
use crate::create_command_group;

pub mod link;

create_command_group!(osu, ["link"]);
