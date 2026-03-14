use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySale {
    pub date: String,
    pub address: String,
    pub county: String,
    pub eircode: String,
    pub price: f64,
    pub not_full_market_price: bool,
    pub vat_exclusive: bool,
    pub description: String,
    pub property_size: String,
}

#[derive(Debug, Tabled, Serialize)]
pub struct PropertyRow {
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Address")]
    pub address: String,
    #[tabled(rename = "County")]
    pub county: String,
    #[tabled(rename = "Price")]
    pub price: String,
    #[tabled(rename = "Description")]
    pub description: String,
    #[tabled(rename = "Size")]
    pub size: String,
}

impl PropertyRow {
    pub fn from_sale(sale: &PropertySale) -> Self {
        let address = irl_core::truncate_display(&sale.address, 40);

        Self {
            date: sale.date.clone(),
            address,
            county: sale.county.clone(),
            price: format_price(sale.price),
            description: sale.description.clone(),
            size: sale.property_size.clone(),
        }
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct StatsRow {
    #[tabled(rename = "Metric")]
    pub metric: String,
    #[tabled(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct PropertyStats {
    pub total_sales: u64,
    pub average_price: f64,
    pub median_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub total_value: f64,
}

impl PropertyStats {
    pub fn calculate(prices: &[f64]) -> Self {
        if prices.is_empty() {
            return Self {
                total_sales: 0,
                average_price: 0.0,
                median_price: 0.0,
                min_price: 0.0,
                max_price: 0.0,
                total_value: 0.0,
            };
        }

        let mut sorted = prices.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let total = sorted.len() as u64;
        let sum: f64 = sorted.iter().sum();
        let average = sum / total as f64;

        let median = if sorted.len().is_multiple_of(2) {
            let mid = sorted.len() / 2;
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        Self {
            total_sales: total,
            average_price: average,
            median_price: median,
            min_price: sorted[0],
            max_price: sorted[sorted.len() - 1],
            total_value: sum,
        }
    }

    pub fn to_rows(&self) -> Vec<StatsRow> {
        vec![
            StatsRow {
                metric: "Total Sales".to_string(),
                value: format!("{}", self.total_sales),
            },
            StatsRow {
                metric: "Average Price".to_string(),
                value: format_price(self.average_price),
            },
            StatsRow {
                metric: "Median Price".to_string(),
                value: format_price(self.median_price),
            },
            StatsRow {
                metric: "Min Price".to_string(),
                value: format_price(self.min_price),
            },
            StatsRow {
                metric: "Max Price".to_string(),
                value: format_price(self.max_price),
            },
            StatsRow {
                metric: "Total Value".to_string(),
                value: format_price(self.total_value),
            },
        ]
    }
}

pub fn format_price(price: f64) -> String {
    // Format with thousands separators
    let whole = price as u64;
    let s = whole.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    let formatted: String = result.chars().rev().collect();
    format!("\u{20ac}{}", formatted)
}

/// Parse a price string from PSRA CSV format.
/// Handles formats like "€123,456.00" or "123456.00" or "€123,456"
pub fn parse_price(s: &str) -> f64 {
    let cleaned: String = s.replace(['\u{20ac}', ',', ' '], "").trim().to_string();
    cleaned.parse::<f64>().unwrap_or(0.0)
}

/// Parse a boolean field from PSRA CSV ("Yes"/"No")
pub fn parse_yes_no(s: &str) -> bool {
    s.trim().eq_ignore_ascii_case("yes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_sale_serialize() {
        let sale = PropertySale {
            date: "2024-01-15".to_string(),
            address: "1 Main Street, Dublin".to_string(),
            county: "Dublin".to_string(),
            eircode: "D01 AB12".to_string(),
            price: 350000.0,
            not_full_market_price: false,
            vat_exclusive: false,
            description: "Second-Hand Dwelling house /Apartment".to_string(),
            property_size: "greater than or equal to 38 sq metres and less than 125 sq metres"
                .to_string(),
        };

        let json = serde_json::to_string(&sale).unwrap();
        let deserialized: PropertySale = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.address, "1 Main Street, Dublin");
        assert_eq!(deserialized.price, 350000.0);
        assert!(!deserialized.not_full_market_price);
    }

    #[test]
    fn test_property_row_from_sale() {
        let sale = PropertySale {
            date: "2024-01-15".to_string(),
            address: "1 Main Street, Dublin".to_string(),
            county: "Dublin".to_string(),
            eircode: "D01 AB12".to_string(),
            price: 350000.0,
            not_full_market_price: false,
            vat_exclusive: false,
            description: "New Dwelling".to_string(),
            property_size: "38-125 sq m".to_string(),
        };

        let row = PropertyRow::from_sale(&sale);
        assert_eq!(row.county, "Dublin");
        assert_eq!(row.price, "\u{20ac}350,000");
    }

    #[test]
    fn test_stats_calculation() {
        let prices = vec![100000.0, 200000.0, 300000.0, 400000.0, 500000.0];
        let stats = PropertyStats::calculate(&prices);
        assert_eq!(stats.total_sales, 5);
        assert!((stats.average_price - 300000.0).abs() < 0.01);
        assert!((stats.median_price - 300000.0).abs() < 0.01);
        assert!((stats.min_price - 100000.0).abs() < 0.01);
        assert!((stats.max_price - 500000.0).abs() < 0.01);
        assert!((stats.total_value - 1500000.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_even_count() {
        let prices = vec![100000.0, 200000.0, 300000.0, 400000.0];
        let stats = PropertyStats::calculate(&prices);
        assert_eq!(stats.total_sales, 4);
        // Median of [100k, 200k, 300k, 400k] = (200k + 300k) / 2 = 250k
        assert!((stats.median_price - 250000.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_empty() {
        let prices: Vec<f64> = vec![];
        let stats = PropertyStats::calculate(&prices);
        assert_eq!(stats.total_sales, 0);
        assert!((stats.average_price - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_single() {
        let prices = vec![250000.0];
        let stats = PropertyStats::calculate(&prices);
        assert_eq!(stats.total_sales, 1);
        assert!((stats.median_price - 250000.0).abs() < 0.01);
        assert!((stats.average_price - 250000.0).abs() < 0.01);
    }

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(350000.0), "\u{20ac}350,000");
        assert_eq!(format_price(1250000.0), "\u{20ac}1,250,000");
        assert_eq!(format_price(50000.0), "\u{20ac}50,000");
        assert_eq!(format_price(999.0), "\u{20ac}999");
    }

    #[test]
    fn test_parse_price() {
        assert!((parse_price("€350,000.00") - 350000.0).abs() < 0.01);
        assert!((parse_price("350000.00") - 350000.0).abs() < 0.01);
        assert!((parse_price("€1,250,000") - 1250000.0).abs() < 0.01);
        assert!((parse_price("invalid") - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_yes_no() {
        assert!(parse_yes_no("Yes"));
        assert!(parse_yes_no("YES"));
        assert!(parse_yes_no("yes"));
        assert!(!parse_yes_no("No"));
        assert!(!parse_yes_no(""));
    }

    #[test]
    fn test_stats_to_rows() {
        let prices = vec![100000.0, 200000.0, 300000.0];
        let stats = PropertyStats::calculate(&prices);
        let rows = stats.to_rows();
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[0].metric, "Total Sales");
        assert_eq!(rows[0].value, "3");
    }

    #[test]
    fn test_property_row_long_address() {
        let sale = PropertySale {
            date: "2024-01-15".to_string(),
            address:
                "A very long address that exceeds forty characters in total length for testing"
                    .to_string(),
            county: "Dublin".to_string(),
            eircode: "".to_string(),
            price: 100000.0,
            not_full_market_price: false,
            vat_exclusive: false,
            description: "".to_string(),
            property_size: "".to_string(),
        };

        let row = PropertyRow::from_sale(&sale);
        assert!(row.address.ends_with("..."));
        assert!(row.address.len() <= 43); // 37 + "..."
    }
}
