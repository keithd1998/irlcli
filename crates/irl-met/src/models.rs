use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- API response: observation entry --

/// A single hourly observation from a Met Eireann weather station.
/// All numeric fields are strings in the API response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Observation {
    pub name: Option<String>,
    pub temperature: Option<String>,
    pub symbol: Option<String>,
    #[serde(rename = "weatherDescription")]
    pub weather_description: Option<String>,
    #[serde(rename = "windSpeed")]
    pub wind_speed: Option<String>,
    #[serde(rename = "windGust")]
    pub wind_gust: Option<String>,
    #[serde(rename = "cardinalWindDirection")]
    pub cardinal_wind_direction: Option<String>,
    #[serde(rename = "windDirection")]
    pub wind_direction: Option<serde_json::Value>,
    pub humidity: Option<String>,
    pub rainfall: Option<String>,
    pub pressure: Option<String>,
    #[serde(rename = "dayName")]
    pub day_name: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "reportTime")]
    pub report_time: Option<String>,
}

// -- API response: weather warning --

/// A weather warning from Met Eireann open data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Warning {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub warning_type: Option<String>,
    pub level: Option<String>,
    pub headline: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub certainty: Option<String>,
    pub issued: Option<String>,
    pub updated: Option<String>,
    pub onset: Option<String>,
    pub expiry: Option<String>,
    pub regions: Option<Vec<String>>,
}

// -- Display row structs for OutputConfig::render() --

/// Table display row for a weather observation.
#[derive(Debug, Tabled, Serialize)]
pub struct ObservationRow {
    #[tabled(rename = "Time")]
    pub time: String,
    #[tabled(rename = "Temp (°C)")]
    pub temperature: String,
    #[tabled(rename = "Weather")]
    pub weather: String,
    #[tabled(rename = "Wind (km/h)")]
    pub wind: String,
    #[tabled(rename = "Wind Dir")]
    pub wind_direction: String,
    #[tabled(rename = "Humidity (%)")]
    pub humidity: String,
    #[tabled(rename = "Rain (mm)")]
    pub rainfall: String,
    #[tabled(rename = "Pressure (hPa)")]
    pub pressure: String,
}

impl ObservationRow {
    pub fn from_observation(obs: &Observation) -> Self {
        Self {
            time: obs.report_time.clone().unwrap_or_default(),
            temperature: obs.temperature.clone().unwrap_or_default(),
            weather: obs.weather_description.clone().unwrap_or_default(),
            wind: obs.wind_speed.clone().unwrap_or_default(),
            wind_direction: obs.cardinal_wind_direction.clone().unwrap_or_default(),
            humidity: obs
                .humidity
                .as_ref()
                .map(|h| h.trim().to_string())
                .unwrap_or_default(),
            rainfall: obs
                .rainfall
                .as_ref()
                .map(|r| r.trim().to_string())
                .unwrap_or_default(),
            pressure: obs.pressure.clone().unwrap_or_default(),
        }
    }
}

/// Table display row for a weather warning.
#[derive(Debug, Tabled, Serialize)]
pub struct WarningRow {
    #[tabled(rename = "Level")]
    pub level: String,
    #[tabled(rename = "Type")]
    pub warning_type: String,
    #[tabled(rename = "Headline")]
    pub headline: String,
    #[tabled(rename = "Severity")]
    pub severity: String,
    #[tabled(rename = "Onset")]
    pub onset: String,
    #[tabled(rename = "Expiry")]
    pub expiry: String,
    #[tabled(rename = "Regions")]
    pub regions: String,
}

impl WarningRow {
    pub fn from_warning(w: &Warning) -> Self {
        let regions = w
            .regions
            .as_ref()
            .map(|rs| rs.join(", "))
            .unwrap_or_default();
        let headline = w.headline.clone().unwrap_or_default();

        Self {
            level: w.level.clone().unwrap_or_default(),
            warning_type: w.warning_type.clone().unwrap_or_default(),
            headline: irl_core::truncate_display(&headline, 60),
            severity: w.severity.clone().unwrap_or_default(),
            onset: w.onset.clone().unwrap_or_default(),
            expiry: w.expiry.clone().unwrap_or_default(),
            regions,
        }
    }
}

/// Table display row for listing available stations.
#[derive(Debug, Tabled, Serialize)]
pub struct StationRow {
    #[tabled(rename = "Location")]
    pub location: String,
    #[tabled(rename = "Station")]
    pub station: String,
    #[tabled(rename = "County")]
    pub county: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const OBSERVATION_JSON: &str = r#"{
        "name": "Dublin Airport",
        "temperature": "3",
        "symbol": "01n",
        "weatherDescription": "Sun / Clear sky",
        "windSpeed": "15",
        "windGust": "-",
        "cardinalWindDirection": "W",
        "windDirection": 270,
        "humidity": " 89 ",
        "rainfall": " 0.0 ",
        "pressure": "1001",
        "dayName": "Saturday",
        "date": "14-03-2026",
        "reportTime": "00:00"
    }"#;

    const OBSERVATIONS_ARRAY_JSON: &str = r#"[
        {
            "name": "Dublin Airport",
            "temperature": "3",
            "symbol": "01n",
            "weatherDescription": "Sun / Clear sky",
            "windSpeed": "15",
            "windGust": "-",
            "cardinalWindDirection": "W",
            "windDirection": 270,
            "humidity": " 89 ",
            "rainfall": " 0.0 ",
            "pressure": "1001",
            "dayName": "Saturday",
            "date": "14-03-2026",
            "reportTime": "00:00"
        },
        {
            "name": "Dublin Airport",
            "temperature": "2",
            "symbol": "01n",
            "weatherDescription": "Clear",
            "windSpeed": "12",
            "windGust": "20",
            "cardinalWindDirection": "NW",
            "windDirection": 315,
            "humidity": " 91 ",
            "rainfall": " 0.0 ",
            "pressure": "1002",
            "dayName": "Saturday",
            "date": "14-03-2026",
            "reportTime": "01:00"
        }
    ]"#;

    #[test]
    fn test_deserialize_single_observation() {
        let obs: Observation = serde_json::from_str(OBSERVATION_JSON).unwrap();
        assert_eq!(obs.name.as_deref(), Some("Dublin Airport"));
        assert_eq!(obs.temperature.as_deref(), Some("3"));
        assert_eq!(obs.weather_description.as_deref(), Some("Sun / Clear sky"));
        assert_eq!(obs.wind_speed.as_deref(), Some("15"));
        assert_eq!(obs.cardinal_wind_direction.as_deref(), Some("W"));
        assert_eq!(obs.humidity.as_deref(), Some(" 89 "));
        assert_eq!(obs.rainfall.as_deref(), Some(" 0.0 "));
        assert_eq!(obs.pressure.as_deref(), Some("1001"));
        assert_eq!(obs.report_time.as_deref(), Some("00:00"));
        assert_eq!(obs.date.as_deref(), Some("14-03-2026"));
    }

    #[test]
    fn test_deserialize_observations_array() {
        let obs: Vec<Observation> = serde_json::from_str(OBSERVATIONS_ARRAY_JSON).unwrap();
        assert_eq!(obs.len(), 2);
        assert_eq!(obs[0].temperature.as_deref(), Some("3"));
        assert_eq!(obs[1].temperature.as_deref(), Some("2"));
        assert_eq!(obs[0].report_time.as_deref(), Some("00:00"));
        assert_eq!(obs[1].report_time.as_deref(), Some("01:00"));
    }

    #[test]
    fn test_observation_row_from_observation() {
        let obs: Observation = serde_json::from_str(OBSERVATION_JSON).unwrap();
        let row = ObservationRow::from_observation(&obs);
        assert_eq!(row.time, "00:00");
        assert_eq!(row.temperature, "3");
        assert_eq!(row.weather, "Sun / Clear sky");
        assert_eq!(row.wind, "15");
        assert_eq!(row.wind_direction, "W");
        assert_eq!(row.humidity, "89"); // trimmed
        assert_eq!(row.rainfall, "0.0"); // trimmed
        assert_eq!(row.pressure, "1001");
    }

    #[test]
    fn test_observation_row_missing_fields() {
        let json = r#"{ "name": "Test" }"#;
        let obs: Observation = serde_json::from_str(json).unwrap();
        let row = ObservationRow::from_observation(&obs);
        assert_eq!(row.time, "");
        assert_eq!(row.temperature, "");
        assert_eq!(row.weather, "");
    }

    #[test]
    fn test_deserialize_empty_warnings() {
        let json = "[]";
        let warnings: Vec<Warning> = serde_json::from_str(json).unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_deserialize_warning() {
        let json = r#"{
            "id": "w001",
            "type": "Wind",
            "level": "Yellow",
            "headline": "Yellow wind warning for Galway and Clare",
            "description": "Strong winds expected",
            "severity": "Moderate",
            "certainty": "Likely",
            "issued": "2026-03-14T06:00:00Z",
            "updated": "2026-03-14T06:00:00Z",
            "onset": "2026-03-14T12:00:00Z",
            "expiry": "2026-03-14T23:00:00Z",
            "regions": ["Galway", "Clare"]
        }"#;
        let warning: Warning = serde_json::from_str(json).unwrap();
        assert_eq!(warning.level.as_deref(), Some("Yellow"));
        assert_eq!(warning.warning_type.as_deref(), Some("Wind"));
        assert_eq!(warning.regions.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_warning_row_from_warning() {
        let json = r#"{
            "id": "w001",
            "type": "Wind",
            "level": "Yellow",
            "headline": "Yellow wind warning for Galway and Clare",
            "severity": "Moderate",
            "onset": "2026-03-14T12:00:00Z",
            "expiry": "2026-03-14T23:00:00Z",
            "regions": ["Galway", "Clare"]
        }"#;
        let warning: Warning = serde_json::from_str(json).unwrap();
        let row = WarningRow::from_warning(&warning);
        assert_eq!(row.level, "Yellow");
        assert_eq!(row.warning_type, "Wind");
        assert_eq!(row.severity, "Moderate");
        assert_eq!(row.regions, "Galway, Clare");
    }

    #[test]
    fn test_warning_row_truncates_long_headline() {
        let json = r#"{
            "headline": "This is a very long warning headline that should be truncated to sixty characters maximum for display"
        }"#;
        let warning: Warning = serde_json::from_str(json).unwrap();
        let row = WarningRow::from_warning(&warning);
        assert!(row.headline.len() <= 60);
        assert!(row.headline.ends_with("..."));
    }

    #[test]
    fn test_observation_row_serializes() {
        let row = ObservationRow {
            time: "12:00".to_string(),
            temperature: "15".to_string(),
            weather: "Cloudy".to_string(),
            wind: "20".to_string(),
            wind_direction: "SW".to_string(),
            humidity: "75".to_string(),
            rainfall: "0.2".to_string(),
            pressure: "1013".to_string(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"time\":\"12:00\""));
        assert!(json.contains("\"temperature\":\"15\""));
    }

    #[test]
    fn test_station_row_serializes() {
        let row = StationRow {
            location: "Dublin".to_string(),
            station: "Dublin Airport".to_string(),
            county: "Dublin".to_string(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"location\":\"Dublin\""));
    }
}
