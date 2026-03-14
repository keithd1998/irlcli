use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::TailteApi;
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum TailteCommands {
    /// Search valuations by address
    ///
    /// Searches the Tailte Éireann valuation database by address.
    ///
    /// Examples:
    ///   irl tailte search --address "Main Street Dublin"
    Search {
        /// Address to search for
        #[arg(long)]
        address: String,
    },

    /// Get valuation details for a property
    ///
    /// Retrieves detailed valuation information for a specific property.
    ///
    /// Examples:
    ///   irl tailte property 12345
    Property {
        /// Property number
        property_number: String,
    },

    /// List properties in a rating authority area
    ///
    /// Lists all valued properties within a rating authority.
    ///
    /// Examples:
    ///   irl tailte area --rating-authority "Dublin City Council"
    Area {
        /// Rating authority name
        #[arg(long)]
        rating_authority: String,
    },

    /// List property categories
    ///
    /// Shows all available property valuation categories.
    ///
    /// Examples:
    ///   irl tailte categories
    Categories,
}

pub async fn handle_command(
    cmd: &TailteCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = TailteApi::new(verbose, quiet, no_cache)?;

    match cmd {
        TailteCommands::Search { address } => {
            output.print_header("Tailte Éireann Valuation Search");
            match api.search_valuations(address).await {
                Ok(response) => {
                    let results = response.results.unwrap_or_default();
                    let rows: Vec<ValuationRow> =
                        results.iter().map(ValuationRow::from_result).collect();
                    output.print_info(&format!("{} valuations found", rows.len()));
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "Tailte Éireann API currently unavailable. \
                         Visit https://www.tailte.ie for valuation data.",
                    );
                }
            }
        }

        TailteCommands::Property { property_number } => {
            output.print_header("Property Valuation Details");
            match api.get_property(property_number).await {
                Ok(valuation) => {
                    let rows = PropertyDetailRow::from_valuation(&valuation);
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "Tailte Éireann API currently unavailable. \
                         Visit https://www.tailte.ie for valuation data.",
                    );
                }
            }
        }

        TailteCommands::Area { rating_authority } => {
            output.print_header("Valuations by Rating Authority");
            match api.get_area(rating_authority).await {
                Ok(response) => {
                    let properties = response.properties.unwrap_or_default();
                    let rows: Vec<ValuationRow> =
                        properties.iter().map(ValuationRow::from_result).collect();
                    output.print_info(&format!(
                        "{} properties in {}",
                        rows.len(),
                        rating_authority
                    ));
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "Tailte Éireann API currently unavailable. \
                         Visit https://www.tailte.ie for valuation data.",
                    );
                }
            }
        }

        TailteCommands::Categories => {
            output.print_header("Property Categories");
            match api.get_categories().await {
                Ok(response) => {
                    let categories = response.categories.unwrap_or_default();
                    let rows: Vec<CategoryRow> =
                        categories.iter().map(CategoryRow::from_category).collect();
                    output.print_info(&format!("{} categories", rows.len()));
                    output.render(&rows)?;
                }
                Err(_) => {
                    output.print_info(
                        "Tailte Éireann API currently unavailable. \
                         Visit https://www.tailte.ie for valuation data.",
                    );
                }
            }
        }
    }

    Ok(())
}
