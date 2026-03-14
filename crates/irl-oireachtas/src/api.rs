use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::*;

const BASE_URL: &str = "https://api.oireachtas.ie/v1";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

pub struct OireachtasApi {
    client: HttpClient,
    cache: Cache,
}

impl OireachtasApi {
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

    pub async fn list_members(
        &self,
        chamber: Option<&str>,
        limit: u32,
        skip: u32,
    ) -> Result<ApiResponse<MemberResult>, IrlError> {
        let mut url = format!(
            "{}/members?limit={}&skip={}&date_start=2020-02-08",
            BASE_URL, limit, skip
        );
        if let Some(chamber) = chamber {
            url.push_str(&format!("&chamber={}", chamber));
        }
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse members: {}", e)))
    }

    pub async fn list_legislation(
        &self,
        limit: u32,
        skip: u32,
    ) -> Result<ApiResponse<BillResult>, IrlError> {
        let url = format!("{}/legislation?limit={}&skip={}", BASE_URL, limit, skip);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse legislation: {}", e)))
    }

    pub async fn list_divisions(
        &self,
        limit: u32,
        skip: u32,
    ) -> Result<ApiResponse<DivisionResult>, IrlError> {
        let url = format!("{}/divisions?limit={}&skip={}", BASE_URL, limit, skip);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse divisions: {}", e)))
    }

    pub async fn list_questions(
        &self,
        limit: u32,
        skip: u32,
    ) -> Result<ApiResponse<QuestionResult>, IrlError> {
        let url = format!("{}/questions?limit={}&skip={}", BASE_URL, limit, skip);
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse questions: {}", e)))
    }

    pub async fn list_debates(
        &self,
        chamber: Option<&str>,
        date_start: Option<&str>,
        date_end: Option<&str>,
        limit: u32,
        skip: u32,
    ) -> Result<ApiResponse<DebateResult>, IrlError> {
        let mut url = format!("{}/debates?limit={}&skip={}", BASE_URL, limit, skip);
        if let Some(chamber) = chamber {
            url.push_str(&format!("&chamber={}", chamber));
        }
        if let Some(date_start) = date_start {
            url.push_str(&format!("&date_start={}", date_start));
        }
        if let Some(date_end) = date_end {
            url.push_str(&format!("&date_end={}", date_end));
        }
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse debates: {}", e)))
    }
}
