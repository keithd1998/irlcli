use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::FeatureCollection;

const STATIONS_URL: &str = "https://waterlevel.ie/geojson/";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

pub struct WaterApi {
    client: HttpClient,
    cache: Cache,
}

impl WaterApi {
    pub fn new(verbose: bool, quiet: bool, no_cache: bool) -> Result<Self, IrlError> {
        Ok(Self {
            client: HttpClient::new(verbose, quiet)?,
            cache: Cache::new(!no_cache),
        })
    }

    async fn get_cached(&self, url: &str) -> Result<String, IrlError> {
        if let Some(cached) = self.cache.get(url) {
            return Ok(cached);
        }
        let text = self.client.get_text(url).await?;
        let _ = self.cache.set(url, &text, CACHE_TTL);
        Ok(text)
    }

    pub async fn get_stations(&self) -> Result<FeatureCollection, IrlError> {
        let text = self.get_cached(STATIONS_URL).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse water stations GeoJSON: {}", e)))
    }
}
