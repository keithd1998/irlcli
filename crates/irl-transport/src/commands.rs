use std::collections::{HashMap, HashSet};

use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::TransportApi;
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum TransportCommands {
    /// Next departures from a stop
    ///
    /// Shows upcoming departures for a given stop ID using real-time GTFS data.
    /// Stop IDs can be found using the 'stops' command.
    ///
    /// Examples:
    ///   irl transport departures --stop 8220DB000002
    ///   irl transport departures --stop 8220DB000002 --route 46A
    Departures {
        /// Stop ID to get departures for
        #[arg(long)]
        stop: String,
        /// Filter by route ID
        #[arg(long)]
        route: Option<String>,
        /// Maximum number of results (default: 50, use 0 for unlimited)
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Live vehicle positions
    ///
    /// Shows real-time GPS positions for vehicles on a given route.
    ///
    /// Examples:
    ///   irl transport vehicles --route 46A
    Vehicles {
        /// Route ID to track
        #[arg(long)]
        route: String,
    },

    /// Search stops by name or ID
    ///
    /// Searches real-time data for stop IDs currently in use.
    /// Note: Full stop name search requires static GTFS data (future feature).
    ///
    /// Examples:
    ///   irl transport stops --search 8220DB
    Stops {
        /// Search term (matches against stop IDs in real-time data)
        #[arg(long)]
        search: String,
        /// Maximum number of results (default: 50, use 0 for unlimited)
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// List routes currently active
    ///
    /// Lists route IDs found in real-time vehicle position data.
    /// Note: Full route details require static GTFS data (future feature).
    ///
    /// Examples:
    ///   irl transport routes
    ///   irl transport routes --operator dublin-bus
    Routes {
        /// Filter hint (matched against route IDs; full operator filtering requires static data)
        #[arg(long)]
        operator: Option<String>,
    },
}

pub async fn handle_command(
    cmd: &TransportCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = TransportApi::new(verbose, quiet, no_cache)?;

    match cmd {
        TransportCommands::Departures { stop, route, limit } => {
            output.print_header("Departures");
            let response = api.get_trip_updates().await?;
            let entities = response.entity.unwrap_or_default();

            let mut rows: Vec<DepartureRow> = Vec::new();
            for entity in &entities {
                if let Some(tu) = &entity.trip_update {
                    let route_id = tu
                        .trip
                        .as_ref()
                        .and_then(|t| t.route_id.as_deref())
                        .unwrap_or("");

                    if let Some(route_filter) = route {
                        if !route_id.eq_ignore_ascii_case(route_filter) {
                            continue;
                        }
                    }

                    if let Some(updates) = &tu.stop_time_update {
                        for stu in updates {
                            if let Some(row) =
                                DepartureRow::from_stop_time_update(route_id, stop, stu)
                            {
                                rows.push(row);
                            }
                        }
                    }
                }
            }

            let total = rows.len();
            if *limit > 0 && rows.len() > *limit {
                rows.truncate(*limit);
                output.print_info(&format!(
                    "Showing {} of {} departures for stop {}",
                    limit, total, stop
                ));
            } else {
                output.print_info(&format!(
                    "{} departures found for stop {}",
                    rows.len(),
                    stop
                ));
            }
            output.render(&rows)?;
        }

        TransportCommands::Vehicles { route } => {
            output.print_header("Vehicle Positions");
            let response = api.get_vehicle_positions().await?;
            let entities = response.entity.unwrap_or_default();

            let rows: Vec<VehicleRow> = entities
                .iter()
                .filter(|e| {
                    e.vehicle
                        .as_ref()
                        .and_then(|v| v.trip.as_ref())
                        .and_then(|t| t.route_id.as_deref())
                        .map(|r| r.eq_ignore_ascii_case(route))
                        .unwrap_or(false)
                })
                .filter_map(VehicleRow::from_entity)
                .collect();

            output.print_info(&format!("{} vehicles on route {}", rows.len(), route));
            output.render(&rows)?;
        }

        TransportCommands::Stops { search, limit } => {
            output.print_header("Stops (from real-time data)");
            output.print_info(
                "Note: Searching stop IDs from real-time feed. \
                 Full stop name search requires static GTFS data (future feature).",
            );

            let response = api.get_trip_updates().await?;
            let entities = response.entity.unwrap_or_default();

            let mut stop_routes: HashMap<String, HashSet<String>> = HashMap::new();
            for entity in &entities {
                if let Some(tu) = &entity.trip_update {
                    let route_id = tu
                        .trip
                        .as_ref()
                        .and_then(|t| t.route_id.clone())
                        .unwrap_or_default();

                    if let Some(updates) = &tu.stop_time_update {
                        for stu in updates {
                            if let Some(stop_id) = &stu.stop_id {
                                if stop_id.to_lowercase().contains(&search.to_lowercase()) {
                                    stop_routes
                                        .entry(stop_id.clone())
                                        .or_default()
                                        .insert(route_id.clone());
                                }
                            }
                        }
                    }
                }
            }

            let mut rows: Vec<StopRow> = stop_routes
                .into_iter()
                .map(|(stop_id, routes)| {
                    let mut route_list: Vec<String> = routes.into_iter().collect();
                    route_list.sort();
                    StopRow {
                        stop_id,
                        routes: route_list.join(", "),
                    }
                })
                .collect();
            rows.sort_by(|a, b| a.stop_id.cmp(&b.stop_id));

            let total = rows.len();
            if *limit > 0 && rows.len() > *limit {
                rows.truncate(*limit);
                output.print_info(&format!(
                    "Showing {} of {} stops matching '{}'",
                    limit, total, search
                ));
            } else {
                output.print_info(&format!("{} stops found matching '{}'", rows.len(), search));
            }
            output.render(&rows)?;
        }

        TransportCommands::Routes { operator } => {
            output.print_header("Active Routes (from real-time data)");
            output.print_info(
                "Note: Route IDs from real-time vehicle feed. \
                 Full route details and operator filtering require static GTFS data (future feature).",
            );

            let response = api.get_vehicle_positions().await?;
            let entities = response.entity.unwrap_or_default();

            let mut route_counts: HashMap<String, u32> = HashMap::new();
            for entity in &entities {
                if let Some(vp) = &entity.vehicle {
                    if let Some(route_id) = vp.trip.as_ref().and_then(|t| t.route_id.clone()) {
                        if let Some(op_filter) = operator {
                            if !route_id.to_lowercase().contains(&op_filter.to_lowercase()) {
                                continue;
                            }
                        }
                        *route_counts.entry(route_id).or_insert(0) += 1;
                    }
                }
            }

            let mut rows: Vec<RouteRow> = route_counts
                .into_iter()
                .map(|(route_id, count)| RouteRow {
                    route_id,
                    active_vehicles: count.to_string(),
                })
                .collect();
            rows.sort_by(|a, b| a.route_id.cmp(&b.route_id));

            output.print_info(&format!("{} active routes found", rows.len()));
            output.render(&rows)?;
        }
    }

    Ok(())
}
