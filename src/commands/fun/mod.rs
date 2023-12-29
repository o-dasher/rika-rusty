pub mod coinflip;

use crate::create_command_group;
use coinflip::coinflip;

create_command_group!(fun, ["coinflip"]);
