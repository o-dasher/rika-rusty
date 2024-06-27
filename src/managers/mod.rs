use sqlx::Pool;
use std::sync::Arc;

use osu::OsuManager;
use sqlx::Postgres;

use crate::{OsakaConfig, RegisterCommandManager};

pub mod osu;
pub mod register;

pub struct OsakaManagers {
    pub register_command_manager: RegisterCommandManager,
    pub osu_manager: OsuManager,
}

impl OsakaManagers {
    pub fn new(config: Arc<OsakaConfig>, pool: Pool<Postgres>, rosu: Arc<rosu_v2::Osu>) -> Self {
        Self {
            register_command_manager: RegisterCommandManager::new(config),
            osu_manager: OsuManager::new(pool, rosu),
        }
    }
}
