use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- EPA API expected response structures --

#[derive(Debug, Deserialize)]
pub struct AirQualityResponse {
    pub stations: Option<Vec<AirQualityReading>>,
}

#[derive(Debug, Deserialize)]
pub struct AirQualityReading {
    pub station_name: Option<String>,
    pub station_id: Option<String>,
    pub epa_index: Option<String>,
    pub pm25: Option<f64>,
    pub pm10: Option<f64>,
    pub no2: Option<f64>,
    pub o3: Option<f64>,
    pub so2: Option<f64>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WaterQualityResponse {
    pub results: Option<Vec<WaterQualityReading>>,
}

#[derive(Debug, Deserialize)]
pub struct WaterQualityReading {
    pub station_name: Option<String>,
    pub catchment: Option<String>,
    pub q_value: Option<String>,
    pub status: Option<String>,
    pub year: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct FacilitiesResponse {
    pub facilities: Option<Vec<Facility>>,
}

#[derive(Debug, Deserialize)]
pub struct Facility {
    pub name: Option<String>,
    pub licence_number: Option<String>,
    pub county: Option<String>,
    pub licence_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmissionsResponse {
    pub data: Option<Vec<EmissionsRecord>>,
}

#[derive(Debug, Deserialize)]
pub struct EmissionsRecord {
    pub sector: Option<String>,
    pub pollutant: Option<String>,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub year: Option<u32>,
}

// -- Display row structs --

#[derive(Debug, Tabled, Serialize)]
pub struct AirQualityRow {
    #[tabled(rename = "Station")]
    pub station: String,
    #[tabled(rename = "Index")]
    pub index: String,
    #[tabled(rename = "PM2.5")]
    pub pm25: String,
    #[tabled(rename = "PM10")]
    pub pm10: String,
    #[tabled(rename = "NO2")]
    pub no2: String,
    #[tabled(rename = "Time")]
    pub timestamp: String,
}

impl AirQualityRow {
    pub fn from_reading(reading: &AirQualityReading) -> Self {
        Self {
            station: reading.station_name.clone().unwrap_or_default(),
            index: reading.epa_index.clone().unwrap_or_default(),
            pm25: reading.pm25.map(|v| format!("{:.1}", v)).unwrap_or_default(),
            pm10: reading.pm10.map(|v| format!("{:.1}", v)).unwrap_or_default(),
            no2: reading.no2.map(|v| format!("{:.1}", v)).unwrap_or_default(),
            timestamp: reading.timestamp.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct WaterQualityRow {
    #[tabled(rename = "Station")]
    pub station: String,
    #[tabled(rename = "Catchment")]
    pub catchment: String,
    #[tabled(rename = "Q-Value")]
    pub q_value: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Year")]
    pub year: String,
}

impl WaterQualityRow {
    pub fn from_reading(reading: &WaterQualityReading) -> Self {
        Self {
            station: reading.station_name.clone().unwrap_or_default(),
            catchment: reading.catchment.clone().unwrap_or_default(),
            q_value: reading.q_value.clone().unwrap_or_default(),
            status: reading.status.clone().unwrap_or_default(),
            year: reading.year.map(|y| y.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct FacilityRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Licence")]
    pub licence: String,
    #[tabled(rename = "County")]
    pub county: String,
    #[tabled(rename = "Type")]
    pub licence_type: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

impl FacilityRow {
    pub fn from_facility(facility: &Facility) -> Self {
        Self {
            name: facility.name.clone().unwrap_or_default(),
            licence: facility.licence_number.clone().unwrap_or_default(),
            county: facility.county.clone().unwrap_or_default(),
            licence_type: facility.licence_type.clone().unwrap_or_default(),
            status: facility.status.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct EmissionsRow {
    #[tabled(rename = "Sector")]
    pub sector: String,
    #[tabled(rename = "Pollutant")]
    pub pollutant: String,
    #[tabled(rename = "Value")]
    pub value: String,
    #[tabled(rename = "Unit")]
    pub unit: String,
    #[tabled(rename = "Year")]
    pub year: String,
}

impl EmissionsRow {
    pub fn from_record(record: &EmissionsRecord) -> Self {
        Self {
            sector: record.sector.clone().unwrap_or_default(),
            pollutant: record.pollutant.clone().unwrap_or_default(),
            value: record.value.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            unit: record.unit.clone().unwrap_or_default(),
            year: record.year.map(|y| y.to_string()).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_air_quality() {
        let json = r#"{
            "stations": [{
                "station_name": "Rathmines",
                "station_id": "AQ001",
                "epa_index": "Good",
                "pm25": 5.2,
                "pm10": 12.3,
                "no2": 15.1,
                "o3": 45.0,
                "so2": 2.1,
                "timestamp": "2026-03-14T12:00:00Z"
            }]
        }"#;

        let response: AirQualityResponse = serde_json::from_str(json).unwrap();
        let stations = response.stations.unwrap();
        assert_eq!(stations.len(), 1);
        assert_eq!(stations[0].station_name.as_deref(), Some("Rathmines"));
        assert_eq!(stations[0].pm25, Some(5.2));
    }

    #[test]
    fn test_deserialize_water_quality() {
        let json = r#"{
            "results": [{
                "station_name": "Liffey at Islandbridge",
                "catchment": "Liffey",
                "q_value": "Q4",
                "status": "Good",
                "year": 2024
            }]
        }"#;

        let response: WaterQualityResponse = serde_json::from_str(json).unwrap();
        let results = response.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].catchment.as_deref(), Some("Liffey"));
        assert_eq!(results[0].q_value.as_deref(), Some("Q4"));
    }

    #[test]
    fn test_deserialize_facilities() {
        let json = r#"{
            "facilities": [{
                "name": "Dublin Waste Facility",
                "licence_number": "W0001-01",
                "county": "Dublin",
                "licence_type": "Waste",
                "status": "Active"
            }]
        }"#;

        let response: FacilitiesResponse = serde_json::from_str(json).unwrap();
        let facilities = response.facilities.unwrap();
        assert_eq!(facilities.len(), 1);
        assert_eq!(facilities[0].name.as_deref(), Some("Dublin Waste Facility"));
    }

    #[test]
    fn test_deserialize_emissions() {
        let json = r#"{
            "data": [{
                "sector": "Energy",
                "pollutant": "CO2",
                "value": 12345.67,
                "unit": "kt",
                "year": 2023
            }]
        }"#;

        let response: EmissionsResponse = serde_json::from_str(json).unwrap();
        let data = response.data.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].sector.as_deref(), Some("Energy"));
        assert_eq!(data[0].value, Some(12345.67));
    }

    #[test]
    fn test_air_quality_row_from_reading() {
        let reading = AirQualityReading {
            station_name: Some("Rathmines".to_string()),
            station_id: Some("AQ001".to_string()),
            epa_index: Some("Good".to_string()),
            pm25: Some(5.2),
            pm10: Some(12.3),
            no2: Some(15.1),
            o3: Some(45.0),
            so2: Some(2.1),
            timestamp: Some("2026-03-14T12:00:00Z".to_string()),
        };

        let row = AirQualityRow::from_reading(&reading);
        assert_eq!(row.station, "Rathmines");
        assert_eq!(row.index, "Good");
        assert_eq!(row.pm25, "5.2");
    }

    #[test]
    fn test_water_quality_row_from_reading() {
        let reading = WaterQualityReading {
            station_name: Some("Liffey".to_string()),
            catchment: Some("Liffey Catchment".to_string()),
            q_value: Some("Q4".to_string()),
            status: Some("Good".to_string()),
            year: Some(2024),
        };

        let row = WaterQualityRow::from_reading(&reading);
        assert_eq!(row.station, "Liffey");
        assert_eq!(row.q_value, "Q4");
        assert_eq!(row.year, "2024");
    }

    #[test]
    fn test_facility_row_missing_fields() {
        let facility = Facility {
            name: None,
            licence_number: None,
            county: None,
            licence_type: None,
            status: None,
        };

        let row = FacilityRow::from_facility(&facility);
        assert_eq!(row.name, "");
        assert_eq!(row.licence, "");
    }

    #[test]
    fn test_emissions_row_from_record() {
        let record = EmissionsRecord {
            sector: Some("Transport".to_string()),
            pollutant: Some("NOx".to_string()),
            value: Some(99.5),
            unit: Some("kt".to_string()),
            year: Some(2023),
        };

        let row = EmissionsRow::from_record(&record);
        assert_eq!(row.sector, "Transport");
        assert_eq!(row.value, "99.50");
        assert_eq!(row.year, "2023");
    }
}
