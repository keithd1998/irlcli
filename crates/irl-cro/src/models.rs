use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- CRO API response structures --

#[derive(Debug, Deserialize)]
pub struct CompanySearchResponse {
    pub companies: Option<Vec<CompanyResult>>,
    pub total: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CompanyResult {
    pub company_name: Option<String>,
    pub company_number: Option<String>,
    pub company_status: Option<String>,
    pub company_type: Option<String>,
    pub date_registered: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompanyDetail {
    pub company_name: Option<String>,
    pub company_number: Option<String>,
    pub company_status: Option<String>,
    pub company_type: Option<String>,
    pub date_registered: Option<String>,
    pub registered_address: Option<String>,
    pub directors: Option<Vec<Director>>,
    pub secretary: Option<String>,
    pub activity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Director {
    pub name: Option<String>,
    pub appointed: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FilingsResponse {
    pub filings: Option<Vec<Filing>>,
    pub total: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Filing {
    pub filing_type: Option<String>,
    pub filing_date: Option<String>,
    pub effective_date: Option<String>,
    pub description: Option<String>,
    pub barcode: Option<String>,
}

// -- Display row structs --

#[derive(Debug, Tabled, Serialize)]
pub struct CompanyRow {
    #[tabled(rename = "Number")]
    pub number: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Type")]
    pub company_type: String,
    #[tabled(rename = "Registered")]
    pub date_registered: String,
}

impl CompanyRow {
    pub fn from_result(result: &CompanyResult) -> Self {
        Self {
            number: result.company_number.clone().unwrap_or_default(),
            name: result.company_name.clone().unwrap_or_default(),
            status: result.company_status.clone().unwrap_or_default(),
            company_type: result.company_type.clone().unwrap_or_default(),
            date_registered: result.date_registered.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct CompanyDetailRow {
    #[tabled(rename = "Field")]
    pub field: String,
    #[tabled(rename = "Value")]
    pub value: String,
}

impl CompanyDetailRow {
    pub fn from_detail(detail: &CompanyDetail) -> Vec<Self> {
        let mut rows = vec![
            Self {
                field: "Company Number".to_string(),
                value: detail.company_number.clone().unwrap_or_default(),
            },
            Self {
                field: "Company Name".to_string(),
                value: detail.company_name.clone().unwrap_or_default(),
            },
            Self {
                field: "Status".to_string(),
                value: detail.company_status.clone().unwrap_or_default(),
            },
            Self {
                field: "Type".to_string(),
                value: detail.company_type.clone().unwrap_or_default(),
            },
            Self {
                field: "Date Registered".to_string(),
                value: detail.date_registered.clone().unwrap_or_default(),
            },
            Self {
                field: "Registered Address".to_string(),
                value: detail.registered_address.clone().unwrap_or_default(),
            },
            Self {
                field: "Activity".to_string(),
                value: detail.activity.clone().unwrap_or_default(),
            },
            Self {
                field: "Secretary".to_string(),
                value: detail.secretary.clone().unwrap_or_default(),
            },
        ];

        if let Some(directors) = &detail.directors {
            for (i, director) in directors.iter().enumerate() {
                rows.push(Self {
                    field: format!("Director {}", i + 1),
                    value: director.name.clone().unwrap_or_default(),
                });
            }
        }

        rows
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct FilingRow {
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Type")]
    pub filing_type: String,
    #[tabled(rename = "Description")]
    pub description: String,
    #[tabled(rename = "Barcode")]
    pub barcode: String,
}

impl FilingRow {
    pub fn from_filing(filing: &Filing) -> Self {
        let description = filing.description.clone().unwrap_or_default();
        Self {
            date: filing.filing_date.clone().unwrap_or_default(),
            filing_type: filing.filing_type.clone().unwrap_or_default(),
            description: if description.len() > 60 {
                format!("{}...", &description[..57])
            } else {
                description
            },
            barcode: filing.barcode.clone().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_company_search() {
        let json = r#"{
            "companies": [{
                "company_name": "Test Ltd",
                "company_number": "123456",
                "company_status": "Normal",
                "company_type": "Private Company Limited by Shares",
                "date_registered": "2020-01-15",
                "address": "1 Main Street, Dublin"
            }],
            "total": 1
        }"#;

        let response: CompanySearchResponse = serde_json::from_str(json).unwrap();
        let companies = response.companies.unwrap();
        assert_eq!(companies.len(), 1);
        assert_eq!(companies[0].company_name.as_deref(), Some("Test Ltd"));
        assert_eq!(companies[0].company_number.as_deref(), Some("123456"));
    }

    #[test]
    fn test_deserialize_company_detail() {
        let json = r#"{
            "company_name": "Test Ltd",
            "company_number": "123456",
            "company_status": "Normal",
            "company_type": "Private Company Limited by Shares",
            "date_registered": "2020-01-15",
            "registered_address": "1 Main Street, Dublin",
            "directors": [
                { "name": "John Smith", "appointed": "2020-01-15" },
                { "name": "Jane Doe", "appointed": "2021-06-01" }
            ],
            "secretary": "Mary Jones",
            "activity": "Software Development"
        }"#;

        let detail: CompanyDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.company_name.as_deref(), Some("Test Ltd"));
        assert_eq!(detail.directors.as_ref().unwrap().len(), 2);
        assert_eq!(
            detail.directors.as_ref().unwrap()[0].name.as_deref(),
            Some("John Smith")
        );
    }

    #[test]
    fn test_deserialize_filings() {
        let json = r#"{
            "filings": [{
                "filing_type": "B1",
                "filing_date": "2024-06-15",
                "effective_date": "2024-06-15",
                "description": "Annual Return",
                "barcode": "ABC123"
            }],
            "total": 1
        }"#;

        let response: FilingsResponse = serde_json::from_str(json).unwrap();
        let filings = response.filings.unwrap();
        assert_eq!(filings.len(), 1);
        assert_eq!(filings[0].filing_type.as_deref(), Some("B1"));
        assert_eq!(filings[0].description.as_deref(), Some("Annual Return"));
    }

    #[test]
    fn test_company_row_from_result() {
        let result = CompanyResult {
            company_name: Some("Test Ltd".to_string()),
            company_number: Some("123456".to_string()),
            company_status: Some("Normal".to_string()),
            company_type: Some("LTD".to_string()),
            date_registered: Some("2020-01-15".to_string()),
            address: Some("Dublin".to_string()),
        };

        let row = CompanyRow::from_result(&result);
        assert_eq!(row.name, "Test Ltd");
        assert_eq!(row.number, "123456");
        assert_eq!(row.status, "Normal");
    }

    #[test]
    fn test_company_detail_rows() {
        let detail = CompanyDetail {
            company_name: Some("Test Ltd".to_string()),
            company_number: Some("123456".to_string()),
            company_status: Some("Normal".to_string()),
            company_type: Some("LTD".to_string()),
            date_registered: Some("2020-01-15".to_string()),
            registered_address: Some("Dublin".to_string()),
            directors: Some(vec![Director {
                name: Some("John Smith".to_string()),
                appointed: Some("2020-01-15".to_string()),
            }]),
            secretary: Some("Mary Jones".to_string()),
            activity: Some("Software".to_string()),
        };

        let rows = CompanyDetailRow::from_detail(&detail);
        assert_eq!(rows.len(), 9); // 8 standard fields + 1 director
        assert_eq!(rows[0].field, "Company Number");
        assert_eq!(rows[0].value, "123456");
    }

    #[test]
    fn test_filing_row_from_filing() {
        let filing = Filing {
            filing_type: Some("B1".to_string()),
            filing_date: Some("2024-06-15".to_string()),
            effective_date: None,
            description: Some("Annual Return".to_string()),
            barcode: Some("ABC123".to_string()),
        };

        let row = FilingRow::from_filing(&filing);
        assert_eq!(row.filing_type, "B1");
        assert_eq!(row.date, "2024-06-15");
        assert_eq!(row.description, "Annual Return");
    }

    #[test]
    fn test_company_row_missing_fields() {
        let result = CompanyResult {
            company_name: None,
            company_number: None,
            company_status: None,
            company_type: None,
            date_registered: None,
            address: None,
        };

        let row = CompanyRow::from_result(&result);
        assert_eq!(row.name, "");
        assert_eq!(row.number, "");
    }
}
