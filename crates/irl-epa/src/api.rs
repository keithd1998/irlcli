use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::{
    AirQualityResponse, EmissionsResponse, FacilitiesResponse, WaterQualityResponse,
};

const AIR_QUALITY_URL: &str = "https://airquality.ie/api/air-quality";
const WATER_QUALITY_URL: &str = "https://epa.ie/api/water-quality";
const FACILITIES_URL: &str = "https://epa.ie/api/facilities";
const EMISSIONS_URL: &str = "https://epa.ie/api/emissions";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

pub struct EpaApi {
    client: HttpClient,
    cache: Cache,
}

impl EpaApi {
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

    pub async fn get_air_quality(
        &self,
        station: Option<&str>,
    ) -> Result<AirQualityResponse, IrlError> {
        let url = match station {
            Some(s) => format!("{}?station={}", AIR_QUALITY_URL, s),
            None => AIR_QUALITY_URL.to_string(),
        };
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse air quality data: {}", e)))
    }

    pub async fn get_water_quality(
        &self,
        catchment: Option<&str>,
    ) -> Result<WaterQualityResponse, IrlError> {
        let url = match catchment {
            Some(c) => format!("{}?catchment={}", WATER_QUALITY_URL, c),
            None => WATER_QUALITY_URL.to_string(),
        };
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse water quality data: {}", e)))
    }

    pub async fn get_facilities(
        &self,
        county: Option<&str>,
    ) -> Result<FacilitiesResponse, IrlError> {
        let url = match county {
            Some(c) => format!("{}?county={}", FACILITIES_URL, c),
            None => FACILITIES_URL.to_string(),
        };
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse facilities data: {}", e)))
    }

    pub async fn get_emissions(
        &self,
        sector: Option<&str>,
    ) -> Result<EmissionsResponse, IrlError> {
        let url = match sector {
            Some(s) => format!("{}?sector={}", EMISSIONS_URL, s),
            None => EMISSIONS_URL.to_string(),
        };
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse emissions data: {}", e)))
    }
}
