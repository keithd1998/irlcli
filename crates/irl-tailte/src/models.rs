use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- Tailte Éireann expected response structures --

#[derive(Debug, Deserialize, Serialize)]
pub struct ValuationSearchResponse {
    pub results: Option<Vec<ValuationResult>>,
    pub total: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValuationResult {
    pub property_number: Option<String>,
    pub address: Option<String>,
    pub uses: Option<String>,
    pub valuation: Option<String>,
    pub rating_authority: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PropertyValuation {
    pub property_number: Option<String>,
    pub address: Option<String>,
    pub uses: Option<String>,
    pub valuation: Option<String>,
    pub net_annual_value: Option<String>,
    pub rating_authority: Option<String>,
    pub category: Option<String>,
    pub floor_area: Option<String>,
    pub effective_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AreaResponse {
    pub properties: Option<Vec<ValuationResult>>,
    pub total: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoriesResponse {
    pub categories: Option<Vec<PropertyCategory>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PropertyCategory {
    pub code: Option<String>,
    pub description: Option<String>,
}

// -- Display row structs --

#[derive(Debug, Tabled, Serialize)]
pub struct ValuationRow {
    #[tabled(rename = "Property No.")]
    pub property_number: String,
    #[tabled(rename = "Address")]
    pub address: String,
    #[tabled(rename = "Uses")]
    pub uses: String,
    #[tabled(rename = "Valuation")]
    pub valuation: String,
    #[tabled(rename = "Authority")]
    pub rating_authority: String,
}

impl ValuationRow {
    pub fn from_result(result: &ValuationResult) -> Self {
        let address = result.address.clone().unwrap_or_default();
        Self {
            property_number: result.property_number.clone().unwrap_or_default(),
            address: irl_core::truncate_display(&address, 40),
            uses: result.uses.clone().unwrap_or_default(),
            valuation: result.valuation.clone().unwrap_or_default(),
            rating_authority: result.rating_authority.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct PropertyDetailRow {
    #[tabled(rename = "Field")]
    pub field: String,
    #[tabled(rename = "Value")]
    pub value: String,
}

impl PropertyDetailRow {
    pub fn from_valuation(v: &PropertyValuation) -> Vec<Self> {
        vec![
            Self {
                field: "Property Number".to_string(),
                value: v.property_number.clone().unwrap_or_default(),
            },
            Self {
                field: "Address".to_string(),
                value: v.address.clone().unwrap_or_default(),
            },
            Self {
                field: "Uses".to_string(),
                value: v.uses.clone().unwrap_or_default(),
            },
            Self {
                field: "Valuation".to_string(),
                value: v.valuation.clone().unwrap_or_default(),
            },
            Self {
                field: "Net Annual Value".to_string(),
                value: v.net_annual_value.clone().unwrap_or_default(),
            },
            Self {
                field: "Rating Authority".to_string(),
                value: v.rating_authority.clone().unwrap_or_default(),
            },
            Self {
                field: "Category".to_string(),
                value: v.category.clone().unwrap_or_default(),
            },
            Self {
                field: "Floor Area".to_string(),
                value: v.floor_area.clone().unwrap_or_default(),
            },
            Self {
                field: "Effective Date".to_string(),
                value: v.effective_date.clone().unwrap_or_default(),
            },
        ]
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct CategoryRow {
    #[tabled(rename = "Code")]
    pub code: String,
    #[tabled(rename = "Description")]
    pub description: String,
}

impl CategoryRow {
    pub fn from_category(cat: &PropertyCategory) -> Self {
        Self {
            code: cat.code.clone().unwrap_or_default(),
            description: cat.description.clone().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_valuation_search() {
        let json = r#"{
            "results": [{
                "property_number": "12345",
                "address": "1 Main Street, Dublin 2",
                "uses": "Office",
                "valuation": "€50,000",
                "rating_authority": "Dublin City Council",
                "category": "OFFICE"
            }],
            "total": 1
        }"#;

        let response: ValuationSearchResponse = serde_json::from_str(json).unwrap();
        let results = response.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].property_number.as_deref(), Some("12345"));
        assert_eq!(results[0].address.as_deref(), Some("1 Main Street, Dublin 2"));
    }

    #[test]
    fn test_deserialize_property_valuation() {
        let json = r#"{
            "property_number": "12345",
            "address": "1 Main Street, Dublin 2",
            "uses": "Office",
            "valuation": "€50,000",
            "net_annual_value": "€48,000",
            "rating_authority": "Dublin City Council",
            "category": "OFFICE",
            "floor_area": "250 sqm",
            "effective_date": "2024-01-01"
        }"#;

        let valuation: PropertyValuation = serde_json::from_str(json).unwrap();
        assert_eq!(valuation.property_number.as_deref(), Some("12345"));
        assert_eq!(valuation.net_annual_value.as_deref(), Some("€48,000"));
        assert_eq!(valuation.floor_area.as_deref(), Some("250 sqm"));
    }

    #[test]
    fn test_deserialize_categories() {
        let json = r#"{
            "categories": [
                {"code": "OFFICE", "description": "Office and Premises"},
                {"code": "RETAIL", "description": "Retail (Shop)"}
            ]
        }"#;

        let response: CategoriesResponse = serde_json::from_str(json).unwrap();
        let categories = response.categories.unwrap();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].code.as_deref(), Some("OFFICE"));
    }

    #[test]
    fn test_valuation_row_from_result() {
        let result = ValuationResult {
            property_number: Some("12345".to_string()),
            address: Some("1 Main Street".to_string()),
            uses: Some("Office".to_string()),
            valuation: Some("€50,000".to_string()),
            rating_authority: Some("Dublin CC".to_string()),
            category: Some("OFFICE".to_string()),
        };

        let row = ValuationRow::from_result(&result);
        assert_eq!(row.property_number, "12345");
        assert_eq!(row.address, "1 Main Street");
        assert_eq!(row.valuation, "€50,000");
    }

    #[test]
    fn test_valuation_row_long_address_truncated() {
        let result = ValuationResult {
            property_number: Some("12345".to_string()),
            address: Some("A very long address that exceeds forty characters limit for display".to_string()),
            uses: None,
            valuation: None,
            rating_authority: None,
            category: None,
        };

        let row = ValuationRow::from_result(&result);
        assert!(row.address.ends_with("..."));
        assert!(row.address.len() <= 43);
    }

    #[test]
    fn test_property_detail_rows() {
        let v = PropertyValuation {
            property_number: Some("12345".to_string()),
            address: Some("Dublin".to_string()),
            uses: Some("Office".to_string()),
            valuation: Some("€50,000".to_string()),
            net_annual_value: Some("€48,000".to_string()),
            rating_authority: Some("Dublin CC".to_string()),
            category: Some("OFFICE".to_string()),
            floor_area: Some("250 sqm".to_string()),
            effective_date: Some("2024-01-01".to_string()),
        };

        let rows = PropertyDetailRow::from_valuation(&v);
        assert_eq!(rows.len(), 9);
        assert_eq!(rows[0].field, "Property Number");
        assert_eq!(rows[0].value, "12345");
    }

    #[test]
    fn test_category_row_from_category() {
        let cat = PropertyCategory {
            code: Some("OFFICE".to_string()),
            description: Some("Office and Premises".to_string()),
        };

        let row = CategoryRow::from_category(&cat);
        assert_eq!(row.code, "OFFICE");
        assert_eq!(row.description, "Office and Premises");
    }

    #[test]
    fn test_valuation_row_missing_fields() {
        let result = ValuationResult {
            property_number: None,
            address: None,
            uses: None,
            valuation: None,
            rating_authority: None,
            category: None,
        };

        let row = ValuationRow::from_result(&result);
        assert_eq!(row.property_number, "");
        assert_eq!(row.address, "");
    }
}
