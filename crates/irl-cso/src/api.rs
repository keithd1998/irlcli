use std::time::Duration;

use irl_core::cache::Cache;
use irl_core::error::IrlError;
use irl_core::http::HttpClient;

use crate::models::*;

const TOC_URL: &str = "https://ws.cso.ie/public/api.restful/PxStat.Data.Cube_API.ReadCollection";
const DATASET_BASE_URL: &str =
    "https://ws.cso.ie/public/api.restful/PxStat.Data.Cube_API.ReadDataset";

const TOC_CACHE_TTL: Duration = Duration::from_secs(86400); // 24 hours
const DATA_CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

pub struct CsoApi {
    client: HttpClient,
    cache: Cache,
}

impl CsoApi {
    pub fn new(verbose: bool, quiet: bool, no_cache: bool) -> Result<Self, IrlError> {
        Ok(Self {
            client: HttpClient::new(verbose, quiet)?,
            cache: Cache::new(!no_cache),
        })
    }

    async fn get_cached(&self, url: &str, ttl: Duration) -> Result<String, IrlError> {
        if let Some(cached) = self.cache.get(url) {
            return Ok(cached);
        }
        let text = self.client.get_text(url).await?;
        let _ = self.cache.set(url, &text, ttl);
        Ok(text)
    }

    /// Fetch the full table of contents (catalogue) from PxStat
    pub async fn fetch_catalog(&self) -> Result<CollectionResponse, IrlError> {
        let text = self.get_cached(TOC_URL, TOC_CACHE_TTL).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse CSO catalogue: {}", e)))
    }

    /// Fetch a single dataset by table code
    pub async fn fetch_dataset(&self, table_code: &str) -> Result<DatasetResponse, IrlError> {
        let url = format!("{}/{}/JSON-stat/2.0/en", DATASET_BASE_URL, table_code);
        let text = self.get_cached(&url, DATA_CACHE_TTL).await?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse dataset {}: {}", table_code, e)))
    }

    /// Search the catalog by keyword (case-insensitive substring match on title and code)
    pub fn search_catalog(items: &[CollectionItem], query: &str) -> Vec<CatalogRow> {
        let query_lower = query.to_lowercase();
        let terms: Vec<&str> = query_lower.split_whitespace().collect();

        items
            .iter()
            .filter(|item| {
                let title = item.label.as_deref().unwrap_or("").to_lowercase();
                let code = item
                    .extension
                    .as_ref()
                    .and_then(|e| e.matrix.as_deref())
                    .unwrap_or("")
                    .to_lowercase();
                // All terms must match in either title or code
                terms
                    .iter()
                    .all(|term| title.contains(term) || code.contains(term))
            })
            .map(CatalogRow::from_item)
            .collect()
    }

    /// Convert all catalog items to display rows
    pub fn catalog_to_rows(items: &[CollectionItem]) -> Vec<CatalogRow> {
        items.iter().map(CatalogRow::from_item).collect()
    }

    /// Extract metadata from a dataset response for the `info` command
    pub fn extract_table_info(table_code: &str, dataset: &DatasetResponse) -> TableInfo {
        let dimensions = dataset
            .id
            .iter()
            .map(|dim_id| {
                let dim = dataset.dimension.get(dim_id);
                let label = dim
                    .and_then(|d| d.label.as_ref())
                    .cloned()
                    .unwrap_or_else(|| dim_id.clone());
                let keys = dim
                    .map(|d| d.category.index.ordered_keys())
                    .unwrap_or_default();
                let labels_map = dim.and_then(|d| d.category.label.as_ref());
                let sample: Vec<String> = keys
                    .iter()
                    .take(5)
                    .map(|k| {
                        labels_map
                            .and_then(|m| m.get(k))
                            .cloned()
                            .unwrap_or_else(|| k.clone())
                    })
                    .collect();
                let suffix = if keys.len() > 5 {
                    format!(", ... ({} total)", keys.len())
                } else {
                    String::new()
                };
                DimensionInfo {
                    name: dim_id.clone(),
                    label,
                    value_count: keys.len(),
                    sample_values: format!("{}{}", sample.join(", "), suffix),
                }
            })
            .collect();

        TableInfo {
            code: table_code.to_uppercase(),
            title: dataset.label.clone().unwrap_or_default(),
            updated: dataset.updated.clone().unwrap_or_default(),
            dimensions,
            notes: dataset.note.clone().unwrap_or_default(),
        }
    }
}

impl CatalogRow {
    pub fn from_item(item: &CollectionItem) -> Self {
        let code = item
            .extension
            .as_ref()
            .and_then(|e| e.matrix.as_deref())
            .unwrap_or("-")
            .to_string();
        let title = item.label.clone().unwrap_or_default();
        let title = irl_core::truncate_display(&title, 80);
        let updated = item
            .updated
            .as_deref()
            .unwrap_or("-")
            .split('T')
            .next()
            .unwrap_or("-")
            .to_string();
        CatalogRow {
            code,
            title,
            updated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_items() -> Vec<CollectionItem> {
        vec![
            CollectionItem {
                label: Some("Consumer Price Index".to_string()),
                href: None,
                updated: Some("2024-06-15T10:00:00Z".to_string()),
                dimension: None,
                extension: Some(CollectionExtension {
                    matrix: Some("CPM01".to_string()),
                }),
            },
            CollectionItem {
                label: Some("House Prices by County".to_string()),
                href: None,
                updated: Some("2024-05-01T08:00:00Z".to_string()),
                dimension: None,
                extension: Some(CollectionExtension {
                    matrix: Some("HPM09".to_string()),
                }),
            },
            CollectionItem {
                label: Some("Population and Migration Estimates".to_string()),
                href: None,
                updated: Some("2024-04-20T12:00:00Z".to_string()),
                dimension: None,
                extension: Some(CollectionExtension {
                    matrix: Some("PEA01".to_string()),
                }),
            },
        ]
    }

    #[test]
    fn test_search_catalog_single_term() {
        let items = make_test_items();
        let results = CsoApi::search_catalog(&items, "house");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "HPM09");
    }

    #[test]
    fn test_search_catalog_multiple_terms() {
        let items = make_test_items();
        let results = CsoApi::search_catalog(&items, "house prices");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "HPM09");
    }

    #[test]
    fn test_search_catalog_by_code() {
        let items = make_test_items();
        let results = CsoApi::search_catalog(&items, "CPM01");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Consumer Price Index");
    }

    #[test]
    fn test_search_catalog_case_insensitive() {
        let items = make_test_items();
        let results = CsoApi::search_catalog(&items, "POPULATION");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "PEA01");
    }

    #[test]
    fn test_search_catalog_no_match() {
        let items = make_test_items();
        let results = CsoApi::search_catalog(&items, "xyz_no_match");
        assert!(results.is_empty());
    }

    #[test]
    fn test_catalog_to_rows() {
        let items = make_test_items();
        let rows = CsoApi::catalog_to_rows(&items);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].code, "CPM01");
        assert_eq!(rows[0].updated, "2024-06-15");
    }

    #[test]
    fn test_catalog_row_truncates_long_title() {
        let item = CollectionItem {
            label: Some("A".repeat(100)),
            href: None,
            updated: None,
            dimension: None,
            extension: Some(CollectionExtension {
                matrix: Some("X01".to_string()),
            }),
        };
        let row = CatalogRow::from_item(&item);
        assert!(row.title.len() <= 83); // 77 + "..."
        assert!(row.title.ends_with("..."));
    }
}
