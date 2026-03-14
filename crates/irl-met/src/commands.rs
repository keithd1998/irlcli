use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::MetApi;
use crate::locations::{self, STATIONS};
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum MetCommands {
    /// Weather observations for a location (today)
    ///
    /// Fetches hourly observation data from Met Eireann for a given station.
    /// Use --hours to limit how many recent hours to show.
    ///
    /// Examples:
    ///   irl met forecast --location dublin
    ///   irl met forecast --location cork --hours 6
    ///   irl met forecast --location "Dublin Airport"
    Forecast {
        /// Location name (e.g. dublin, cork, galway, or full station name)
        #[arg(long)]
        location: String,
        /// Limit to the most recent N hours of observations
        #[arg(long)]
        hours: Option<usize>,
    },

    /// Current weather warnings for Ireland
    ///
    /// Fetches active weather warnings from Met Eireann open data.
    /// Returns an empty table when no warnings are in effect.
    ///
    /// Examples:
    ///   irl met warnings
    Warnings,

    /// List available weather stations / locations
    ///
    /// Shows all supported location aliases and their corresponding
    /// Met Eireann observation stations.
    ///
    /// Examples:
    ///   irl met stations
    Stations,
}

pub async fn handle_command(
    cmd: &MetCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = MetApi::new(verbose, quiet, no_cache)?;

    match cmd {
        MetCommands::Forecast { location, hours } => {
            let station = locations::resolve_location(location).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown location '{}'. Run 'irl met stations' to see available locations.",
                    location
                )
            })?;

            output.print_header(&format!("Weather Observations — {}", station));

            let observations = api.get_observations(station).await?;

            let mut rows: Vec<ObservationRow> = observations
                .iter()
                .map(ObservationRow::from_observation)
                .collect();

            if let Some(n) = hours {
                // Keep only the last N entries (most recent observations)
                let len = rows.len();
                if *n < len {
                    rows = rows.split_off(len - n);
                }
            }

            output.print_info(&format!("{} observations", rows.len()));
            output.render(&rows)?;
        }

        MetCommands::Warnings => {
            output.print_header("Met Eireann Weather Warnings");

            let warnings = api.get_warnings().await?;

            if warnings.is_empty() {
                output.print_info("No active weather warnings for Ireland.");
                return Ok(());
            }

            let rows: Vec<WarningRow> = warnings.iter().map(WarningRow::from_warning).collect();

            output.print_info(&format!("{} active warning(s)", rows.len()));
            output.render(&rows)?;
        }

        MetCommands::Stations => {
            output.print_header("Available Weather Stations");

            let rows: Vec<StationRow> = STATIONS
                .iter()
                .map(|s| StationRow {
                    location: s.alias.to_string(),
                    station: s.api_name.to_string(),
                    county: s.county.to_string(),
                })
                .collect();

            output.print_info(&format!("{} locations available", rows.len()));
            output.render(&rows)?;
        }
    }

    Ok(())
}
