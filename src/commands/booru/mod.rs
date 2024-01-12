pub mod search;

use crate::create_command_group;
use search::search;

create_command_group!(booru, ["search"]);
