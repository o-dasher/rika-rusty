use std::sync::Arc;

use fchashmap::FcHashMap;
use strum::Display;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct BeatmapCache {
    pub client: reqwest::Client,
    pub cache: Arc<Mutex<FcHashMap<u32, Arc<[u8]>, 256>>>,
}

#[derive(thiserror::Error, Display, Debug, derive_more::From)]
pub enum BeatmapCacheError {
    Client(reqwest::Error),
}

impl Default for BeatmapCache {
    fn default() -> Self {
        Self::new()
    }
}

impl BeatmapCache {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(FcHashMap::new())),
        }
    }

    pub async fn get_beatmap_file(&self, beatmap_id: u32) -> Result<Arc<[u8]>, BeatmapCacheError> {
        if let Some(cached) = self.cache.lock().await.get(&beatmap_id) {
            return Ok(cached.clone());
        };

        let response = self
            .client
            .get(&format!("https://osu.ppy.sh/osu/{beatmap_id}"))
            .send()
            .await?;

        let map_bytes: Arc<[u8]> = response
            .bytes()
            .await
            .map(|bytes| Vec::<u8>::from(bytes).into())?;

        let _ = self
            .cache
            .lock()
            .await
            .insert(beatmap_id, map_bytes.clone());

        Ok(map_bytes)
    }
}
