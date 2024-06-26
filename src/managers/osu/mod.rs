use beatmap::BeatmapCacheManager;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use submit::ScoreSubmitter;

pub mod beatmap;
pub mod submit;

pub struct OsuManager {
    pub beatmap_cache_manager: BeatmapCacheManager,
    pub submit_manager: ScoreSubmitter,
}

impl OsuManager {
    pub fn new(
        cache: Arc<BeatmapCacheManager>,
        pool: Pool<Postgres>,
        rosu: Arc<rosu_v2::Osu>,
    ) -> Self {
        Self {
            beatmap_cache_manager: BeatmapCacheManager::new(),
            submit_manager: ScoreSubmitter::new(cache, pool, rosu),
        }
    }
}
