use sqlx::{Pool, Postgres};
use std::sync::Arc;
use submit::ScoreSubmitter;

pub mod beatmap_cache;
pub mod submit;

pub struct Manager {
    pub beatmap_cache_manager: Arc<beatmap_cache::Manager>,
    pub submit_manager: Arc<ScoreSubmitter>,
}

impl Manager {
    #[must_use]
    pub fn new(rosu: Arc<rosu_v2::Osu>, pool: Pool<Postgres>) -> Self {
        let beatmap_cache_manager = Arc::new(beatmap_cache::Manager::new());

        Self {
            beatmap_cache_manager: Arc::clone(&beatmap_cache_manager),
            submit_manager: Arc::new(ScoreSubmitter::new(beatmap_cache_manager, rosu, pool)),
        }
    }
}
