use crate::{commands::owner::register::register, create_command_group};

pub mod register;

create_command_group!(owner, ["register"]);
