use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use irl_core::output::OutputConfig;

use crate::api::CsoApi;
use crate::jsonstat::{unpack_dataset, UnpackOptions};

#[derive(Debug, Subcommand)]
pub enum CsoCommands {
    /// List available CSO statistical tables
    ///
    /// Fetches the full PxStat catalogue (~4000 tables).
    /// Use --search to filter by keyword.
    ///
    /// Examples:
    ///   irl cso tables
    ///   irl cso tables --search "house prices"
    ///   irl cso tables --search "population"
    Tables {
        /// Search tables by keyword (case-insensitive, matches title and code)
        #[arg(long)]
        search: Option<String>,
    },

    /// Show metadata for a CSO table
    ///
    /// Displays the table title, last updated date, dimensions, and sample values.
    ///
    /// Examples:
    ///   irl cso info CPM01
    ///   irl cso info B0101
    Info {
        /// Table code (e.g., CPM01, B0101)
        table_code: String,
    },

    /// Query data from a CSO table
    ///
    /// Fetches and displays data from a PxStat table.
    /// Filter by dimension values or limit to recent time periods.
    ///
    /// Examples:
    ///   irl cso query CPM01
    ///   irl cso query CPM01 --last 12
    ///   irl cso query CPM01 --dimension "Year=2024"
    ///   irl cso query HPM09 --dimension "County=Dublin" --last 6
    Query {
        /// Table code (e.g., CPM01, B0101)
        table_code: String,
        /// Filter by dimension value (format: "DimensionLabel=Value"), repeatable
        #[arg(long)]
        dimension: Vec<String>,
        /// Show only the last N time periods
        #[arg(long)]
        last: Option<u32>,
    },

    /// County statistical profile
    ///
    /// Returns key CSO statistics for a county: population, house prices.
    /// Queries multiple CSO tables automatically.
    ///
    /// Examples:
    ///   irl cso local --county Dublin
    ///   irl cso local --county Cork
    Local {
        /// County name (e.g., Dublin, Cork, Galway)
        #[arg(long)]
        county: String,
    },
}

pub async fn handle_command(
    cmd: &CsoCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = CsoApi::new(verbose, quiet, no_cache)?;

    match cmd {
        CsoCommands::Tables { search } => {
            output.print_header("CSO PxStat Tables");
            let catalog = api.fetch_catalog().await?;

            let rows = if let Some(query) = search {
                CsoApi::search_catalog(&catalog.link.item, query)
            } else {
                CsoApi::catalog_to_rows(&catalog.link.item)
            };

            output.print_info(&format!("{} tables found", rows.len()));
            output.render(&rows)?;
        }

        CsoCommands::Info { table_code } => {
            let code = table_code.to_uppercase();
            output.print_header(&format!("CSO Table: {}", code));
            let dataset = api.fetch_dataset(&code).await?;
            let info = CsoApi::extract_table_info(&code, &dataset);

            output.print_info(&format!("Title: {}", info.title));
            output.print_info(&format!("Last Updated: {}", info.updated));

            if !info.notes.is_empty() {
                output.print_info("");
                for note in &info.notes {
                    output.print_info(&format!("Note: {}", note));
                }
            }

            output.print_info("");
            output.print_header("Dimensions");
            output.render(&info.dimensions)?;
        }

        CsoCommands::Query {
            table_code,
            dimension,
            last,
        } => {
            let code = table_code.to_uppercase();
            output.print_header(&format!("CSO Data: {}", code));
            let dataset = api.fetch_dataset(&code).await?;

            let title = dataset.label.as_deref().unwrap_or(&code);
            output.print_info(&format!("Table: {}", title));

            let options = UnpackOptions::default()
                .with_dimension_filters(dimension)
                .with_last_n(*last);

            let rows = unpack_dataset(&dataset, &options);

            output.print_info(&format!("{} data points", rows.len()));
            output.render(&rows)?;
        }

        CsoCommands::Local { county } => {
            output.print_header(&format!("County Profile: {}", county));

            let mut profile = CountyProfile {
                county: county.clone(),
                population: None,
                house_prices: None,
            };

            // G0420: Population per County (Census 2022)
            if let Ok(dataset) = api.fetch_dataset("G0420").await {
                let county_key = format!("Co. {}", county);
                let options = UnpackOptions::default()
                    .with_dimension_filters(&[format!("County={}", county_key)]);
                let rows = unpack_dataset(&dataset, &options);
                if let Some(row) = rows.first() {
                    profile.population = Some(PopulationData {
                        year: row.period.clone(),
                        count: row.value.clone(),
                    });
                }
            }

            // HSA06: Average Price of Houses
            if let Ok(dataset) = api.fetch_dataset("HSA06").await {
                // HSA06 uses city names: Dublin, Cork, Galway, Limerick, Waterford, Other, National
                let area = county.to_string();
                let options = UnpackOptions::default()
                    .with_dimension_filters(&[format!("Area={}", area)])
                    .with_last_n(Some(1));
                let rows = unpack_dataset(&dataset, &options);
                if !rows.is_empty() {
                    let prices: Vec<HousePriceData> = rows
                        .iter()
                        .map(|r| HousePriceData {
                            year: r.period.clone(),
                            category: r.category.clone(),
                            price: r.value.clone(),
                        })
                        .collect();
                    profile.house_prices = Some(prices);
                }
            }

            output.render_single(&profile)?;
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct CountyProfile {
    county: String,
    population: Option<PopulationData>,
    house_prices: Option<Vec<HousePriceData>>,
}

#[derive(Debug, Serialize)]
struct PopulationData {
    year: String,
    count: String,
}

#[derive(Debug, Serialize)]
struct HousePriceData {
    year: String,
    category: String,
    price: String,
}
