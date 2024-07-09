use std::sync::Arc;
use submit::ScoreSubmitter;
use tokio::sync::RwLock;

pub mod beatmap_cache;
pub mod submit;

pub struct Manager {
    pub beatmap_cache_manager: Arc<beatmap_cache::Manager>,
    pub submit_manager: Arc<RwLock<ScoreSubmitter>>,
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Manager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            beatmap_cache_manager: Arc::new(beatmap_cache::Manager::new()),
            submit_manager: Arc::new(RwLock::new(ScoreSubmitter::new())),
        }
    }
}
