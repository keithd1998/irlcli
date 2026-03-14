// Geographic utilities for cross-source location queries.

use serde::Serialize;

/// A geographic location with optional metadata.
#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

/// Approximate distance between two lat/lon points in kilometres.
/// Uses the Haversine formula.
pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_same_point() {
        assert!((haversine_km(53.35, -6.26, 53.35, -6.26) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_haversine_dublin_to_cork() {
        // Dublin Airport to Cork Airport ~220km
        let dist = haversine_km(53.4264, -6.2499, 51.8413, -8.4911);
        assert!(dist > 200.0 && dist < 250.0, "Expected ~220km, got {}", dist);
    }

    #[test]
    fn test_haversine_dublin_to_galway() {
        // Dublin to Galway ~190km
        let dist = haversine_km(53.4264, -6.2499, 53.2964, -8.7483);
        assert!(dist > 150.0 && dist < 200.0, "Expected ~175km, got {}", dist);
    }

    #[test]
    fn test_location_serializes() {
        let loc = Location {
            name: "Dublin".to_string(),
            lat: 53.35,
            lon: -6.26,
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains("Dublin"));
        assert!(json.contains("53.35"));
    }
}
