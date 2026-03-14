use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use irl_core::fuzzy;
use irl_core::output::OutputConfig;

use crate::api::OireachtasApi;
use crate::constituencies;
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

    /// TD profile — unified view of a member's activity
    ///
    /// Shows a TD's party, constituency, recent questions, and sponsored bills
    /// in a single JSON output. Supports fuzzy name matching.
    ///
    /// Examples:
    ///   irl oireachtas td --name "Paschal Donohoe"
    ///   irl oireachtas td --name "mcdonald"
    Td {
        /// TD name (fuzzy matching supported)
        #[arg(long)]
        name: String,
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

            // Resolve constituency name (handles historical redistricting)
            let resolved_constituency = if let Some(const_input) = constituency {
                let (resolved, was_historical) = constituencies::resolve_constituency(const_input);
                if was_historical && !resolved.is_empty() {
                    output.print_info(&format!(
                        "Note: \"{}\" was redistricted. Searching current constituencies: {}",
                        const_input,
                        resolved.join(", ")
                    ));
                }
                if resolved.is_empty() {
                    // Try fuzzy matching
                    let suggestions = fuzzy::fuzzy_match(
                        const_input,
                        constituencies::CURRENT_CONSTITUENCIES,
                        0.75,
                    );
                    if !suggestions.is_empty() {
                        output.print_info(&format!(
                            "No constituency matching \"{}\". {}",
                            const_input,
                            fuzzy::format_suggestions(&suggestions)
                        ));
                    } else {
                        output.print_info(&format!(
                            "No constituency matching \"{}\". Use `irl oireachtas members` to see all constituencies.",
                            const_input
                        ));
                    }
                }
                Some(resolved)
            } else {
                None
            };

            let has_filter = party.is_some() || constituency.is_some();

            let mut results = if has_filter {
                let all = paginate_all(|limit, skip| api.list_members(Some("dail"), limit, skip)).await?;
                all.into_iter()
                    .filter(|r| {
                        // Party filter
                        if let Some(party_filter) = party {
                            let member_party = r.member.memberships.as_ref()
                                .and_then(|ms| ms.last())
                                .and_then(|mw| mw.membership.parties.as_ref())
                                .and_then(|ps| ps.last())
                                .and_then(|pw| pw.party.show_as.as_deref())
                                .unwrap_or("");
                            if !member_party.to_lowercase().contains(&party_filter.to_lowercase()) {
                                return false;
                            }
                        }
                        // Constituency filter (using resolved names)
                        if let Some(ref resolved) = resolved_constituency {
                            if resolved.is_empty() {
                                return false; // No valid constituency to match against
                            }
                            let member_const = r.member.memberships.as_ref()
                                .and_then(|ms| ms.last())
                                .and_then(|mw| mw.membership.represents.as_ref())
                                .and_then(|rs| rs.last())
                                .and_then(|rw| rw.represent.show_as.as_deref())
                                .unwrap_or("");
                            let member_lower = member_const.to_lowercase();
                            if !resolved.iter().any(|c| member_lower.contains(&c.to_lowercase())) {
                                return false;
                            }
                        }
                        true
                    })
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

        OireachtasCommands::Td { name } => {
            output.print_header("TD Profile");

            // Find the TD by name (fuzzy matching)
            let all_members = paginate_all(|limit, skip| api.list_members(Some("dail"), limit, skip)).await?;

            // Collect all member names for fuzzy matching
            let member_names: Vec<String> = all_members
                .iter()
                .filter_map(|r| r.member.full_name.clone())
                .collect();
            let name_refs: Vec<&str> = member_names.iter().map(|s| s.as_str()).collect();

            // Find exact or fuzzy match
            let name_lower = name.to_lowercase();
            let matched_name = member_names
                .iter()
                .find(|n| n.to_lowercase() == name_lower)
                .or_else(|| {
                    member_names
                        .iter()
                        .find(|n| n.to_lowercase().contains(&name_lower))
                })
                .cloned()
                .or_else(|| {
                    let matches = fuzzy::fuzzy_match(name, &name_refs, 0.75);
                    matches.first().map(|m| m.candidate.clone())
                });

            let matched_name = match matched_name {
                Some(n) => n,
                None => {
                    let suggestions = fuzzy::fuzzy_match(name, &name_refs, 0.6);
                    if suggestions.is_empty() {
                        output.print_error(&format!(
                            "No TD found matching '{}'. Use `irl oireachtas members` to see all TDs.",
                            name
                        ));
                    } else {
                        output.print_error(&format!(
                            "No TD found matching '{}'. {}",
                            name,
                            fuzzy::format_suggestions(&suggestions)
                        ));
                    }
                    return Ok(());
                }
            };

            // Get member info
            let member_result = all_members
                .iter()
                .rev()
                .find(|r| r.member.full_name.as_deref() == Some(&matched_name));

            let (member_party, member_constituency, member_house) = member_result
                .map(|r| {
                    let ms = r.member.memberships.as_ref().and_then(|ms| ms.last());
                    let party = ms
                        .and_then(|mw| mw.membership.parties.as_ref())
                        .and_then(|ps| ps.last())
                        .and_then(|pw| pw.party.show_as.clone())
                        .unwrap_or_default();
                    let constituency = ms
                        .and_then(|mw| mw.membership.represents.as_ref())
                        .and_then(|rs| rs.last())
                        .and_then(|rw| rw.represent.show_as.clone())
                        .unwrap_or_default();
                    let house = ms
                        .and_then(|mw| mw.membership.house.as_ref())
                        .and_then(|h| h.show_as.clone())
                        .unwrap_or_default();
                    (party, constituency, house)
                })
                .unwrap_or_default();

            // Fetch questions by this member
            let all_questions = paginate_all(|limit, skip| api.list_questions(limit, skip)).await?;
            let member_questions: Vec<TdQuestion> = all_questions
                .iter()
                .filter(|r| {
                    r.question
                        .by
                        .as_ref()
                        .and_then(|b| b.show_as.as_ref())
                        .map(|n| n.to_lowercase().contains(&matched_name.to_lowercase()))
                        .unwrap_or(false)
                })
                .take(10)
                .map(|r| TdQuestion {
                    date: r.question.date.clone().unwrap_or_default(),
                    question_type: r.question.question_type.clone().unwrap_or_default(),
                    topic: r.question.show_as.clone().unwrap_or_default(),
                })
                .collect();

            // Fetch bills sponsored by this member
            let all_bills = paginate_all(|limit, skip| api.list_legislation(limit, skip)).await?;
            let sponsored_bills: Vec<TdBill> = all_bills
                .iter()
                .filter(|r| {
                    r.bill
                        .sponsors
                        .as_ref()
                        .map(|sponsors| {
                            sponsors.iter().any(|sw| {
                                sw.sponsor
                                    .by
                                    .as_ref()
                                    .and_then(|by| by.show_as.as_ref())
                                    .map(|n| n.to_lowercase().contains(&matched_name.to_lowercase()))
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                })
                .map(|r| TdBill {
                    title: r.bill.short_title_en.clone().unwrap_or_default(),
                    year: r.bill.bill_year.clone().unwrap_or_default(),
                    status: r.bill.status.clone().unwrap_or_default(),
                })
                .collect();

            let profile = TdProfile {
                name: matched_name,
                party: member_party,
                constituency: member_constituency,
                house: member_house,
                recent_questions: member_questions,
                sponsored_bills,
            };

            output.render_single(&profile)?;
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct TdProfile {
    name: String,
    party: String,
    constituency: String,
    house: String,
    recent_questions: Vec<TdQuestion>,
    sponsored_bills: Vec<TdBill>,
}

#[derive(Debug, Serialize)]
struct TdQuestion {
    date: String,
    question_type: String,
    topic: String,
}

#[derive(Debug, Serialize)]
struct TdBill {
    title: String,
    year: String,
    status: String,
}
