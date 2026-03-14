use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::{CompanyDetail, CompanySearchResponse, FilingsResponse};

const BASE_URL: &str = "https://core.cro.ie/api";
const CACHE_TTL: Duration = Duration::from_secs(86400); // 24 hours

pub struct CroApi {
    client: HttpClient,
    cache: Cache,
}

impl CroApi {
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

    pub async fn search_companies(
        &self,
        name: &str,
        status: Option<&str>,
    ) -> Result<CompanySearchResponse, IrlError> {
        let encoded_name = name.replace(' ', "%20");
        let mut url = format!("{}/company?company_name={}", BASE_URL, encoded_name);
        if let Some(status) = status {
            url.push_str(&format!("&company_status={}", status));
        }
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse company search: {}", e)))
    }

    pub async fn get_company(&self, number: &str) -> Result<CompanyDetail, IrlError> {
        let url = format!("{}/company/{}", BASE_URL, number);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse company detail: {}", e)))
    }

    pub async fn get_filings(
        &self,
        number: &str,
        filing_type: Option<&str>,
    ) -> Result<FilingsResponse, IrlError> {
        let mut url = format!("{}/company/{}/filings", BASE_URL, number);
        if let Some(ft) = filing_type {
            url.push_str(&format!("?filing_type={}", ft));
        }
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse filings: {}", e)))
    }
}
