use anyhow::Result;
use clap::{Parser, Subcommand};

use irl_core::config::Config;
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
    }

    Ok(())
}
