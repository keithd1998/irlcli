use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use irl_core::output::OutputConfig;

use crate::api::PropertyData;
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum PropertyCommands {
    /// Search property sales
    ///
    /// Search the Property Price Register database by county, year,
    /// price range, or address. Requires data to be loaded first
    /// using `irl property update`.
    ///
    /// Examples:
    ///   irl property search --county Dublin --year 2024
    ///   irl property search --min 200000 --max 400000
    ///   irl property search --address "main street"
    Search {
        /// Filter by county name
        #[arg(long)]
        county: Option<String>,
        /// Filter by year (e.g., 2024)
        #[arg(long)]
        year: Option<String>,
        /// Minimum price
        #[arg(long)]
        min: Option<f64>,
        /// Maximum price
        #[arg(long)]
        max: Option<f64>,
        /// Search address (case-insensitive substring)
        #[arg(long)]
        address: Option<String>,
    },

    /// Property price statistics
    ///
    /// Shows aggregate statistics (average, median, min, max) for
    /// property sales, optionally filtered by county and year.
    /// Use --compare to show two years side by side.
    ///
    /// Examples:
    ///   irl property stats --county Dublin
    ///   irl property stats --county Dublin --year 2024
    ///   irl property stats --county Dublin --year 2024 --compare 2023
    Stats {
        /// Filter by county name
        #[arg(long)]
        county: Option<String>,
        /// Year to analyse
        #[arg(long)]
        year: Option<String>,
        /// Compare with another year
        #[arg(long)]
        compare: Option<String>,
    },

    /// Year-over-year property price trends
    ///
    /// Shows average and median prices by year with percentage changes.
    ///
    /// Examples:
    ///   irl property trends --county Dublin
    ///   irl property trends --county Dublin --from 2020 --to 2024
    Trends {
        /// Filter by county name
        #[arg(long)]
        county: Option<String>,
        /// Start year
        #[arg(long)]
        from: Option<String>,
        /// End year
        #[arg(long)]
        to: Option<String>,
    },

    /// Download/refresh local property data
    ///
    /// The PSRA Property Price Register requires a form-based download.
    /// This command explains how to download the CSV and import it.
    ///
    /// Alternatively, provide a path to a previously downloaded CSV file
    /// to import it directly.
    Update,
}

pub async fn handle_command(
    cmd: &PropertyCommands,
    output: &OutputConfig,
    _verbose: bool,
    _quiet: bool,
    _no_cache: bool,
) -> Result<()> {
    match cmd {
        PropertyCommands::Search {
            county,
            year,
            min,
            max,
            address,
        } => {
            output.print_header("Property Price Register");

            if !PropertyData::is_loaded() {
                output.print_error(
                    "No property data loaded. Run 'irl property update' for instructions \
                     on how to download and import data.",
                );
                return Ok(());
            }

            let results = PropertyData::search(
                county.as_deref(),
                year.as_deref(),
                *min,
                *max,
                address.as_deref(),
            )?;

            let rows: Vec<PropertyRow> = results.iter().map(PropertyRow::from_sale).collect();
            output.print_info(&format!("{} properties found", rows.len()));
            output.render_full(&rows, &results)?;
        }

        PropertyCommands::Stats {
            county,
            year,
            compare,
        } => {
            output.print_header("Property Price Statistics");

            if !PropertyData::is_loaded() {
                output.print_error(
                    "No property data loaded. Run 'irl property update' for instructions \
                     on how to download and import data.",
                );
                return Ok(());
            }

            let stats = PropertyData::stats(county.as_deref(), year.as_deref())?;
            let label = build_label(county.as_deref(), year.as_deref());
            output.print_info(&label);
            let rows = stats.to_rows();
            output.render(&rows)?;

            if let Some(compare_year) = compare {
                output.print_info("");
                let compare_stats = PropertyData::stats(county.as_deref(), Some(compare_year))?;
                let compare_label = build_label(county.as_deref(), Some(compare_year));
                output.print_info(&compare_label);
                let compare_rows = compare_stats.to_rows();
                output.render(&compare_rows)?;
            }
        }

        PropertyCommands::Trends { county, from, to } => {
            output.print_header("Property Price Trends");

            if !PropertyData::is_loaded() {
                output.print_error(
                    "No property data loaded. Run 'irl property update' for instructions.",
                );
                return Ok(());
            }

            let yearly = PropertyData::trends(
                county.as_deref(),
                from.as_deref(),
                to.as_deref(),
            )?;

            if yearly.is_empty() {
                output.print_info("No data found for the given filters.");
                return Ok(());
            }

            let mut trend_rows: Vec<TrendYear> = Vec::new();
            for (i, (year, avg, median, count)) in yearly.iter().enumerate() {
                let yoy_change = if i > 0 {
                    let prev_median = yearly[i - 1].2;
                    if prev_median > 0.0 {
                        Some(((median - prev_median) / prev_median) * 100.0)
                    } else {
                        None
                    }
                } else {
                    None
                };

                trend_rows.push(TrendYear {
                    year: year.clone(),
                    average_price: format_price(*avg),
                    median_price: format_price(*median),
                    sales_count: *count,
                    yoy_change_pct: yoy_change.map(|c| format!("{:+.1}%", c)),
                });
            }

            let label = if let Some(c) = county {
                format!("{} — year-over-year", c)
            } else {
                "National — year-over-year".to_string()
            };
            output.print_info(&label);
            output.render_single(&trend_rows)?;
        }

        PropertyCommands::Update => {
            output.print_header("Property Price Register - Data Update");

            let csv_dir = irl_core::config::Config::data_dir();
            let csv_path = csv_dir.join("ppr.csv");

            if csv_path.exists() {
                output.print_info(&format!(
                    "Found existing CSV at {}. Importing...",
                    csv_path.display()
                ));
                let count = PropertyData::import_csv(csv_path.to_str().unwrap())?;
                output.print_info(&format!("Successfully imported {} property sales.", count));
            } else {
                let count = PropertyData::record_count().unwrap_or(0);
                if count > 0 {
                    output.print_info(&format!("Database contains {} records.", count));
                }

                eprintln!();
                eprintln!("To download Property Price Register data:");
                eprintln!();
                eprintln!("  1. Visit https://www.propertypriceregister.ie/");
                eprintln!("  2. Click 'Download Data' or navigate to the download section");
                eprintln!("  3. Select your date range and click 'Download'");
                eprintln!("  4. Save the CSV file to:");
                eprintln!("     {}", csv_path.display());
                eprintln!();
                eprintln!("  Then run 'irl property update' again to import it.");
                eprintln!();
                eprintln!("Alternatively, you can import any PSRA-format CSV directly:");
                eprintln!("  cp /path/to/downloaded.csv {}", csv_path.display());
                eprintln!("  irl property update");
            }
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct TrendYear {
    year: String,
    average_price: String,
    median_price: String,
    sales_count: u64,
    yoy_change_pct: Option<String>,
}

fn build_label(county: Option<&str>, year: Option<&str>) -> String {
    let mut label = "Statistics".to_string();
    if let Some(county) = county {
        label.push_str(&format!(" for {}", county));
    }
    if let Some(year) = year {
        label.push_str(&format!(" ({})", year));
    }
    label
}
