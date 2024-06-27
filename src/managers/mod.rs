use sqlx::Pool;
use std::sync::Arc;

use osu::OsuManager;
use sqlx::Postgres;

use crate::OsakaConfig;

pub mod osu;
pub mod register_command;

pub struct Osaka {
    pub register_command_manager: register_command::Manager,
    pub osu_manager: OsuManager,
}

impl Osaka {
    pub fn new(config: Arc<OsakaConfig>, pool: Pool<Postgres>, rosu: Arc<rosu_v2::Osu>) -> Self {
        Self {
            register_command_manager: register_command::Manager::new(config),
            osu_manager: OsuManager::new(pool, rosu),
        }
    }
}
