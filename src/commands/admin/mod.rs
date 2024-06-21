use crate::{commands::admin::register::register, create_command_group};

pub mod register;

create_command_group!(admin, ["register"]);
