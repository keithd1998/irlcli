use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::EpaApi;
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum EpaCommands {
    /// Current air quality data
    ///
    /// Fetches air quality readings from EPA monitoring stations.
    /// Optionally filter by station name.
    ///
    /// Examples:
    ///   irl epa air-quality
    ///   irl epa air-quality --station rathmines
    AirQuality {
        /// Filter by station name
        #[arg(long)]
        station: Option<String>,
    },

    /// Water quality data
    ///
    /// Fetches water quality monitoring results.
    /// Optionally filter by river catchment.
    ///
    /// Examples:
    ///   irl epa water-quality
    ///   irl epa water-quality --catchment liffey
    WaterQuality {
        /// Filter by catchment area
        #[arg(long)]
        catchment: Option<String>,
    },

    /// Licensed facilities
    ///
    /// Lists EPA licensed facilities (waste, industrial, etc.).
    /// Optionally filter by county.
    ///
    /// Examples:
    ///   irl epa facilities
    ///   irl epa facilities --county dublin
    Facilities {
        /// Filter by county
        #[arg(long)]
        county: Option<String>,
    },

    /// Emissions data
    ///
    /// Fetches emissions data by sector.
    ///
    /// Examples:
    ///   irl epa emissions
    ///   irl epa emissions --sector energy
    Emissions {
        /// Filter by sector
        #[arg(long)]
        sector: Option<String>,
    },
}

pub async fn handle_command(
    cmd: &EpaCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = EpaApi::new(verbose, quiet, no_cache)?;

    match cmd {
        EpaCommands::AirQuality { station } => {
            output.print_header("EPA Air Quality");
            match api.get_air_quality(station.as_deref()).await {
                Ok(response) => {
                    let readings = response.stations.unwrap_or_default();
                    let rows: Vec<AirQualityRow> =
                        readings.iter().map(AirQualityRow::from_reading).collect();
                    output.print_info(&format!("{} stations reporting", rows.len()));
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "EPA data API currently unavailable. \
                         Visit https://airquality.ie for current data.",
                    );
                }
            }
        }

        EpaCommands::WaterQuality { catchment } => {
            output.print_header("EPA Water Quality");
            match api.get_water_quality(catchment.as_deref()).await {
                Ok(response) => {
                    let results = response.results.unwrap_or_default();
                    let rows: Vec<WaterQualityRow> =
                        results.iter().map(WaterQualityRow::from_reading).collect();
                    output.print_info(&format!("{} results found", rows.len()));
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "EPA data API currently unavailable. \
                         Visit https://www.epa.ie/our-services/monitoring--assessment/\
                         assessment/water-quality/ for current data.",
                    );
                }
            }
        }

        EpaCommands::Facilities { county } => {
            output.print_header("EPA Licensed Facilities");
            match api.get_facilities(county.as_deref()).await {
                Ok(response) => {
                    let facilities = response.facilities.unwrap_or_default();
                    let rows: Vec<FacilityRow> =
                        facilities.iter().map(FacilityRow::from_facility).collect();
                    output.print_info(&format!("{} facilities found", rows.len()));
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "EPA data API currently unavailable. \
                         Visit https://www.epa.ie for facility information.",
                    );
                }
            }
        }

        EpaCommands::Emissions { sector } => {
            output.print_header("EPA Emissions Data");
            match api.get_emissions(sector.as_deref()).await {
                Ok(response) => {
                    let data = response.data.unwrap_or_default();
                    let rows: Vec<EmissionsRow> =
                        data.iter().map(EmissionsRow::from_record).collect();
                    output.print_info(&format!("{} records found", rows.len()));
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "EPA data API currently unavailable. \
                         Visit https://www.epa.ie/our-services/monitoring--assessment/\
                         climate-change/ghg/ for emissions data.",
                    );
                }
            }
        }
    }

    Ok(())
}
