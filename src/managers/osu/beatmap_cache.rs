use std::sync::Arc;

use fchashmap::FcHashMap;
use strum::Display;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Manager {
    pub client: reqwest::Client,
    pub cache: Arc<Mutex<FcHashMap<u32, Arc<Vec<u8>>, 256>>>,
}

#[derive(thiserror::Error, Display, Debug, derive_more::From)]
pub enum Error {
    Client(reqwest::Error),
}

impl Manager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(FcHashMap::new())),
        }
    }

    pub async fn get_beatmap_file(&self, beatmap_id: u32) -> Result<Arc<Vec<u8>>, Error> {
        if let Some(cached) = self.cache.lock().await.get(&beatmap_id) {
            return Ok(Arc::clone(cached));
        };

        let response = self
            .client
            .get(&format!("https://osu.ppy.sh/osu/{beatmap_id}"))
            .send()
            .await?;

        let map_bytes = Arc::new(response.bytes().await.map(|b| b.to_vec())?);

        let _ = self
            .cache
            .lock()
            .await
            .insert(beatmap_id, Arc::clone(&map_bytes));

        Ok(map_bytes)
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
