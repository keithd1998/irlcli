use anyhow::Result;
use clap::{Parser, Subcommand};

use irl_core::config::Config;
use irl_core::output::{self, OutputConfig, OutputFormat};
use irl_oireachtas::commands::OireachtasCommands;

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

    /// Met Éireann — weather forecasts, warnings, and station data
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

    /// Manage irl configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

// -- Stub subcommands for unimplemented modules --

#[derive(Subcommand)]
enum MetCommands {
    /// Get weather forecast for a location
    Forecast {
        #[arg(long)]
        lat: Option<f64>,
        #[arg(long)]
        lon: Option<f64>,
        #[arg(long)]
        location: Option<String>,
        #[arg(long)]
        hours: Option<u32>,
    },
    /// View current weather warnings
    Warnings,
    /// List Met Éireann stations
    Stations,
}

#[derive(Subcommand)]
enum CsoCommands {
    /// List available statistical tables
    Tables {
        #[arg(long)]
        search: Option<String>,
    },
    /// Show table metadata
    Info { table_code: String },
    /// Query table data
    Query {
        table_code: String,
        #[arg(long)]
        dimension: Vec<String>,
        #[arg(long)]
        last: Option<u32>,
    },
}

#[derive(Subcommand)]
enum TransportCommands {
    /// Next departures from a stop
    Departures {
        #[arg(long)]
        stop: String,
        #[arg(long)]
        route: Option<String>,
    },
    /// Live vehicle positions
    Vehicles {
        #[arg(long)]
        route: String,
    },
    /// Search stops by name
    Stops {
        #[arg(long)]
        search: String,
    },
    /// List routes
    Routes {
        #[arg(long)]
        operator: Option<String>,
    },
}

#[derive(Subcommand)]
enum CroCommands {
    /// Search companies by name
    Search {
        name: String,
        #[arg(long)]
        status: Option<String>,
    },
    /// Get company details
    Company { number: String },
    /// List company filings
    Filings {
        number: String,
        #[arg(long, name = "type")]
        filing_type: Option<String>,
    },
}

#[derive(Subcommand)]
enum PropertyCommands {
    /// Search property sales
    Search {
        #[arg(long)]
        county: Option<String>,
        #[arg(long)]
        year: Option<String>,
        #[arg(long)]
        min: Option<f64>,
        #[arg(long)]
        max: Option<f64>,
        #[arg(long)]
        address: Option<String>,
    },
    /// Property price statistics
    Stats {
        #[arg(long)]
        county: Option<String>,
        #[arg(long)]
        year: Option<String>,
        #[arg(long)]
        compare: Option<String>,
    },
    /// Download/refresh local property data
    Update,
}

#[derive(Subcommand)]
enum EpaCommands {
    /// Current air quality data
    AirQuality {
        #[arg(long)]
        station: Option<String>,
    },
    /// Water quality data
    WaterQuality {
        #[arg(long)]
        catchment: Option<String>,
    },
    /// Licensed facilities
    Facilities {
        #[arg(long)]
        county: Option<String>,
    },
    /// Emissions data
    Emissions {
        #[arg(long)]
        sector: Option<String>,
    },
}

#[derive(Subcommand)]
enum WaterCommands {
    /// List monitoring stations
    Stations {
        #[arg(long)]
        county: Option<String>,
    },
    /// Current water level at a station
    Level {
        station_id: String,
        #[arg(long)]
        history: Option<String>,
    },
    /// Stations with high water levels
    Alerts,
    /// Search stations by name
    Search { query: String },
}

#[derive(Subcommand)]
enum TailteCommands {
    /// Search valuations by address
    Search {
        #[arg(long)]
        address: String,
    },
    /// Get valuation details
    Property { property_number: String },
    /// List properties in rating authority area
    Area {
        #[arg(long)]
        rating_authority: String,
    },
    /// List property categories
    Categories,
}

#[derive(Subcommand)]
enum GeoCommands {
    /// Fetch boundary data
    Boundaries {
        #[arg(long, name = "type")]
        boundary_type: String,
    },
    /// Find what boundary contains a point
    Search {
        #[arg(long)]
        lat: f64,
        #[arg(long)]
        lon: f64,
    },
    /// List available spatial datasets
    Datasets,
    /// Download a dataset
    Fetch {
        dataset_id: String,
        #[arg(long, default_value = "geojson")]
        format: String,
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

        // Stub handlers for unimplemented modules
        Commands::Met { .. } => output::coming_soon("met"),
        Commands::Cso { .. } => output::coming_soon("cso"),
        Commands::Transport { .. } => output::coming_soon("transport"),
        Commands::Cro { .. } => output::coming_soon("cro"),
        Commands::Property { .. } => output::coming_soon("property"),
        Commands::Epa { .. } => output::coming_soon("epa"),
        Commands::Water { .. } => output::coming_soon("water"),
        Commands::Tailte { .. } => output::coming_soon("tailte"),
        Commands::Geo { .. } => output::coming_soon("geo"),
    }

    Ok(())
}
