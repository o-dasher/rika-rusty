use crate::create_command_group;

pub mod avatar;

use avatar::avatar;

create_command_group!(user, ["avatar"]);
