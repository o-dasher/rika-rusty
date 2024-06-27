use osu::Manager;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::OsakaConfig;

pub mod osu;
pub mod register_command;

pub struct Osaka {
    pub register_command_manager: register_command::Manager,
    pub osu_manager: Manager,
}

impl Osaka {
    #[must_use]
    pub fn new(config: Arc<OsakaConfig>, pool: Pool<Postgres>, rosu: Arc<rosu_v2::Osu>) -> Self {
        Self {
            register_command_manager: register_command::Manager::new(config),
            osu_manager: osu::Manager::new(pool, rosu),
        }
    }
}
