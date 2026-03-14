use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- OPW Water Levels GeoJSON structures --

#[derive(Debug, Deserialize)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub collection_type: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub id: Option<u64>,
    pub properties: Properties,
    pub geometry: Geometry,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    pub name: Option<String>,
    #[serde(rename = "ref")]
    pub station_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub coordinates: Vec<f64>,
}

// -- Display row structs --

#[derive(Debug, Tabled, Serialize)]
pub struct StationRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Ref")]
    pub station_ref: String,
    #[tabled(rename = "Latitude")]
    pub lat: String,
    #[tabled(rename = "Longitude")]
    pub lon: String,
}

impl StationRow {
    pub fn from_feature(feature: &Feature) -> Self {
        let (lon, lat) = if feature.geometry.coordinates.len() >= 2 {
            (feature.geometry.coordinates[0], feature.geometry.coordinates[1])
        } else {
            (0.0, 0.0)
        };

        Self {
            name: feature.properties.name.clone().unwrap_or_default(),
            station_ref: feature.properties.station_ref.clone().unwrap_or_default(),
            lat: format!("{:.6}", lat),
            lon: format!("{:.6}", lon),
        }
    }
}

/// Search stations by query string (case-insensitive substring match on name)
pub fn search_stations(features: &[Feature], query: &str) -> Vec<StationRow> {
    let query_lower = query.to_lowercase();
    features
        .iter()
        .filter(|f| {
            f.properties
                .name
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(&query_lower)
        })
        .map(StationRow::from_feature)
        .collect()
}

/// Filter stations by county (substring match on station name)
pub fn filter_by_county(features: &[Feature], county: &str) -> Vec<StationRow> {
    search_stations(features, county)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_geojson() -> &'static str {
        r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "id": 3,
                    "properties": {"name": "Sandy Mills", "ref": "0000001041"},
                    "geometry": {"type": "Point", "coordinates": [-7.575758, 54.838318]}
                },
                {
                    "type": "Feature",
                    "id": 5,
                    "properties": {"name": "Ballybofey", "ref": "0000001042"},
                    "geometry": {"type": "Point", "coordinates": [-7.790123, 54.799456]}
                },
                {
                    "type": "Feature",
                    "id": 10,
                    "properties": {"name": "Dublin - Liffey", "ref": "0000002001"},
                    "geometry": {"type": "Point", "coordinates": [-6.260310, 53.346544]}
                }
            ]
        }"#
    }

    #[test]
    fn test_deserialize_feature_collection() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        assert_eq!(fc.collection_type, "FeatureCollection");
        assert_eq!(fc.features.len(), 3);
    }

    #[test]
    fn test_deserialize_feature() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        let feature = &fc.features[0];
        assert_eq!(feature.properties.name.as_deref(), Some("Sandy Mills"));
        assert_eq!(feature.properties.station_ref.as_deref(), Some("0000001041"));
        assert_eq!(feature.id, Some(3));
    }

    #[test]
    fn test_geometry_coordinates() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        let feature = &fc.features[0];
        assert_eq!(feature.geometry.geometry_type, "Point");
        assert!((feature.geometry.coordinates[0] - (-7.575758)).abs() < 0.0001);
        assert!((feature.geometry.coordinates[1] - 54.838318).abs() < 0.0001);
    }

    #[test]
    fn test_station_row_from_feature() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        let row = StationRow::from_feature(&fc.features[0]);
        assert_eq!(row.name, "Sandy Mills");
        assert_eq!(row.station_ref, "0000001041");
        assert_eq!(row.lat, "54.838318");
        assert_eq!(row.lon, "-7.575758");
    }

    #[test]
    fn test_search_stations() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        let results = search_stations(&fc.features, "sandy");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Sandy Mills");
    }

    #[test]
    fn test_search_stations_case_insensitive() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        let results = search_stations(&fc.features, "DUBLIN");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Dublin - Liffey");
    }

    #[test]
    fn test_search_stations_no_match() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        let results = search_stations(&fc.features, "nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_filter_by_county() {
        let fc: FeatureCollection = serde_json::from_str(sample_geojson()).unwrap();
        let results = filter_by_county(&fc.features, "bally");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Ballybofey");
    }

    #[test]
    fn test_station_row_missing_properties() {
        let feature = Feature {
            feature_type: "Feature".to_string(),
            id: None,
            properties: Properties {
                name: None,
                station_ref: None,
            },
            geometry: Geometry {
                geometry_type: "Point".to_string(),
                coordinates: vec![-6.0, 53.0],
            },
        };
        let row = StationRow::from_feature(&feature);
        assert_eq!(row.name, "");
        assert_eq!(row.station_ref, "");
    }
}
