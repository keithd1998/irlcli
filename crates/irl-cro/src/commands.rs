use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::CroApi;
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum CroCommands {
    /// Search companies by name
    ///
    /// Searches the Companies Registration Office for companies matching
    /// the given name. Optionally filter by company status.
    ///
    /// Note: The CRO API may be unavailable or behind Cloudflare protection.
    /// If the request fails, try again later or use the CRO website directly
    /// at https://core.cro.ie
    ///
    /// Examples:
    ///   irl cro search "Acme"
    ///   irl cro search "Acme" --status Normal
    Search {
        /// Company name to search for
        name: String,
        /// Filter by status (e.g., Normal, Dissolved, Strike-Off Listed)
        #[arg(long)]
        status: Option<String>,
    },

    /// Get company details by number
    ///
    /// Retrieves detailed information about a specific company
    /// including directors, registered address, and activity.
    ///
    /// Examples:
    ///   irl cro company 123456
    Company {
        /// Company registration number
        number: String,
    },

    /// List company filings
    ///
    /// Shows filings (annual returns, change of directors, etc.)
    /// for a given company number.
    ///
    /// Examples:
    ///   irl cro filings 123456
    ///   irl cro filings 123456 --type B1
    Filings {
        /// Company registration number
        number: String,
        /// Filter by filing type (e.g., B1, B2, C1)
        #[arg(long, name = "type")]
        filing_type: Option<String>,
    },
}

pub async fn handle_command(
    cmd: &CroCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = CroApi::new(verbose, quiet, no_cache)?;

    match cmd {
        CroCommands::Search { name, status } => {
            output.print_header("CRO Company Search");
            match api.search_companies(name, status.as_deref()).await {
                Ok(response) => {
                    let companies = response.companies.unwrap_or_default();
                    let rows: Vec<CompanyRow> =
                        companies.iter().map(CompanyRow::from_result).collect();
                    output.print_info(&format!("{} companies found", rows.len()));
                    output.render_full(&rows, &companies)?;
                }
                Err(e) => {
                    output.print_error(&format!(
                        "CRO API request failed: {}\n\n\
                         The CRO API at core.cro.ie may be unavailable or behind \
                         Cloudflare protection. You can search directly at:\n  \
                         https://core.cro.ie/",
                        e
                    ));
                }
            }
        }

        CroCommands::Company { number } => {
            output.print_header("Company Details");
            match api.get_company(number).await {
                Ok(detail) => {
                    let rows = CompanyDetailRow::from_detail(&detail);
                    output.render(&rows)?;
                }
                Err(e) => {
                    output.print_error(&format!(
                        "CRO API request failed: {}\n\n\
                         The CRO API at core.cro.ie may be unavailable or behind \
                         Cloudflare protection. You can look up company {} directly at:\n  \
                         https://core.cro.ie/",
                        e, number
                    ));
                }
            }
        }

        CroCommands::Filings {
            number,
            filing_type,
        } => {
            output.print_header("Company Filings");
            match api.get_filings(number, filing_type.as_deref()).await {
                Ok(response) => {
                    let filings = response.filings.unwrap_or_default();
                    let rows: Vec<FilingRow> = filings.iter().map(FilingRow::from_filing).collect();
                    output.print_info(&format!(
                        "{} filings found for company {}",
                        rows.len(),
                        number
                    ));
                    output.render_full(&rows, &filings)?;
                }
                Err(e) => {
                    output.print_error(&format!(
                        "CRO API request failed: {}\n\n\
                         The CRO API at core.cro.ie may be unavailable or behind \
                         Cloudflare protection. You can view filings for company {} at:\n  \
                         https://core.cro.ie/",
                        e, number
                    ));
                }
            }
        }
    }

    Ok(())
}
