use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::OireachtasApi;
use crate::models::*;

/// Maximum number of results to fetch when auto-paginating for filtered queries.
const MAX_AUTO_PAGINATE: u32 = 500;

/// Page size when auto-paginating.
const AUTO_PAGE_SIZE: u32 = 50;

#[derive(Debug, Subcommand)]
pub enum OireachtasCommands {
    /// List current TDs and Senators
    ///
    /// Fetches member data from the Houses of the Oireachtas API.
    /// Filter by party or constituency.
    ///
    /// Examples:
    ///   irl oireachtas members
    ///   irl oireachtas members --party "Sinn Féin"
    ///   irl oireachtas members --constituency "Dublin Central"
    Members {
        /// Filter by party name (case-insensitive substring match)
        #[arg(long)]
        party: Option<String>,
        /// Filter by constituency name (case-insensitive substring match)
        #[arg(long)]
        constituency: Option<String>,
        /// Maximum number of results to display
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Page number (0-based)
        #[arg(long, default_value = "0")]
        page: u32,
    },

    /// List recent legislation (Bills)
    ///
    /// Fetches legislation data from the Oireachtas API.
    /// Search by keyword or filter by status and year.
    ///
    /// Examples:
    ///   irl oireachtas legislation
    ///   irl oireachtas legislation --search "planning"
    ///   irl oireachtas legislation --status enacted --year 2025
    Legislation {
        /// Search bills by keyword (case-insensitive)
        #[arg(long)]
        search: Option<String>,
        /// Filter by status (e.g., "Current", "Enacted", "Withdrawn", "Rejected", "Lapsed")
        #[arg(long)]
        status: Option<String>,
        /// Filter by year
        #[arg(long)]
        year: Option<String>,
        /// Maximum number of results to display
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Page number (0-based)
        #[arg(long, default_value = "0")]
        page: u32,
    },

    /// View Dáil and Seanad debates
    ///
    /// Lists debate records by date and chamber.
    ///
    /// Examples:
    ///   irl oireachtas debates --date 2025-03-01
    ///   irl oireachtas debates --date 2025-03-01 --chamber dail
    Debates {
        /// Date to fetch debates for (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// Chamber: "dail" or "seanad"
        #[arg(long)]
        chamber: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Browse parliamentary questions
    ///
    /// Lists questions asked by TDs and Senators.
    ///
    /// Examples:
    ///   irl oireachtas questions
    ///   irl oireachtas questions --member "Mary Lou McDonald"
    Questions {
        /// Filter by member name (case-insensitive substring match)
        #[arg(long)]
        member: Option<String>,
        /// Maximum number of results to display
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Page number (0-based)
        #[arg(long, default_value = "0")]
        page: u32,
    },

    /// View recent Dáil/Seanad votes (divisions)
    ///
    /// Shows voting records from the Oireachtas.
    ///
    /// Examples:
    ///   irl oireachtas divisions
    ///   irl oireachtas divisions --recent
    Divisions {
        /// Show only the most recent divisions
        #[arg(long)]
        recent: bool,
        /// Maximum number of results
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Page number (0-based)
        #[arg(long, default_value = "0")]
        page: u32,
    },
}

/// Auto-paginate through all API pages to collect complete results.
/// This ensures filtered queries don't miss results that fall on later pages.
async fn paginate_all<T, F, Fut>(fetch_page: F) -> Result<Vec<T>>
where
    F: Fn(u32, u32) -> Fut,
    Fut: std::future::Future<Output = Result<ApiResponse<T>, irl_core::error::IrlError>>,
    T: serde::Serialize,
{
    let mut all_results = Vec::new();
    let mut skip = 0u32;
    loop {
        let response = fetch_page(AUTO_PAGE_SIZE, skip).await?;
        let count = response.results.len() as u32;
        all_results.extend(response.results);
        if count < AUTO_PAGE_SIZE || all_results.len() as u32 >= MAX_AUTO_PAGINATE {
            break;
        }
        skip += AUTO_PAGE_SIZE;
    }
    Ok(all_results)
}

/// Helper to filter MemberResult by party/constituency using raw API data.
fn member_matches(result: &MemberResult, party: &Option<String>, constituency: &Option<String>) -> bool {
    let membership = result
        .member
        .memberships
        .as_ref()
        .and_then(|ms| ms.last());

    if let Some(party_filter) = party {
        let member_party = membership
            .and_then(|mw| mw.membership.parties.as_ref())
            .and_then(|ps| ps.last())
            .and_then(|pw| pw.party.show_as.as_deref())
            .unwrap_or("");
        if !member_party.to_lowercase().contains(&party_filter.to_lowercase()) {
            return false;
        }
    }
    if let Some(const_filter) = constituency {
        let member_const = membership
            .and_then(|mw| mw.membership.represents.as_ref())
            .and_then(|rs| rs.last())
            .and_then(|rw| rw.represent.show_as.as_deref())
            .unwrap_or("");
        if !member_const.to_lowercase().contains(&const_filter.to_lowercase()) {
            return false;
        }
    }
    true
}

/// Helper to filter BillResult using raw API data.
fn bill_matches(
    result: &BillResult,
    search: &Option<String>,
    status: &Option<String>,
    year: &Option<String>,
) -> bool {
    if let Some(search_term) = search {
        let title = result.bill.short_title_en.as_deref().unwrap_or("");
        if !title.to_lowercase().contains(&search_term.to_lowercase()) {
            return false;
        }
    }
    if let Some(status_filter) = status {
        let bill_status = result.bill.status.as_deref().unwrap_or("");
        if !bill_status.to_lowercase().contains(&status_filter.to_lowercase()) {
            return false;
        }
    }
    if let Some(year_filter) = year {
        let bill_year = result.bill.bill_year.as_deref().unwrap_or("");
        if bill_year != year_filter {
            return false;
        }
    }
    true
}

pub async fn handle_command(
    cmd: &OireachtasCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = OireachtasApi::new(verbose, quiet, no_cache)?;

    match cmd {
        OireachtasCommands::Members {
            party,
            constituency,
            limit,
            page,
        } => {
            output.print_header("Members of the Oireachtas");

            let has_filter = party.is_some() || constituency.is_some();

            let mut results = if has_filter {
                // Auto-paginate to find all matching members
                let all = paginate_all(|limit, skip| api.list_members(Some("dail"), limit, skip)).await?;
                all.into_iter()
                    .filter(|r| member_matches(r, party, constituency))
                    .collect::<Vec<_>>()
            } else {
                let skip = page * limit;
                let response = api.list_members(Some("dail"), *limit, skip).await?;
                response.results
            };

            results.truncate(*limit as usize);
            let display_rows: Vec<MemberRow> =
                results.iter().map(MemberRow::from_result).collect();

            output.print_info(&format!("{} members found", display_rows.len()));
            output.render_full(&display_rows, &results)?;
        }

        OireachtasCommands::Legislation {
            search,
            status,
            year,
            limit,
            page,
        } => {
            output.print_header("Legislation");

            let has_filter = search.is_some() || status.is_some() || year.is_some();

            let mut results = if has_filter {
                let all = paginate_all(|limit, skip| api.list_legislation(limit, skip)).await?;
                all.into_iter()
                    .filter(|r| bill_matches(r, search, status, year))
                    .collect::<Vec<_>>()
            } else {
                let skip = page * limit;
                let response = api.list_legislation(*limit, skip).await?;
                response.results
            };

            results.truncate(*limit as usize);
            let display_rows: Vec<BillRow> =
                results.iter().map(BillRow::from_result).collect();

            output.print_info(&format!("{} bills found", display_rows.len()));
            output.render_full(&display_rows, &results)?;
        }

        OireachtasCommands::Debates {
            date,
            chamber,
            limit,
        } => {
            output.print_header("Debates");
            let (date_start, date_end) = if let Some(d) = date {
                (Some(d.as_str()), Some(d.as_str()))
            } else {
                (None, None)
            };
            let response = api
                .list_debates(chamber.as_deref(), date_start, date_end, *limit, 0)
                .await?;

            let display_rows: Vec<DebateRow> = response
                .results
                .iter()
                .map(DebateRow::from_result)
                .collect();

            output.print_info(&format!("{} debate records found", display_rows.len()));
            output.render_full(&display_rows, &response.results)?;
        }

        OireachtasCommands::Questions {
            member,
            limit,
            page,
        } => {
            output.print_header("Parliamentary Questions");

            let mut results = if member.is_some() {
                // Auto-paginate to find all questions by this member
                let all = paginate_all(|limit, skip| api.list_questions(limit, skip)).await?;
                let member_filter = member.as_ref().unwrap().to_lowercase();
                all.into_iter()
                    .filter(|r| {
                        r.question
                            .by
                            .as_ref()
                            .and_then(|b| b.show_as.as_ref())
                            .map(|name| name.to_lowercase().contains(&member_filter))
                            .unwrap_or(false)
                    })
                    .collect::<Vec<_>>()
            } else {
                let skip = page * limit;
                let response = api.list_questions(*limit, skip).await?;
                response.results
            };

            results.truncate(*limit as usize);
            let display_rows: Vec<QuestionRow> =
                results.iter().map(QuestionRow::from_result).collect();

            output.print_info(&format!("{} questions found", display_rows.len()));
            output.render_full(&display_rows, &results)?;
        }

        OireachtasCommands::Divisions {
            recent: _,
            limit,
            page,
        } => {
            output.print_header("Divisions (Votes)");
            let skip = page * limit;
            let response = api.list_divisions(*limit, skip).await?;

            let display_rows: Vec<DivisionRow> = response
                .results
                .iter()
                .map(DivisionRow::from_result)
                .collect();

            output.print_info(&format!("{} divisions found", display_rows.len()));
            output.render_full(&display_rows, &response.results)?;
        }
    }

    Ok(())
}
