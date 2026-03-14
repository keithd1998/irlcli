use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::QueryResponse;

// ArcGIS REST API base — FeatureServer URLs need to be configured
// once the correct service IDs are discovered on services.arcgis.com
const BASE_URL: &str = "https://services-eu1.arcgis.com/LtKMADpQ1GRHcefh/ArcGIS/rest/services";
const CACHE_TTL: Duration = Duration::from_secs(86400); // 24 hours

pub struct GeoApi {
    client: HttpClient,
    cache: Cache,
}

impl GeoApi {
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

    pub async fn query_boundaries(
        &self,
        boundary_type: &str,
    ) -> Result<QueryResponse, IrlError> {
        let service_name = match boundary_type.to_lowercase().as_str() {
            "county" | "counties" => "Counties_National_Statutory_Boundary_2019",
            "province" | "provinces" => "Provinces_National_Statutory_Boundary_2019",
            "electoral" | "ed" => "Electoral_Divisions_National_Statutory_Boundary_2019",
            _ => boundary_type,
        };
        let url = format!(
            "{}/{}/FeatureServer/0/query?where=1%3D1&outFields=*&f=json",
            BASE_URL, service_name
        );
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse boundary data: {}", e)))
    }

    pub async fn query_point(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<QueryResponse, IrlError> {
        // Query county boundaries containing the given point
        let url = format!(
            "{}/Counties_National_Statutory_Boundary_2019/FeatureServer/0/query\
             ?geometry={},{}&geometryType=esriGeometryPoint\
             &spatialRel=esriSpatialRelIntersects\
             &inSR=4326&outFields=*&f=json",
            BASE_URL, lon, lat
        );
        let text = self.get_cached(&url).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse point query: {}", e)))
    }

    pub async fn fetch_dataset(
        &self,
        dataset_id: &str,
        format: &str,
    ) -> Result<String, IrlError> {
        let f = match format.to_lowercase().as_str() {
            "geojson" => "geojson",
            "json" => "json",
            _ => "json",
        };
        let url = format!(
            "{}/{}/FeatureServer/0/query?where=1%3D1&outFields=*&f={}",
            BASE_URL, dataset_id, f
        );
        self.get_cached(&url).await
    }
}
