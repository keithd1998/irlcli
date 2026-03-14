use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- ArcGIS REST API response structures --

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryResponse {
    pub features: Option<Vec<ArcGisFeature>>,
    #[serde(rename = "objectIdFieldName")]
    pub object_id_field_name: Option<String>,
    #[serde(rename = "geometryType")]
    pub geometry_type: Option<String>,
    pub fields: Option<Vec<FieldDef>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArcGisFeature {
    pub attributes: Option<serde_json::Value>,
    pub geometry: Option<ArcGisGeometry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArcGisGeometry {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub rings: Option<Vec<Vec<Vec<f64>>>>,
    pub paths: Option<Vec<Vec<Vec<f64>>>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FieldDef {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub field_type: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub service_type: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServicesResponse {
    pub services: Option<Vec<ServiceInfo>>,
}

// -- Display row structs --

#[derive(Debug, Tabled, Serialize)]
pub struct BoundaryRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Type")]
    pub boundary_type: String,
    #[tabled(rename = "ID")]
    pub id: String,
}

impl BoundaryRow {
    pub fn from_feature(feature: &ArcGisFeature, name_field: &str, id_field: &str) -> Self {
        let attrs = feature.attributes.as_ref();
        Self {
            name: attrs
                .and_then(|a| a.get(name_field))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            boundary_type: String::new(),
            id: attrs
                .and_then(|a| a.get(id_field))
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => String::new(),
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct SearchResultRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Type")]
    pub boundary_type: String,
    #[tabled(rename = "Contains Point")]
    pub contains: String,
}

#[derive(Debug, Tabled, Serialize)]
pub struct DatasetRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Type")]
    pub service_type: String,
    #[tabled(rename = "Description")]
    pub description: String,
}

impl DatasetRow {
    pub fn from_service(service: &ServiceInfo) -> Self {
        let description = service.description.clone().unwrap_or_default();
        Self {
            name: service.name.clone().unwrap_or_default(),
            service_type: service.service_type.clone().unwrap_or_default(),
            description: irl_core::truncate_display(&description, 50),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_query_response() {
        let json = r#"{
            "objectIdFieldName": "OBJECTID",
            "geometryType": "esriGeometryPolygon",
            "features": [{
                "attributes": {
                    "OBJECTID": 1,
                    "COUNTY_NAME": "Dublin",
                    "COUNTY_ID": "DUB"
                },
                "geometry": {
                    "rings": [[[-6.0, 53.0], [-6.1, 53.1], [-6.0, 53.0]]]
                }
            }],
            "fields": [
                {"name": "OBJECTID", "type": "esriFieldTypeOID", "alias": "Object ID"},
                {"name": "COUNTY_NAME", "type": "esriFieldTypeString", "alias": "County Name"}
            ]
        }"#;

        let response: QueryResponse = serde_json::from_str(json).unwrap();
        let features = response.features.unwrap();
        assert_eq!(features.len(), 1);
        assert_eq!(
            response.object_id_field_name.as_deref(),
            Some("OBJECTID")
        );
    }

    #[test]
    fn test_deserialize_feature_with_attributes() {
        let json = r#"{
            "attributes": {
                "OBJECTID": 1,
                "NAME": "Dublin",
                "CODE": "D"
            },
            "geometry": {
                "x": -6.26,
                "y": 53.35
            }
        }"#;

        let feature: ArcGisFeature = serde_json::from_str(json).unwrap();
        let attrs = feature.attributes.unwrap();
        assert_eq!(attrs["NAME"].as_str(), Some("Dublin"));
        assert_eq!(attrs["OBJECTID"].as_i64(), Some(1));
    }

    #[test]
    fn test_deserialize_geometry_point() {
        let json = r#"{"x": -6.26, "y": 53.35}"#;

        let geom: ArcGisGeometry = serde_json::from_str(json).unwrap();
        assert!((geom.x.unwrap() - (-6.26)).abs() < 0.001);
        assert!((geom.y.unwrap() - 53.35).abs() < 0.001);
    }

    #[test]
    fn test_deserialize_services_response() {
        let json = r#"{
            "services": [
                {
                    "name": "AdminBoundaries",
                    "description": "Administrative boundaries of Ireland",
                    "type": "FeatureServer",
                    "url": "https://services.arcgis.com/example/FeatureServer"
                }
            ]
        }"#;

        let response: ServicesResponse = serde_json::from_str(json).unwrap();
        let services = response.services.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name.as_deref(), Some("AdminBoundaries"));
        assert_eq!(services[0].service_type.as_deref(), Some("FeatureServer"));
    }

    #[test]
    fn test_boundary_row_from_feature() {
        let json = r#"{
            "attributes": {"COUNTY_NAME": "Cork", "COUNTY_ID": "CRK"},
            "geometry": null
        }"#;

        let feature: ArcGisFeature = serde_json::from_str(json).unwrap();
        let row = BoundaryRow::from_feature(&feature, "COUNTY_NAME", "COUNTY_ID");
        assert_eq!(row.name, "Cork");
        assert_eq!(row.id, "CRK");
    }

    #[test]
    fn test_boundary_row_numeric_id() {
        let json = r#"{
            "attributes": {"NAME": "Dublin", "OBJECTID": 42},
            "geometry": null
        }"#;

        let feature: ArcGisFeature = serde_json::from_str(json).unwrap();
        let row = BoundaryRow::from_feature(&feature, "NAME", "OBJECTID");
        assert_eq!(row.name, "Dublin");
        assert_eq!(row.id, "42");
    }

    #[test]
    fn test_dataset_row_from_service() {
        let service = ServiceInfo {
            name: Some("Counties".to_string()),
            description: Some("County boundaries".to_string()),
            service_type: Some("FeatureServer".to_string()),
            url: Some("https://example.com".to_string()),
        };

        let row = DatasetRow::from_service(&service);
        assert_eq!(row.name, "Counties");
        assert_eq!(row.service_type, "FeatureServer");
        assert_eq!(row.description, "County boundaries");
    }

    #[test]
    fn test_dataset_row_long_description_truncated() {
        let service = ServiceInfo {
            name: Some("Test".to_string()),
            description: Some("A very long description that exceeds the fifty character limit for display".to_string()),
            service_type: Some("FeatureServer".to_string()),
            url: None,
        };

        let row = DatasetRow::from_service(&service);
        assert!(row.description.ends_with("..."));
        assert!(row.description.len() <= 53);
    }

    #[test]
    fn test_boundary_row_missing_attributes() {
        let feature = ArcGisFeature {
            attributes: None,
            geometry: None,
        };

        let row = BoundaryRow::from_feature(&feature, "NAME", "ID");
        assert_eq!(row.name, "");
        assert_eq!(row.id, "");
    }
}
