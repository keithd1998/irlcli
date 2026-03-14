use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};

use irl_core::cache::Cache;
use irl_core::config::Config;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::GtfsResponse;

const VEHICLES_URL: &str = "https://api.nationaltransport.ie/gtfsr/v2/Vehicles?format=json";
const TRIP_UPDATES_URL: &str = "https://api.nationaltransport.ie/gtfsr/v2/TripUpdates?format=json";
const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

pub struct TransportApi {
    client: HttpClient,
    cache: Cache,
    api_key: String,
}

impl TransportApi {
    pub fn new(verbose: bool, quiet: bool, no_cache: bool) -> Result<Self, IrlError> {
        let config = Config::load().unwrap_or_default();
        let api_key = config.transport.api_key.clone();

        if api_key.is_empty() {
            return Err(IrlError::Other(
                "API key required. Register at https://developer.nationaltransport.ie/ \
                 then run: irl config set transport.api_key YOUR_KEY"
                    .to_string(),
            ));
        }

        Ok(Self {
            client: HttpClient::new(verbose, quiet)?,
            cache: Cache::new(!no_cache),
            api_key,
        })
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Ok(val) = HeaderValue::from_str(&self.api_key) {
            headers.insert("x-api-key", val);
        }
        headers
    }

    async fn get_cached(&self, url: &str) -> Result<String, IrlError> {
        if let Some(cached) = self.cache.get(url) {
            return Ok(cached);
        }
        let headers = self.auth_headers();
        let response = self.client.get_with_headers(url, &headers).await?;
        let text = response.text().await.map_err(IrlError::Http)?;
        let _ = self.cache.set(url, &text, CACHE_TTL);
        Ok(text)
    }

    pub async fn get_vehicle_positions(&self) -> Result<GtfsResponse, IrlError> {
        let text = self.get_cached(VEHICLES_URL).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse vehicle positions: {}", e)))
    }

    pub async fn get_trip_updates(&self) -> Result<GtfsResponse, IrlError> {
        let text = self.get_cached(TRIP_UPDATES_URL).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse trip updates: {}", e)))
    }
}
