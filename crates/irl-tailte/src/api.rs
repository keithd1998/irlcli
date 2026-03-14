use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::BrowserLikeClient;

use crate::models::{
    AreaResponse, CategoriesResponse, PropertyValuation, ValuationSearchResponse,
};

const BASE_URL: &str = "https://opendata.tailte.ie/api";
const CACHE_TTL: Duration = Duration::from_secs(86400); // 24 hours

pub struct TailteApi {
    client: BrowserLikeClient,
    cache: Cache,
}

impl TailteApi {
    pub fn new(verbose: bool, quiet: bool, no_cache: bool) -> Result<Self, IrlError> {
        Ok(Self {
            client: BrowserLikeClient::new(verbose, quiet)?,
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

    pub async fn search_valuations(
        &self,
        address: &str,
    ) -> Result<ValuationSearchResponse, IrlError> {
        let encoded = address.replace(' ', "%20");
        let url = format!("{}/valuations?address={}", BASE_URL, encoded);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse valuation search: {}", e)))
    }

    pub async fn get_property(
        &self,
        property_number: &str,
    ) -> Result<PropertyValuation, IrlError> {
        let url = format!("{}/valuations/{}", BASE_URL, property_number);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse property valuation: {}", e)))
    }

    pub async fn get_area(
        &self,
        rating_authority: &str,
    ) -> Result<AreaResponse, IrlError> {
        let encoded = rating_authority.replace(' ', "%20");
        let url = format!("{}/valuations?rating_authority={}", BASE_URL, encoded);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse area data: {}", e)))
    }

    pub async fn get_categories(&self) -> Result<CategoriesResponse, IrlError> {
        let url = format!("{}/categories", BASE_URL);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse categories: {}", e)))
    }
}
