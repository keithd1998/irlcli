use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::OireachtasApi;
use crate::models::*;

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
        /// Maximum number of results
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
        /// Maximum number of results
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
        /// Maximum number of results
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
            let skip = page * limit;
            let response = api.list_members(Some("dail"), *limit, skip).await?;

            let mut rows: Vec<MemberRow> = response
                .results
                .iter()
                .map(MemberRow::from_result)
                .collect();

            // Client-side filtering
            if let Some(party_filter) = party {
                let filter = party_filter.to_lowercase();
                rows.retain(|r| r.party.to_lowercase().contains(&filter));
            }
            if let Some(const_filter) = constituency {
                let filter = const_filter.to_lowercase();
                rows.retain(|r| r.constituency.to_lowercase().contains(&filter));
            }

            output.print_info(&format!("{} members found", rows.len()));
            output.render(&rows)?;
        }

        OireachtasCommands::Legislation {
            search,
            status,
            year,
            limit,
            page,
        } => {
            output.print_header("Legislation");
            let skip = page * limit;
            let response = api.list_legislation(*limit, skip).await?;

            let mut rows: Vec<BillRow> =
                response.results.iter().map(BillRow::from_result).collect();

            if let Some(search_term) = search {
                let filter = search_term.to_lowercase();
                rows.retain(|r| r.title.to_lowercase().contains(&filter));
            }
            if let Some(status_filter) = status {
                let filter = status_filter.to_lowercase();
                rows.retain(|r| r.status.to_lowercase().contains(&filter));
            }
            if let Some(year_filter) = year {
                rows.retain(|r| r.year == *year_filter);
            }

            output.print_info(&format!("{} bills found", rows.len()));
            output.render(&rows)?;
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

            let rows: Vec<DebateRow> = response
                .results
                .iter()
                .map(DebateRow::from_result)
                .collect();

            output.print_info(&format!("{} debate records found", rows.len()));
            output.render(&rows)?;
        }

        OireachtasCommands::Questions {
            member,
            limit,
            page,
        } => {
            output.print_header("Parliamentary Questions");
            let skip = page * limit;
            let response = api.list_questions(*limit, skip).await?;

            let mut rows: Vec<QuestionRow> = response
                .results
                .iter()
                .map(QuestionRow::from_result)
                .collect();

            if let Some(member_filter) = member {
                let filter = member_filter.to_lowercase();
                rows.retain(|r| r.asked_by.to_lowercase().contains(&filter));
            }

            output.print_info(&format!("{} questions found", rows.len()));
            output.render(&rows)?;
        }

        OireachtasCommands::Divisions {
            recent: _,
            limit,
            page,
        } => {
            output.print_header("Divisions (Votes)");
            let skip = page * limit;
            let response = api.list_divisions(*limit, skip).await?;

            let rows: Vec<DivisionRow> = response
                .results
                .iter()
                .map(DivisionRow::from_result)
                .collect();

            output.print_info(&format!("{} divisions found", rows.len()));
            output.render(&rows)?;
        }
    }

    Ok(())
}
