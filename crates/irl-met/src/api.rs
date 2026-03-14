use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::{Observation, Warning};

const OBSERVATIONS_BASE_URL: &str = "https://prodapi.metweb.ie/observations";
const WARNINGS_URL: &str = "https://www.met.ie/Open_Data/json/warning_IRELAND.json";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

pub struct MetApi {
    client: HttpClient,
    cache: Cache,
}

impl MetApi {
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

    /// Fetch today's observations for a given station name.
    /// The station name should be the API name (e.g. "Dublin Airport").
    pub async fn get_observations(&self, station: &str) -> Result<Vec<Observation>, IrlError> {
        let encoded = station.replace(' ', "%20");
        let url = format!("{}/{}/today", OBSERVATIONS_BASE_URL, encoded);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse observations: {}", e)))
    }

    /// Fetch current weather warnings for Ireland.
    /// Returns an empty vec when no warnings are active.
    pub async fn get_warnings(&self) -> Result<Vec<Warning>, IrlError> {
        let text = self.get_cached(WARNINGS_URL).await?;

        // The API returns an empty array "[]" when no warnings are active.
        // It may also return an empty string or whitespace-only response.
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_str(trimmed)
            .map_err(|e| IrlError::Parse(format!("Failed to parse warnings: {}", e)))
    }
}
