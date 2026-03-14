use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;

use irl_core::config::Config;
use irl_core::geo;
use irl_core::output::{OutputConfig, OutputFormat};
use irl_cro::commands::CroCommands;
use irl_cso::commands::CsoCommands;
use irl_epa::commands::EpaCommands;
use irl_geo::commands::GeoCommands;
use irl_met::commands::MetCommands;
use irl_oireachtas::commands::OireachtasCommands;
use irl_property::commands::PropertyCommands;
use irl_tailte::commands::TailteCommands;
use irl_transport::commands::TransportCommands;
use irl_water::commands::WaterCommands;

#[derive(Parser)]
#[command(
    name = "irl",
    version,
    about = "Unified CLI for Irish public sector open data",
    long_about = "Access Irish government data from the command line.\n\n\
        Available data sources:\n  \
        oireachtas  Houses of the Oireachtas (parliament)\n  \
        met         Met Éireann weather forecasts & warnings\n  \
        cso         Central Statistics Office (PxStat)\n  \
        transport   Transport for Ireland (NTA/GTFS)\n  \
        cro         Companies Registration Office\n  \
        property    Property Price Register (PSRA)\n  \
        epa         Environmental Protection Agency\n  \
        water       OPW Water Levels\n  \
        tailte      Tailte Éireann (Valuation Office)\n  \
        geo         GeoHive / OSi spatial data\n\n\
        Get started:\n  \
        irl config init              Create config file\n  \
        irl oireachtas members       List TDs and Senators\n  \
        irl met forecast --location dublin   Weather forecast"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: table, json, or csv
    #[arg(long, global = true)]
    format: Option<String>,

    /// Disable coloured output
    #[arg(long, global = true)]
    no_colour: bool,

    /// Bypass local cache
    #[arg(long, global = true)]
    no_cache: bool,

    /// Show HTTP request/response details
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress everything except data output
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Houses of the Oireachtas — TDs, Senators, legislation, debates, votes
    Oireachtas {
        #[command(subcommand)]
        command: OireachtasCommands,
    },

    /// Met Éireann — weather observations, warnings, and station data
    Met {
        #[command(subcommand)]
        command: MetCommands,
    },

    /// Central Statistics Office — census, economy, health, housing data
    Cso {
        #[command(subcommand)]
        command: CsoCommands,
    },

    /// Transport for Ireland — bus/rail departures, routes, vehicle positions
    Transport {
        #[command(subcommand)]
        command: TransportCommands,
    },

    /// Companies Registration Office — company search and filings
    Cro {
        #[command(subcommand)]
        command: CroCommands,
    },

    /// Property Price Register — residential property sales since 2010
    Property {
        #[command(subcommand)]
        command: PropertyCommands,
    },

    /// Environmental Protection Agency — air quality, water quality, emissions
    Epa {
        #[command(subcommand)]
        command: EpaCommands,
    },

    /// OPW Water Levels — real-time river and lake monitoring
    Water {
        #[command(subcommand)]
        command: WaterCommands,
    },

    /// Tailte Éireann — commercial property valuation data
    Tailte {
        #[command(subcommand)]
        command: TailteCommands,
    },

    /// GeoHive / OSi — spatial data, boundaries, and geographic datasets
    Geo {
        #[command(subcommand)]
        command: GeoCommands,
    },

    /// What's near a location — combines weather, water, and more
    ///
    /// Shows data from multiple sources for a geographic area.
    /// Use --location for a named place or --lat/--lon for coordinates.
    ///
    /// Examples:
    ///   irl nearby --location dublin
    ///   irl nearby --lat 53.35 --lon -6.26
    Nearby {
        /// Location name (e.g., dublin, cork, galway)
        #[arg(long)]
        location: Option<String>,
        /// Latitude (WGS84)
        #[arg(long)]
        lat: Option<f64>,
        /// Longitude (WGS84)
        #[arg(long, allow_hyphen_values = true)]
        lon: Option<f64>,
    },

    /// Manage irl configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Create a new configuration file with defaults
    Init,
    /// Set a configuration value
    Set {
        /// Key in format section.key (e.g., transport.api_key)
        key: String,
        /// Value to set
        value: String,
    },
    /// Show current configuration
    Show,
}

fn resolve_format(cli_format: &Option<String>, config: &Config) -> OutputFormat {
    if let Some(fmt) = cli_format {
        OutputFormat::from_str_opt(fmt).unwrap_or(OutputFormat::Table)
    } else {
        OutputFormat::from_str_opt(&config.general.default_format).unwrap_or(OutputFormat::Table)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load().unwrap_or_default();

    let format = resolve_format(&cli.format, &config);
    let colour = config.general.colour && !cli.no_colour;
    let output = OutputConfig::new(format, colour, cli.quiet);

    match &cli.command {
        Commands::Oireachtas { command } => {
            irl_oireachtas::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }

        Commands::Config { command } => match command {
            ConfigCommands::Init => {
                let path = Config::config_path();
                if path.exists() {
                    output.print_info(&format!("Config already exists at {}", path.display()));
                } else {
                    Config::init_interactive(&path)?;
                    output.print_info(&format!("Config created at {}", path.display()));
                }
            }
            ConfigCommands::Set { key, value } => {
                let mut config = Config::load().unwrap_or_default();
                config.set_value(key, value)?;
                config.save()?;
                output.print_info(&format!("Set {} = {}", key, value));
            }
            ConfigCommands::Show => {
                let config = Config::load().unwrap_or_default();
                let toml_str = toml::to_string_pretty(&config)?;
                println!("{}", toml_str);
            }
        },

        Commands::Met { command } => {
            irl_met::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Cso { command } => {
            irl_cso::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Transport { command } => {
            irl_transport::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Cro { command } => {
            irl_cro::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Property { command } => {
            irl_property::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Epa { command } => {
            irl_epa::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Water { command } => {
            irl_water::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Tailte { command } => {
            irl_tailte::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }
        Commands::Geo { command } => {
            irl_geo::commands::handle_command(
                command,
                &output,
                cli.verbose,
                cli.quiet,
                cli.no_cache,
            )
            .await?;
        }

        Commands::Nearby {
            location,
            lat,
            lon,
        } => {
            handle_nearby(&output, location, lat, lon, cli.verbose, cli.quiet, cli.no_cache).await?;
        }
    }

    Ok(())
}

/// Cross-source "nearby" handler — combines weather, water, and other data.
#[derive(Debug, Serialize)]
struct NearbyResult {
    location: geo::Location,
    weather: Option<NearbyWeather>,
    water_stations: Vec<NearbyWaterStation>,
}

#[derive(Debug, Serialize)]
struct NearbyWeather {
    station: String,
    distance_km: f64,
    temperature: String,
    weather: String,
    wind: String,
    rainfall: String,
}

#[derive(Debug, Serialize)]
struct NearbyWaterStation {
    name: String,
    distance_km: f64,
}

async fn handle_nearby(
    output: &OutputConfig,
    location: &Option<String>,
    lat: &Option<f64>,
    lon: &Option<f64>,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    use irl_met::locations::STATIONS;

    output.print_header("Nearby — Cross-Source View");

    // Resolve location to coordinates
    let (center_lat, center_lon, location_name) = if let Some(loc_name) = location {
        // Find coordinates from Met station aliases
        let station = STATIONS
            .iter()
            .find(|s| s.alias == loc_name.to_lowercase())
            .or_else(|| {
                STATIONS
                    .iter()
                    .find(|s| s.api_name.to_lowercase() == loc_name.to_lowercase())
            })
            .or_else(|| {
                // Fuzzy match
                let aliases: Vec<&str> = STATIONS.iter().map(|s| s.alias).collect();
                let matches = irl_core::fuzzy::fuzzy_match(loc_name, &aliases, 0.8);
                matches.first().and_then(|m| {
                    STATIONS.iter().find(|s| s.alias == m.candidate)
                })
            });

        match station {
            Some(s) => (s.lat, s.lon, loc_name.clone()),
            None => {
                output.print_error(&format!(
                    "Unknown location '{}'. Use `irl met stations` to see available locations, \
                     or use --lat and --lon for custom coordinates.",
                    loc_name
                ));
                return Ok(());
            }
        }
    } else if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
        (*lat_val, *lon_val, format!("{:.4}, {:.4}", lat_val, lon_val))
    } else {
        output.print_error(
            "Specify --location <name> or --lat <LAT> --lon <LON>",
        );
        return Ok(());
    };

    let loc = geo::Location {
        name: location_name.clone(),
        lat: center_lat,
        lon: center_lon,
    };

    output.print_info(&format!(
        "Location: {} ({:.4}, {:.4})",
        location_name, center_lat, center_lon
    ));

    // Find nearest Met station and get weather
    let mut station_distances: Vec<(&irl_met::locations::Station, f64)> = STATIONS
        .iter()
        .map(|s| (s, geo::haversine_km(center_lat, center_lon, s.lat, s.lon)))
        .collect();
    station_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    station_distances.dedup_by(|a, b| a.0.api_name == b.0.api_name);

    let nearest_station = station_distances.first().map(|(s, _)| *s);

    let weather = if let Some(station) = nearest_station {
        let dist = geo::haversine_km(center_lat, center_lon, station.lat, station.lon);
        let met_api = irl_met::api::MetApi::new(verbose, quiet, no_cache)?;
        match met_api.get_observations(station.api_name).await {
            Ok(obs) => obs.last().map(|latest| NearbyWeather {
                station: station.api_name.to_string(),
                distance_km: (dist * 10.0).round() / 10.0,
                temperature: format!(
                    "{}°C",
                    latest.temperature.as_deref().unwrap_or("?")
                ),
                weather: latest
                    .weather_description
                    .clone()
                    .unwrap_or_default(),
                wind: format!(
                    "{} km/h {}",
                    latest.wind_speed.as_deref().unwrap_or("?"),
                    latest.cardinal_wind_direction.as_deref().unwrap_or("")
                ),
                rainfall: format!(
                    "{} mm",
                    latest
                        .rainfall
                        .as_ref()
                        .map(|r| r.trim())
                        .unwrap_or("?")
                ),
            })
            Err(_) => None,
        }
    } else {
        None
    };

    // Find nearby water monitoring stations
    let water_stations = match irl_water::api::WaterApi::new(verbose, quiet, no_cache) {
        Ok(water_api) => match water_api.get_stations().await {
            Ok(fc) => {
                let mut nearby: Vec<NearbyWaterStation> = fc
                    .features
                    .iter()
                    .filter_map(|f| {
                        let coords = &f.geometry.coordinates;
                        if coords.len() >= 2 {
                            let station_lon = coords[0];
                            let station_lat = coords[1];
                            let dist =
                                geo::haversine_km(center_lat, center_lon, station_lat, station_lon);
                            if dist < 50.0 {
                                let name = f
                                    .properties
                                    .name
                                    .clone()
                                    .unwrap_or_else(|| "Unknown".to_string());
                                Some(NearbyWaterStation {
                                    name,
                                    distance_km: (dist * 10.0).round() / 10.0,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                nearby.sort_by(|a, b| {
                    a.distance_km
                        .partial_cmp(&b.distance_km)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                nearby.truncate(5);
                nearby
            }
            Err(_) => vec![],
        },
        Err(_) => vec![],
    };

    let result = NearbyResult {
        location: loc,
        weather,
        water_stations,
    };

    // Always output as JSON for this cross-source command
    output.render_single(&result)?;

    Ok(())
}
