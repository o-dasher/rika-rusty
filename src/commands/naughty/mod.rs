pub mod booru;

use crate::create_command_group;

use booru::search;

create_command_group!(naughty, ["search"]);
