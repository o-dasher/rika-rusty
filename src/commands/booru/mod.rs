pub mod search;
pub mod blacklist;

use crate::create_command_group;
use search::search;
use blacklist::blacklist;

create_command_group!(booru, ["search", "blacklist"]);
