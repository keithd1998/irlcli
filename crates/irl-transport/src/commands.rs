use std::collections::HashMap;

use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use irl_core::output::OutputConfig;

use crate::api::TransportApi;
use crate::gtfs_static::GtfsData;
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
    /// Searches static GTFS stop data by name or stop ID.
    /// Downloads GTFS data on first use (cached for 7 days).
    ///
    /// Examples:
    ///   irl transport stops --search "O'Connell"
    ///   irl transport stops --search "Connolly Station"
    ///   irl transport stops --search 8220DB
    Stops {
        /// Search term (matches stop name or ID)
        #[arg(long)]
        search: String,
        /// Maximum number of results (default: 50, use 0 for unlimited)
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// List routes currently active
    ///
    /// Lists routes from real-time vehicle data, enriched with names
    /// from static GTFS data.
    ///
    /// Examples:
    ///   irl transport routes
    ///   irl transport routes --search 46A
    Routes {
        /// Filter by route name or ID (case-insensitive substring)
        #[arg(long)]
        search: Option<String>,
    },

    /// Download/refresh GTFS static data (stops and routes)
    ///
    /// Downloads the latest GTFS schedule data from Transport for Ireland.
    /// This is done automatically on first use, but you can force a refresh.
    ///
    /// Examples:
    ///   irl transport gtfs-update
    GtfsUpdate,
}

#[derive(Debug, Serialize)]
struct StopSearchResult {
    stop_id: String,
    stop_name: String,
    lat: f64,
    lon: f64,
}

pub async fn handle_command(
    cmd: &TransportCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    match cmd {
        TransportCommands::GtfsUpdate => {
            output.print_header("GTFS Static Data Update");
            GtfsData::download(verbose, quiet).await?;
            let data = GtfsData::load(verbose, quiet).await?;
            output.print_info(&format!(
                "Loaded {} stops and {} routes",
                data.stops.len(),
                data.routes.len()
            ));
            return Ok(());
        }

        TransportCommands::Stops { search, limit } => {
            output.print_header("Stop Search");

            let gtfs = GtfsData::load(verbose, quiet).await?;

            // Search by name and ID
            let lower = search.to_lowercase();
            let mut results: Vec<StopSearchResult> = gtfs
                .stops
                .iter()
                .filter(|s| {
                    s.stop_name.to_lowercase().contains(&lower)
                        || s.stop_id.to_lowercase().contains(&lower)
                })
                .map(|s| StopSearchResult {
                    stop_id: s.stop_id.clone(),
                    stop_name: s.stop_name.clone(),
                    lat: s.stop_lat,
                    lon: s.stop_lon,
                })
                .collect();

            let total = results.len();
            if *limit > 0 && results.len() > *limit {
                results.truncate(*limit);
                output.print_info(&format!(
                    "Showing {} of {} stops matching '{}'",
                    limit, total, search
                ));
            } else {
                output.print_info(&format!("{} stops found matching '{}'", results.len(), search));
            }

            output.render_single(&results)?;
            return Ok(());
        }

        _ => {}
    }

    // Commands that need the real-time API
    let api = TransportApi::new(verbose, quiet, no_cache)?;

    // Try to load GTFS for route name enrichment (non-fatal if unavailable)
    let gtfs = GtfsData::load(verbose, true).await.ok();

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

        TransportCommands::Routes { search } => {
            output.print_header("Active Routes");
            let response = api.get_vehicle_positions().await?;
            let entities = response.entity.unwrap_or_default();

            let mut route_counts: HashMap<String, u32> = HashMap::new();
            for entity in &entities {
                if let Some(vp) = &entity.vehicle {
                    if let Some(route_id) = vp.trip.as_ref().and_then(|t| t.route_id.clone()) {
                        *route_counts.entry(route_id).or_insert(0) += 1;
                    }
                }
            }

            // Get current day of week (0=Mon..6=Sun)
            let day_of_week = {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                // Jan 1 1970 was a Thursday (3)
                ((secs / 86400 + 3) % 7) as u32
            };

            let mut rows: Vec<RouteRow> = route_counts
                .into_iter()
                .map(|(route_id, count)| {
                    let (short_name, long_name) = gtfs
                        .as_ref()
                        .and_then(|g| g.get_route(&route_id))
                        .map(|r| (r.route_short_name.clone(), r.route_long_name.clone()))
                        .unwrap_or_default();

                    let scheduled = gtfs
                        .as_ref()
                        .map(|g| g.count_scheduled_trips(&route_id, day_of_week))
                        .unwrap_or(0);

                    RouteRow {
                        route_id,
                        short_name,
                        long_name,
                        active_vehicles: count.to_string(),
                        scheduled_trips: if scheduled > 0 {
                            scheduled.to_string()
                        } else {
                            String::new()
                        },
                    }
                })
                .collect();

            // Apply search filter on names
            if let Some(query) = search {
                let lower = query.to_lowercase();
                rows.retain(|r| {
                    r.route_id.to_lowercase().contains(&lower)
                        || r.short_name.to_lowercase().contains(&lower)
                        || r.long_name.to_lowercase().contains(&lower)
                });
            }

            rows.sort_by(|a, b| a.short_name.cmp(&b.short_name));

            output.print_info(&format!("{} active routes found", rows.len()));
            output.render(&rows)?;
        }

        // Already handled above
        TransportCommands::Stops { .. } | TransportCommands::GtfsUpdate => unreachable!(),
    }

    Ok(())
}
