use beatmap::BeatmapCacheManager;

pub mod beatmap;

pub struct OsuManager {
    pub beatmap_cache_manager: BeatmapCacheManager,
}

impl OsuManager {
    pub fn new() -> Self {
        Self {
            beatmap_cache_manager: BeatmapCacheManager::new(),
        }
    }
}
