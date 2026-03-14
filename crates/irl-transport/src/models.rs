use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- GTFS-R JSON response structures --
// The NTA API returns snake_case field names when using ?format=json

#[derive(Debug, Deserialize)]
pub struct GtfsResponse {
    pub header: Option<FeedHeader>,
    pub entity: Option<Vec<FeedEntity>>,
}

#[derive(Debug, Deserialize)]
pub struct FeedHeader {
    #[serde(alias = "gtfsRealtimeVersion", alias = "gtfs_realtime_version")]
    pub gtfs_realtime_version: Option<String>,
    pub timestamp: Option<String>,
    pub incrementality: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeedEntity {
    pub id: Option<String>,
    #[serde(alias = "tripUpdate", alias = "trip_update")]
    pub trip_update: Option<TripUpdate>,
    pub vehicle: Option<VehiclePosition>,
}

// -- Trip Updates --

#[derive(Debug, Deserialize)]
pub struct TripUpdate {
    pub trip: Option<TripDescriptor>,
    #[serde(alias = "stopTimeUpdate", alias = "stop_time_update")]
    pub stop_time_update: Option<Vec<StopTimeUpdate>>,
    pub vehicle: Option<VehicleDescriptor>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TripDescriptor {
    #[serde(alias = "tripId", alias = "trip_id")]
    pub trip_id: Option<String>,
    #[serde(alias = "routeId", alias = "route_id")]
    pub route_id: Option<String>,
    #[serde(alias = "directionId", alias = "direction_id")]
    pub direction_id: Option<u32>,
    #[serde(alias = "startTime", alias = "start_time")]
    pub start_time: Option<String>,
    #[serde(alias = "startDate", alias = "start_date")]
    pub start_date: Option<String>,
    #[serde(alias = "scheduleRelationship", alias = "schedule_relationship")]
    pub schedule_relationship: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StopTimeUpdate {
    #[serde(alias = "stopSequence", alias = "stop_sequence")]
    pub stop_sequence: Option<u32>,
    #[serde(alias = "stopId", alias = "stop_id")]
    pub stop_id: Option<String>,
    pub arrival: Option<StopTimeEvent>,
    pub departure: Option<StopTimeEvent>,
    #[serde(alias = "scheduleRelationship", alias = "schedule_relationship")]
    pub schedule_relationship: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StopTimeEvent {
    pub delay: Option<i64>,
    pub time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VehicleDescriptor {
    pub id: Option<String>,
    pub label: Option<String>,
}

// -- Vehicle Positions --

#[derive(Debug, Deserialize)]
pub struct VehiclePosition {
    pub trip: Option<TripDescriptor>,
    pub vehicle: Option<VehicleDescriptor>,
    pub position: Option<Position>,
    pub timestamp: Option<String>,
    #[serde(alias = "currentStopSequence", alias = "current_stop_sequence")]
    pub current_stop_sequence: Option<u32>,
    #[serde(alias = "stopId", alias = "stop_id")]
    pub stop_id: Option<String>,
    #[serde(alias = "currentStatus", alias = "current_status")]
    pub current_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub bearing: Option<f64>,
    pub speed: Option<f64>,
}

// -- Display row structs --

#[derive(Debug, Tabled, Serialize)]
pub struct DepartureRow {
    #[tabled(rename = "Route")]
    pub route: String,
    #[tabled(rename = "Stop")]
    pub stop_id: String,
    #[tabled(rename = "Scheduled")]
    pub scheduled: String,
    #[tabled(rename = "Delay")]
    pub delay: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

impl DepartureRow {
    pub fn from_stop_time_update(
        route_id: &str,
        stop_id_filter: &str,
        stu: &StopTimeUpdate,
    ) -> Option<Self> {
        let stop = stu.stop_id.as_deref().unwrap_or("");
        if !stop.contains(stop_id_filter) && stop_id_filter != stop {
            return None;
        }

        let (scheduled, delay) = if let Some(dep) = &stu.departure {
            let time_str = dep.time.clone().unwrap_or_default();
            let delay_secs = dep.delay.unwrap_or(0);
            let delay_str = if delay_secs == 0 {
                "On time".to_string()
            } else if delay_secs > 0 {
                format!("+{} min", delay_secs / 60)
            } else {
                format!("{} min", delay_secs / 60)
            };
            (time_str, delay_str)
        } else if let Some(arr) = &stu.arrival {
            let time_str = arr.time.clone().unwrap_or_default();
            let delay_secs = arr.delay.unwrap_or(0);
            let delay_str = if delay_secs == 0 {
                "On time".to_string()
            } else if delay_secs > 0 {
                format!("+{} min", delay_secs / 60)
            } else {
                format!("{} min", delay_secs / 60)
            };
            (time_str, delay_str)
        } else {
            return None;
        };

        let status = stu
            .schedule_relationship
            .clone()
            .unwrap_or_else(|| "SCHEDULED".to_string());

        Some(Self {
            route: route_id.to_string(),
            stop_id: stop.to_string(),
            scheduled,
            delay,
            status,
        })
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct VehicleRow {
    #[tabled(rename = "Vehicle")]
    pub vehicle_id: String,
    #[tabled(rename = "Route")]
    pub route: String,
    #[tabled(rename = "Latitude")]
    pub latitude: String,
    #[tabled(rename = "Longitude")]
    pub longitude: String,
    #[tabled(rename = "Speed")]
    pub speed: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

impl VehicleRow {
    pub fn from_entity(entity: &FeedEntity) -> Option<Self> {
        let vp = entity.vehicle.as_ref()?;
        let pos = vp.position.as_ref()?;
        let vehicle_id = vp
            .vehicle
            .as_ref()
            .and_then(|v| v.id.clone())
            .unwrap_or_else(|| entity.id.clone().unwrap_or_default());
        let route = vp
            .trip
            .as_ref()
            .and_then(|t| t.route_id.clone())
            .unwrap_or_default();

        Some(Self {
            vehicle_id,
            route,
            latitude: pos
                .latitude
                .map(|l| format!("{:.5}", l))
                .unwrap_or_default(),
            longitude: pos
                .longitude
                .map(|l| format!("{:.5}", l))
                .unwrap_or_default(),
            speed: pos
                .speed
                .map(|s| format!("{:.1} km/h", s * 3.6))
                .unwrap_or_else(|| "N/A".to_string()),
            status: vp.current_status.clone().unwrap_or_default(),
        })
    }
}

#[derive(Debug, Tabled, Serialize)]
pub struct StopRow {
    #[tabled(rename = "Stop ID")]
    pub stop_id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Routes")]
    pub routes: String,
}

#[derive(Debug, Tabled, Serialize)]
pub struct RouteRow {
    #[tabled(rename = "Route ID")]
    pub route_id: String,
    #[tabled(rename = "Name")]
    pub short_name: String,
    #[tabled(rename = "Description")]
    pub long_name: String,
    #[tabled(rename = "Active Vehicles")]
    pub active_vehicles: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_vehicle_position_snake_case() {
        // Real API format — snake_case
        let json = r#"{
            "header": {
                "gtfs_realtime_version": "2.0",
                "incrementality": "FULL_DATASET",
                "timestamp": "1773504370"
            },
            "entity": [{
                "id": "V1",
                "vehicle": {
                    "trip": {
                        "trip_id": "119662_332",
                        "start_time": "16:05:00",
                        "start_date": "20260314",
                        "schedule_relationship": "SCHEDULED",
                        "route_id": "119662",
                        "direction_id": 0
                    },
                    "position": {
                        "latitude": 53.3501778,
                        "longitude": -6.25059223
                    },
                    "timestamp": "1773504356",
                    "vehicle": {
                        "id": "4"
                    }
                }
            }]
        }"#;

        let response: GtfsResponse = serde_json::from_str(json).unwrap();
        assert!(response.entity.is_some());
        let entities = response.entity.unwrap();
        assert_eq!(entities.len(), 1);

        let vp = entities[0].vehicle.as_ref().unwrap();
        assert_eq!(vp.trip.as_ref().unwrap().route_id.as_deref(), Some("119662"));
        assert_eq!(vp.vehicle.as_ref().unwrap().id.as_deref(), Some("4"));
        let pos = vp.position.as_ref().unwrap();
        assert!((pos.latitude.unwrap() - 53.3501).abs() < 0.01);
    }

    #[test]
    fn test_deserialize_vehicle_position_camel_case() {
        // Alternate camelCase format
        let json = r#"{
            "header": {
                "gtfsRealtimeVersion": "2.0",
                "timestamp": "1710000000"
            },
            "entity": [{
                "id": "V1",
                "vehicle": {
                    "trip": {
                        "tripId": "T1",
                        "routeId": "46A",
                        "directionId": 0
                    },
                    "vehicle": {
                        "id": "BUS123",
                        "label": "Bus 123"
                    },
                    "position": {
                        "latitude": 53.3498,
                        "longitude": -6.2603,
                        "bearing": 180.0,
                        "speed": 12.5
                    },
                    "currentStatus": "IN_TRANSIT_TO",
                    "timestamp": "1710000000"
                }
            }]
        }"#;

        let response: GtfsResponse = serde_json::from_str(json).unwrap();
        let entities = response.entity.unwrap();
        let vp = entities[0].vehicle.as_ref().unwrap();
        assert_eq!(vp.trip.as_ref().unwrap().route_id.as_deref(), Some("46A"));
        assert_eq!(vp.vehicle.as_ref().unwrap().id.as_deref(), Some("BUS123"));
    }

    #[test]
    fn test_deserialize_trip_update() {
        let json = r#"{
            "header": {
                "gtfsRealtimeVersion": "2.0",
                "timestamp": "1710000000"
            },
            "entity": [{
                "id": "TU1",
                "tripUpdate": {
                    "trip": {
                        "tripId": "T100",
                        "routeId": "LUAS-RED",
                        "startTime": "08:30:00",
                        "startDate": "20250314"
                    },
                    "stopTimeUpdate": [{
                        "stopSequence": 1,
                        "stopId": "LUAS_TAL",
                        "departure": {
                            "delay": 120,
                            "time": "1710000120"
                        }
                    }, {
                        "stopSequence": 2,
                        "stopId": "LUAS_HOS",
                        "arrival": {
                            "delay": 60,
                            "time": "1710000180"
                        }
                    }],
                    "timestamp": "1710000000"
                }
            }]
        }"#;

        let response: GtfsResponse = serde_json::from_str(json).unwrap();
        let entities = response.entity.unwrap();
        let tu = entities[0].trip_update.as_ref().unwrap();
        assert_eq!(
            tu.trip.as_ref().unwrap().route_id.as_deref(),
            Some("LUAS-RED")
        );
        let updates = tu.stop_time_update.as_ref().unwrap();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].departure.as_ref().unwrap().delay, Some(120));
    }

    #[test]
    fn test_vehicle_row_from_entity() {
        let entity = FeedEntity {
            id: Some("V1".to_string()),
            trip_update: None,
            vehicle: Some(VehiclePosition {
                trip: Some(TripDescriptor {
                    trip_id: Some("T1".to_string()),
                    route_id: Some("46A".to_string()),
                    direction_id: None,
                    start_time: None,
                    start_date: None,
                    schedule_relationship: None,
                }),
                vehicle: Some(VehicleDescriptor {
                    id: Some("BUS123".to_string()),
                    label: None,
                }),
                position: Some(Position {
                    latitude: Some(53.3498),
                    longitude: Some(-6.2603),
                    bearing: None,
                    speed: Some(10.0),
                }),
                timestamp: None,
                current_stop_sequence: None,
                stop_id: None,
                current_status: Some("IN_TRANSIT_TO".to_string()),
            }),
        };

        let row = VehicleRow::from_entity(&entity).unwrap();
        assert_eq!(row.vehicle_id, "BUS123");
        assert_eq!(row.route, "46A");
        assert_eq!(row.status, "IN_TRANSIT_TO");
    }

    #[test]
    fn test_departure_row_from_stop_time_update() {
        let stu = StopTimeUpdate {
            stop_sequence: Some(1),
            stop_id: Some("STOP_123".to_string()),
            arrival: None,
            departure: Some(StopTimeEvent {
                delay: Some(300),
                time: Some("1710000300".to_string()),
            }),
            schedule_relationship: None,
        };

        let row = DepartureRow::from_stop_time_update("46A", "STOP_123", &stu).unwrap();
        assert_eq!(row.route, "46A");
        assert_eq!(row.stop_id, "STOP_123");
        assert_eq!(row.delay, "+5 min");
    }

    #[test]
    fn test_departure_row_no_match() {
        let stu = StopTimeUpdate {
            stop_sequence: Some(1),
            stop_id: Some("OTHER_STOP".to_string()),
            arrival: None,
            departure: Some(StopTimeEvent {
                delay: Some(0),
                time: Some("1710000000".to_string()),
            }),
            schedule_relationship: None,
        };

        let row = DepartureRow::from_stop_time_update("46A", "STOP_123", &stu);
        assert!(row.is_none());
    }

    #[test]
    fn test_vehicle_row_missing_position() {
        let entity = FeedEntity {
            id: Some("V1".to_string()),
            trip_update: None,
            vehicle: Some(VehiclePosition {
                trip: None,
                vehicle: None,
                position: None,
                timestamp: None,
                current_stop_sequence: None,
                stop_id: None,
                current_status: None,
            }),
        };

        let row = VehicleRow::from_entity(&entity);
        assert!(row.is_none());
    }
}
