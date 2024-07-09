use osu::Manager;

pub mod osu;
pub mod register_command;

pub struct Osaka {
    pub register_command_manager: register_command::Manager,
    pub osu_manager: Manager,
}

impl Osaka {
    #[must_use]
    pub fn new() -> Self {
        Self {
            register_command_manager: register_command::Manager(),
            osu_manager: osu::Manager::new(),
        }
    }
}
