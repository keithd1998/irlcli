use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::WaterApi;
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum WaterCommands {
    /// List water level monitoring stations
    ///
    /// Fetches the full list of OPW water level monitoring stations.
    /// Optionally filter by county (substring match on station name).
    ///
    /// Examples:
    ///   irl water stations
    ///   irl water stations --county donegal
    Stations {
        /// Filter stations by county (substring match on station name)
        #[arg(long)]
        county: Option<String>,
    },

    /// Current water level at a station
    ///
    /// Fetches the current water level reading for a given station.
    /// Note: Water level data endpoints are being investigated.
    ///
    /// Examples:
    ///   irl water level 0000001041
    ///   irl water level 0000001041 --history 7d
    Level {
        /// Station reference number
        station_id: String,
        /// History period (e.g., 7d, 30d)
        #[arg(long)]
        history: Option<String>,
    },

    /// Stations with high water levels
    ///
    /// Shows stations currently reporting elevated water levels.
    /// Note: Alert data endpoints are being investigated.
    Alerts,

    /// Search stations by name
    ///
    /// Case-insensitive substring search across all station names.
    ///
    /// Examples:
    ///   irl water search "liffey"
    ///   irl water search "cork"
    Search {
        /// Search query
        query: String,
    },
}

pub async fn handle_command(
    cmd: &WaterCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = WaterApi::new(verbose, quiet, no_cache)?;

    match cmd {
        WaterCommands::Stations { county } => {
            output.print_header("OPW Water Level Stations");
            match api.get_stations().await {
                Ok(fc) => {
                    let rows = if let Some(county) = county {
                        let filtered = filter_by_county(&fc.features, county);
                        output.print_info(&format!(
                            "{} stations matching '{}'",
                            filtered.len(),
                            county
                        ));
                        filtered
                    } else {
                        let all: Vec<StationRow> =
                            fc.features.iter().map(StationRow::from_feature).collect();
                        output.print_info(&format!("{} stations found", all.len()));
                        all
                    };
                    output.render(&rows)?;
                }
                Err(e) => {
                    output.print_error(&format!(
                        "Failed to fetch water stations: {}\n\n\
                         Visit https://waterlevel.ie for current data.",
                        e
                    ));
                }
            }
        }

        WaterCommands::Level {
            station_id,
            history: _,
        } => {
            output.print_header("Water Level Reading");
            output.print_info(&format!(
                "Water level data endpoints for station '{}' are being investigated.\n\
                 Station listing and search are available.\n\n\
                 Visit https://waterlevel.ie for current water level readings.",
                station_id
            ));
        }

        WaterCommands::Alerts => {
            output.print_header("Water Level Alerts");
            output.print_info(
                "Water level alert data endpoints are being investigated.\n\
                 Station listing and search are available.\n\n\
                 Visit https://waterlevel.ie for current alert information.",
            );
        }

        WaterCommands::Search { query } => {
            output.print_header("Water Station Search");
            match api.get_stations().await {
                Ok(fc) => {
                    let rows = search_stations(&fc.features, query);
                    output.print_info(&format!(
                        "{} stations matching '{}'",
                        rows.len(),
                        query
                    ));
                    output.render(&rows)?;
                }
                Err(e) => {
                    output.print_error(&format!(
                        "Failed to search water stations: {}\n\n\
                         Visit https://waterlevel.ie for current data.",
                        e
                    ));
                }
            }
        }
    }

    Ok(())
}
