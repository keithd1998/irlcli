use anyhow::Result;
use clap::Subcommand;

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
    }

    Ok(())
}
