use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- JSON-stat 2.0 Collection (Table of Contents) --

/// The top-level response from ReadCollection is a JSON-stat collection.
/// Each item in `link.item` represents a table in the CSO PxStat catalogue.
#[derive(Debug, Deserialize)]
pub struct CollectionResponse {
    pub link: CollectionLink,
}

#[derive(Debug, Deserialize)]
pub struct CollectionLink {
    pub item: Vec<CollectionItem>,
}

#[derive(Debug, Deserialize)]
pub struct CollectionItem {
    /// Human-readable table title
    pub label: Option<String>,
    /// Link to the full dataset
    pub href: Option<String>,
    /// Last updated timestamp (ISO 8601)
    pub updated: Option<String>,
    /// Dimension metadata for this table
    pub dimension: Option<HashMap<String, serde_json::Value>>,
    /// Extension block containing the table code (matrix)
    pub extension: Option<CollectionExtension>,
}

#[derive(Debug, Deserialize)]
pub struct CollectionExtension {
    /// The table code, e.g. "B0101", "CPM01"
    pub matrix: Option<String>,
}

// -- JSON-stat 2.0 Dataset Response --

/// Full dataset response from ReadDataset
#[derive(Debug, Deserialize)]
pub struct DatasetResponse {
    /// List of dimension IDs in order, e.g. ["STATISTIC", "TLIST(M1)", "C01779V03424"]
    pub id: Vec<String>,
    /// Count of categories per dimension, same order as `id`
    pub size: Vec<usize>,
    /// Flat array of all data values (product of sizes = total count)
    /// Values can be numbers or null
    pub value: Vec<Option<f64>>,
    /// Dimension definitions keyed by dimension ID
    pub dimension: HashMap<String, DimensionDef>,
    /// Dataset label (title)
    pub label: Option<String>,
    /// Last updated timestamp
    pub updated: Option<String>,
    /// Extension with extra metadata
    pub extension: Option<DatasetExtension>,
    /// Notes about the dataset
    pub note: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DimensionDef {
    /// Human-readable dimension label
    pub label: Option<String>,
    /// Category info
    pub category: CategoryDef,
}

#[derive(Debug, Deserialize)]
pub struct CategoryDef {
    /// Maps category code to its position index.
    /// Can be a map of string->int or an ordered list of strings.
    #[serde(default)]
    pub index: CategoryIndex,
    /// Maps category code to human-readable label
    pub label: Option<HashMap<String, String>>,
}

/// Category index can be either an ordered list or a map of code->position
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CategoryIndex {
    List(Vec<String>),
    Map(HashMap<String, usize>),
}

impl Default for CategoryIndex {
    fn default() -> Self {
        CategoryIndex::List(Vec::new())
    }
}

impl CategoryIndex {
    /// Return ordered list of category codes
    pub fn ordered_keys(&self) -> Vec<String> {
        match self {
            CategoryIndex::List(list) => list.clone(),
            CategoryIndex::Map(map) => {
                let mut entries: Vec<(&String, &usize)> = map.iter().collect();
                entries.sort_by_key(|(_, idx)| **idx);
                entries.into_iter().map(|(k, _)| k.clone()).collect()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DatasetExtension {
    pub matrix: Option<String>,
}

// -- Display row structs --

#[derive(Debug, Tabled, Serialize, Clone)]
pub struct CatalogRow {
    #[tabled(rename = "Code")]
    pub code: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Last Updated")]
    pub updated: String,
}

#[derive(Debug, Tabled, Serialize, Clone)]
pub struct DataRow {
    #[tabled(rename = "Statistic")]
    pub statistic: String,
    #[tabled(rename = "Period")]
    pub period: String,
    #[tabled(rename = "Category")]
    pub category: String,
    #[tabled(rename = "Value")]
    pub value: String,
}

/// Metadata info for a single table, used by `cso info`
#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub code: String,
    pub title: String,
    pub updated: String,
    pub dimensions: Vec<DimensionInfo>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Tabled)]
pub struct DimensionInfo {
    #[tabled(rename = "Dimension")]
    pub name: String,
    #[tabled(rename = "Label")]
    pub label: String,
    #[tabled(rename = "Values")]
    pub value_count: usize,
    #[tabled(rename = "Sample Values")]
    pub sample_values: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_index_list() {
        let idx = CategoryIndex::List(vec!["a".into(), "b".into(), "c".into()]);
        assert_eq!(idx.ordered_keys(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_category_index_map() {
        let mut map = HashMap::new();
        map.insert("b".into(), 1);
        map.insert("a".into(), 0);
        map.insert("c".into(), 2);
        let idx = CategoryIndex::Map(map);
        assert_eq!(idx.ordered_keys(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_deserialize_collection_item() {
        let json = r#"{
            "label": "Population by County",
            "href": "https://ws.cso.ie/public/api.restful/PxStat.Data.Cube_API.ReadDataset/B0101/JSON-stat/2.0/en",
            "updated": "2024-01-15T10:00:00Z",
            "extension": { "matrix": "B0101" }
        }"#;
        let item: CollectionItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.label.unwrap(), "Population by County");
        assert_eq!(item.extension.unwrap().matrix.unwrap(), "B0101");
    }

    #[test]
    fn test_deserialize_dataset_response() {
        let json = r#"{
            "id": ["STATISTIC", "TLIST(A1)"],
            "size": [2, 3],
            "value": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "dimension": {
                "STATISTIC": {
                    "label": "Statistic",
                    "category": {
                        "index": ["S1", "S2"],
                        "label": { "S1": "Population", "S2": "Density" }
                    }
                },
                "TLIST(A1)": {
                    "label": "Year",
                    "category": {
                        "index": {"2020": 0, "2021": 1, "2022": 2},
                        "label": { "2020": "2020", "2021": "2021", "2022": "2022" }
                    }
                }
            },
            "label": "Test Dataset",
            "updated": "2024-01-01"
        }"#;
        let ds: DatasetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(ds.id.len(), 2);
        assert_eq!(ds.size, vec![2, 3]);
        assert_eq!(ds.value.len(), 6);
        assert_eq!(ds.label.unwrap(), "Test Dataset");
    }
}
