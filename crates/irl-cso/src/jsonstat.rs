use std::collections::HashMap;

use crate::models::{DataRow, DatasetResponse};

/// Filters that can be applied when unpacking JSON-stat data
#[derive(Default)]
pub struct UnpackOptions {
    /// Dimension filters: dimension_label -> set of allowed value labels
    pub filters: HashMap<String, Vec<String>>,
    /// If set, only include the last N time periods
    pub last_n: Option<u32>,
}

impl UnpackOptions {
    /// Parse "--dimension" args like "Year=2024" into filters
    pub fn with_dimension_filters(mut self, dimension_args: &[String]) -> Self {
        for arg in dimension_args {
            if let Some((key, value)) = arg.split_once('=') {
                self.filters
                    .entry(key.to_string())
                    .or_default()
                    .push(value.to_string());
            }
        }
        self
    }

    pub fn with_last_n(mut self, last: Option<u32>) -> Self {
        self.last_n = last;
        self
    }
}

/// Unpack a JSON-stat 2.0 dataset into flat DataRow records.
///
/// The JSON-stat format stores values in a flat array indexed by the product
/// of dimension sizes. We iterate over all combinations of dimension indices,
/// compute the flat offset, and build a row for each combination.
pub fn unpack_dataset(dataset: &DatasetResponse, options: &UnpackOptions) -> Vec<DataRow> {
    let num_dims = dataset.id.len();
    if num_dims == 0 || dataset.value.is_empty() {
        return Vec::new();
    }

    // Build ordered category keys for each dimension
    let dim_keys: Vec<Vec<String>> = dataset
        .id
        .iter()
        .map(|dim_id| {
            dataset
                .dimension
                .get(dim_id)
                .map(|d| d.category.index.ordered_keys())
                .unwrap_or_default()
        })
        .collect();

    // Build label lookup for each dimension
    let dim_labels: Vec<HashMap<&str, &str>> = dataset
        .id
        .iter()
        .map(|dim_id| {
            dataset
                .dimension
                .get(dim_id)
                .and_then(|d| d.category.label.as_ref())
                .map(|labels| {
                    labels
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();

    // Build human-readable dimension labels
    let dim_display_labels: Vec<String> = dataset
        .id
        .iter()
        .map(|dim_id| {
            dataset
                .dimension
                .get(dim_id)
                .and_then(|d| d.label.as_ref())
                .cloned()
                .unwrap_or_else(|| dim_id.clone())
        })
        .collect();

    // Identify the time dimension index (contains "TLIST" in the id)
    let time_dim_idx = dataset.id.iter().position(|id| id.contains("TLIST"));

    // If --last N is specified, compute which time period keys to keep
    let allowed_time_keys: Option<Vec<String>> =
        if let (Some(last_n), Some(t_idx)) = (options.last_n, time_dim_idx) {
            let keys = &dim_keys[t_idx];
            let n = last_n as usize;
            if keys.len() > n {
                Some(keys[keys.len() - n..].to_vec())
            } else {
                None // all keys fit, no filtering needed
            }
        } else {
            None
        };

    // Pre-compute filter lookups: for each dimension, which category codes are allowed?
    let dim_filter_codes: Vec<Option<Vec<&str>>> = dataset
        .id
        .iter()
        .enumerate()
        .map(|(i, _dim_id)| {
            let display_label = &dim_display_labels[i];
            if let Some(filter_values) = options.filters.get(display_label) {
                // Find category codes whose labels match the filter values
                let codes: Vec<&str> = dim_keys[i]
                    .iter()
                    .filter(|code| {
                        let label = dim_labels[i]
                            .get(code.as_str())
                            .copied()
                            .unwrap_or(code.as_str());
                        filter_values
                            .iter()
                            .any(|fv| label.eq_ignore_ascii_case(fv) || label.contains(fv.as_str()))
                    })
                    .map(|s| s.as_str())
                    .collect();
                if codes.is_empty() {
                    None // no matches — don't filter (show all)
                } else {
                    Some(codes)
                }
            } else {
                None
            }
        })
        .collect();

    // Compute strides for flat index calculation
    // stride[i] = product of sizes[i+1..num_dims]
    let mut strides = vec![1usize; num_dims];
    for i in (0..num_dims - 1).rev() {
        strides[i] = strides[i + 1] * dataset.size[i + 1];
    }

    let mut rows = Vec::new();

    // Iterate over all combinations using a counter array
    let mut indices = vec![0usize; num_dims];

    loop {
        // Check filters
        let mut skip = false;
        for d in 0..num_dims {
            let code = &dim_keys[d][indices[d]];

            // Check dimension value filter
            if let Some(ref allowed_codes) = dim_filter_codes[d] {
                if !allowed_codes.contains(&code.as_str()) {
                    skip = true;
                    break;
                }
            }

            // Check --last N time filter
            if let Some(ref allowed_times) = allowed_time_keys {
                if Some(d) == time_dim_idx && !allowed_times.contains(code) {
                    skip = true;
                    break;
                }
            }
        }

        if !skip {
            // Compute flat index
            let flat_idx: usize = indices
                .iter()
                .enumerate()
                .map(|(d, &idx)| idx * strides[d])
                .sum();

            let value = dataset
                .value
                .get(flat_idx)
                .and_then(|v| *v)
                .map(|v| {
                    if v == v.floor() {
                        format!("{:.0}", v)
                    } else {
                        format!("{:.2}", v)
                    }
                })
                .unwrap_or_else(|| "..".to_string());

            // Build row fields. We use a standard layout:
            // - First dimension typically = statistic
            // - Time dimension = period
            // - Remaining dimensions = category (joined)
            let mut statistic = String::new();
            let mut period = String::new();
            let mut categories = Vec::new();

            for d in 0..num_dims {
                let code = &dim_keys[d][indices[d]];
                let label = dim_labels[d]
                    .get(code.as_str())
                    .copied()
                    .unwrap_or(code.as_str());

                if Some(d) == time_dim_idx {
                    period = label.to_string();
                } else if d == 0 {
                    statistic = label.to_string();
                } else {
                    categories.push(label.to_string());
                }
            }

            let category = if categories.is_empty() {
                "-".to_string()
            } else {
                categories.join(" | ")
            };

            rows.push(DataRow {
                statistic,
                period,
                category,
                value,
            });
        }

        // Increment counter (odometer style, rightmost dimension first)
        let mut carry = true;
        for d in (0..num_dims).rev() {
            if carry {
                indices[d] += 1;
                if indices[d] >= dim_keys[d].len() {
                    indices[d] = 0;
                } else {
                    carry = false;
                }
            }
        }
        if carry {
            break; // all combinations exhausted
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn make_test_dataset() -> DatasetResponse {
        let json = r#"{
            "id": ["STATISTIC", "TLIST(A1)", "C01"],
            "size": [2, 3, 2],
            "value": [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120],
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
                        "index": ["2020", "2021", "2022"],
                        "label": { "2020": "2020", "2021": "2021", "2022": "2022" }
                    }
                },
                "C01": {
                    "label": "County",
                    "category": {
                        "index": ["C1", "C2"],
                        "label": { "C1": "Dublin", "C2": "Cork" }
                    }
                }
            },
            "label": "Test Dataset",
            "updated": "2024-01-01"
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_unpack_all_rows() {
        let ds = make_test_dataset();
        let opts = UnpackOptions::default();
        let rows = unpack_dataset(&ds, &opts);
        // 2 stats * 3 years * 2 counties = 12 rows
        assert_eq!(rows.len(), 12);
    }

    #[test]
    fn test_unpack_values_correct() {
        let ds = make_test_dataset();
        let opts = UnpackOptions::default();
        let rows = unpack_dataset(&ds, &opts);
        // First row: S1, 2020, C1 => value[0] = 10
        assert_eq!(rows[0].statistic, "Population");
        assert_eq!(rows[0].period, "2020");
        assert_eq!(rows[0].category, "Dublin");
        assert_eq!(rows[0].value, "10");
        // Last row: S2, 2022, C2 => value[11] = 120
        assert_eq!(rows[11].statistic, "Density");
        assert_eq!(rows[11].period, "2022");
        assert_eq!(rows[11].category, "Cork");
        assert_eq!(rows[11].value, "120");
    }

    #[test]
    fn test_unpack_with_dimension_filter() {
        let ds = make_test_dataset();
        let filter_args = vec!["County=Dublin".to_string()];
        let opts = UnpackOptions::default().with_dimension_filters(&filter_args);
        let rows = unpack_dataset(&ds, &opts);
        // Only Dublin: 2 stats * 3 years = 6
        assert_eq!(rows.len(), 6);
        for row in &rows {
            assert_eq!(row.category, "Dublin");
        }
    }

    #[test]
    fn test_unpack_with_last_n() {
        let ds = make_test_dataset();
        let opts = UnpackOptions::default().with_last_n(Some(2));
        let rows = unpack_dataset(&ds, &opts);
        // Last 2 years (2021, 2022) * 2 stats * 2 counties = 8
        assert_eq!(rows.len(), 8);
        for row in &rows {
            assert!(row.period == "2021" || row.period == "2022");
        }
    }

    #[test]
    fn test_unpack_combined_filters() {
        let ds = make_test_dataset();
        let filter_args = vec!["County=Cork".to_string()];
        let opts = UnpackOptions::default()
            .with_dimension_filters(&filter_args)
            .with_last_n(Some(1));
        let rows = unpack_dataset(&ds, &opts);
        // Cork only, last 1 year (2022), 2 stats = 2
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].period, "2022");
        assert_eq!(rows[0].category, "Cork");
    }

    #[test]
    fn test_unpack_empty_dataset() {
        let json = r#"{
            "id": [],
            "size": [],
            "value": [],
            "dimension": {}
        }"#;
        let ds: DatasetResponse = serde_json::from_str(json).unwrap();
        let rows = unpack_dataset(&ds, &UnpackOptions::default());
        assert!(rows.is_empty());
    }

    #[test]
    fn test_unpack_null_values() {
        let json = r#"{
            "id": ["STATISTIC", "TLIST(A1)"],
            "size": [1, 2],
            "value": [42.0, null],
            "dimension": {
                "STATISTIC": {
                    "label": "Statistic",
                    "category": {
                        "index": ["S1"],
                        "label": { "S1": "Count" }
                    }
                },
                "TLIST(A1)": {
                    "label": "Year",
                    "category": {
                        "index": ["2020", "2021"],
                        "label": { "2020": "2020", "2021": "2021" }
                    }
                }
            }
        }"#;
        let ds: DatasetResponse = serde_json::from_str(json).unwrap();
        let rows = unpack_dataset(&ds, &UnpackOptions::default());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].value, "42");
        assert_eq!(rows[1].value, "..");
    }
}
