use serde::{Deserialize, Serialize};
use tabled::Tabled;

// -- Generic API response wrapper --

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub head: Head,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Head {
    pub counts: serde_json::Value,
}

// -- Members --

#[derive(Debug, Deserialize, Serialize)]
pub struct MemberResult {
    pub member: MemberData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemberData {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    pub gender: Option<String>,
    #[serde(rename = "memberCode")]
    pub member_code: Option<String>,
    #[serde(rename = "pId")]
    pub pid: Option<String>,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub uri: Option<String>,
    #[serde(rename = "dateOfDeath")]
    pub date_of_death: Option<String>,
    pub memberships: Option<Vec<MembershipWrapper>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MembershipWrapper {
    pub membership: Membership,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Membership {
    pub house: Option<HouseInfo>,
    pub represents: Option<Vec<RepresentWrapper>>,
    pub parties: Option<Vec<PartyWrapper>>,
    #[serde(rename = "dateRange")]
    pub date_range: Option<DateRange>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HouseInfo {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    #[serde(rename = "houseCode")]
    pub house_code: Option<String>,
    #[serde(rename = "houseNo")]
    pub house_no: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepresentWrapper {
    pub represent: Represent,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Represent {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    #[serde(rename = "representCode")]
    pub represent_code: Option<String>,
    #[serde(rename = "representType")]
    pub represent_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PartyWrapper {
    pub party: Party,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Party {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    #[serde(rename = "partyCode")]
    pub party_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DateRange {
    pub start: Option<String>,
    pub end: Option<String>,
}

// -- Display structs for table output --

#[derive(Debug, Tabled, Serialize)]
pub struct MemberRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Party")]
    pub party: String,
    #[tabled(rename = "Constituency")]
    pub constituency: String,
    #[tabled(rename = "House")]
    pub house: String,
}

impl MemberRow {
    pub fn from_result(result: &MemberResult) -> Self {
        let m = &result.member;
        let name = m
            .full_name
            .clone()
            .or(m.show_as.clone())
            .unwrap_or_default();

        let (party, constituency, house) = m
            .memberships
            .as_ref()
            .and_then(|ms| ms.last())
            .map(|mw| {
                let ms = &mw.membership;
                let party = ms
                    .parties
                    .as_ref()
                    .and_then(|ps| ps.last())
                    .and_then(|pw| pw.party.show_as.clone())
                    .unwrap_or_default();
                let constituency = ms
                    .represents
                    .as_ref()
                    .and_then(|rs| rs.last())
                    .and_then(|rw| rw.represent.show_as.clone())
                    .unwrap_or_default();
                let house = ms
                    .house
                    .as_ref()
                    .and_then(|h| h.show_as.clone())
                    .unwrap_or_default();
                (party, constituency, house)
            })
            .unwrap_or_default();

        Self {
            name,
            party,
            constituency,
            house,
        }
    }
}

// -- Legislation --

#[derive(Debug, Deserialize, Serialize)]
pub struct BillResult {
    pub bill: BillData,
    #[serde(rename = "contextDate")]
    pub context_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BillData {
    #[serde(rename = "shortTitleEn")]
    pub short_title_en: Option<String>,
    #[serde(rename = "longTitleEn")]
    pub long_title_en: Option<String>,
    #[serde(rename = "billNo")]
    pub bill_no: Option<String>,
    #[serde(rename = "billYear")]
    pub bill_year: Option<String>,
    #[serde(rename = "billType")]
    pub bill_type: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    pub sponsors: Option<Vec<SponsorWrapper>>,
    pub uri: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SponsorWrapper {
    pub sponsor: Sponsor,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sponsor {
    pub by: Option<SponsorBy>,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SponsorBy {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
}

#[derive(Debug, Tabled, Serialize)]
pub struct BillRow {
    #[tabled(rename = "Bill No.")]
    pub bill_no: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Year")]
    pub year: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Sponsor")]
    pub sponsor: String,
}

impl BillRow {
    pub fn from_result(result: &BillResult) -> Self {
        let b = &result.bill;
        let sponsor = b
            .sponsors
            .as_ref()
            .and_then(|ss| ss.iter().find(|s| s.sponsor.is_primary == Some(true)))
            .or_else(|| b.sponsors.as_ref().and_then(|ss| ss.first()))
            .and_then(|sw| sw.sponsor.by.as_ref())
            .and_then(|by| by.show_as.clone())
            .unwrap_or_default();

        Self {
            bill_no: b.bill_no.clone().unwrap_or_default(),
            title: b.short_title_en.clone().unwrap_or_default(),
            year: b.bill_year.clone().unwrap_or_default(),
            status: b.status.clone().unwrap_or_default(),
            sponsor,
        }
    }
}

// -- Divisions --

#[derive(Debug, Deserialize, Serialize)]
pub struct DivisionResult {
    pub division: Option<DivisionDetail>,
    pub outcome: Option<String>,
    pub date: Option<String>,
    pub subject: Option<SubjectInfo>,
    pub tallies: Option<Tallies>,
    pub house: Option<HouseInfo>,
    #[serde(rename = "voteId")]
    pub vote_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DivisionDetail {
    pub uri: Option<String>,
    pub debate: Option<DivisionDebate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DivisionDebate {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubjectInfo {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tallies {
    #[serde(rename = "taVotes")]
    pub ta_votes: Option<VoteGroup>,
    #[serde(rename = "nilVotes")]
    pub nil_votes: Option<VoteGroup>,
    #[serde(rename = "staonVotes")]
    pub staon_votes: Option<VoteGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoteGroup {
    pub tally: Option<u32>,
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
}

#[derive(Debug, Tabled, Serialize)]
pub struct DivisionRow {
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Subject")]
    pub subject: String,
    #[tabled(rename = "House")]
    pub house: String,
    #[tabled(rename = "Tá")]
    pub ta: String,
    #[tabled(rename = "Níl")]
    pub nil: String,
    #[tabled(rename = "Outcome")]
    pub outcome: String,
}

impl DivisionRow {
    pub fn from_result(result: &DivisionResult) -> Self {
        // Subject: try top-level subject first, fall back to division.debate.showAs
        let subject = result
            .subject
            .as_ref()
            .and_then(|s| s.show_as.clone())
            .or_else(|| {
                result
                    .division
                    .as_ref()
                    .and_then(|d| d.debate.as_ref())
                    .and_then(|db| db.show_as.clone())
            })
            .unwrap_or_default();

        // Date: try top-level date, fall back to extracting from division URI
        // URI format: .../division/house/dail/34/2026-03-04/vote_58
        let date = result.date.clone().or_else(|| {
            result
                .division
                .as_ref()
                .and_then(|d| d.uri.as_ref())
                .and_then(|uri| {
                    uri.split('/')
                        .rev()
                        .nth(1)
                        .filter(|s| s.len() == 10 && s.contains('-'))
                        .map(|s| s.to_string())
                })
        }).unwrap_or_default();

        let ta = result
            .tallies
            .as_ref()
            .and_then(|t| t.ta_votes.as_ref())
            .and_then(|v| v.tally)
            .map(|t| t.to_string())
            .unwrap_or_default();
        let nil = result
            .tallies
            .as_ref()
            .and_then(|t| t.nil_votes.as_ref())
            .and_then(|v| v.tally)
            .map(|t| t.to_string())
            .unwrap_or_default();

        // House: try top-level, fall back to extracting from URI
        let house = result
            .house
            .as_ref()
            .and_then(|h| h.show_as.clone())
            .or_else(|| {
                result
                    .division
                    .as_ref()
                    .and_then(|d| d.uri.as_ref())
                    .and_then(|uri| {
                        if uri.contains("/dail/") {
                            Some("Dáil Éireann".to_string())
                        } else if uri.contains("/seanad/") {
                            Some("Seanad Éireann".to_string())
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or_default();

        Self {
            date,
            subject: irl_core::truncate_display(&subject, 60),
            house,
            ta,
            nil,
            outcome: result.outcome.clone().unwrap_or_default(),
        }
    }
}

// -- Questions --

#[derive(Debug, Deserialize, Serialize)]
pub struct QuestionResult {
    pub question: QuestionData,
    #[serde(rename = "contextDate")]
    pub context_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuestionData {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    #[serde(rename = "questionNumber")]
    pub question_number: Option<u32>,
    #[serde(rename = "questionType")]
    pub question_type: Option<String>,
    pub by: Option<QuestionMember>,
    pub to: Option<serde_json::Value>,
    pub date: Option<String>,
    pub house: Option<HouseInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuestionMember {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    #[serde(rename = "memberCode")]
    pub member_code: Option<String>,
}

#[derive(Debug, Tabled, Serialize)]
pub struct QuestionRow {
    #[tabled(rename = "No.")]
    pub number: String,
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Type")]
    pub question_type: String,
    #[tabled(rename = "Asked By")]
    pub asked_by: String,
    #[tabled(rename = "Topic")]
    pub topic: String,
}

impl QuestionRow {
    pub fn from_result(result: &QuestionResult) -> Self {
        let q = &result.question;
        let topic = q.show_as.clone().unwrap_or_default();
        Self {
            number: q.question_number.map(|n| n.to_string()).unwrap_or_default(),
            date: q.date.clone().unwrap_or_default(),
            question_type: q.question_type.clone().unwrap_or_default(),
            asked_by: q
                .by
                .as_ref()
                .and_then(|b| b.show_as.clone())
                .unwrap_or_default(),
            topic: irl_core::truncate_display(&topic, 50),
        }
    }
}

// -- Debates --

#[derive(Debug, Deserialize, Serialize)]
pub struct DebateResult {
    #[serde(rename = "contextDate")]
    pub context_date: Option<String>,
    #[serde(rename = "debateRecord")]
    pub debate_record: Option<DebateRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DebateRecord {
    pub date: Option<String>,
    pub chamber: Option<ChamberInfo>,
    pub counts: Option<DebateCounts>,
    pub sections: Option<Vec<DebateSection>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChamberInfo {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    #[serde(rename = "chamberCode")]
    pub chamber_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DebateCounts {
    #[serde(rename = "questionCount")]
    pub question_count: Option<u32>,
    #[serde(rename = "debateSectionCount")]
    pub debate_section_count: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DebateSection {
    #[serde(rename = "showAs")]
    pub show_as: Option<String>,
    #[serde(rename = "debateSectionId")]
    pub debate_section_id: Option<String>,
}

#[derive(Debug, Tabled, Serialize)]
pub struct DebateRow {
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Chamber")]
    pub chamber: String,
    #[tabled(rename = "Sections")]
    pub sections: String,
}

impl DebateRow {
    pub fn from_result(result: &DebateResult) -> Self {
        let record = result.debate_record.as_ref();
        let date = record.and_then(|r| r.date.clone()).unwrap_or_default();
        let chamber = record
            .and_then(|r| r.chamber.as_ref())
            .and_then(|c| c.show_as.clone())
            .unwrap_or_default();
        let section_count = record
            .and_then(|r| r.counts.as_ref())
            .and_then(|c| c.debate_section_count)
            .unwrap_or(0);

        Self {
            date,
            chamber,
            sections: format!("{} sections", section_count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_row_from_result() {
        let json = r#"{
            "member": {
                "showAs": "Test TD",
                "fullName": "Test TD Full",
                "firstName": "Test",
                "lastName": "TD",
                "memberCode": "T123",
                "memberships": [{
                    "membership": {
                        "house": { "showAs": "Dáil Éireann", "houseCode": "dail" },
                        "represents": [{ "represent": { "showAs": "Dublin Central" } }],
                        "parties": [{ "party": { "showAs": "Sinn Féin", "partyCode": "SF" } }]
                    }
                }]
            }
        }"#;
        let result: MemberResult = serde_json::from_str(json).unwrap();
        let row = MemberRow::from_result(&result);
        assert_eq!(row.name, "Test TD Full");
        assert_eq!(row.party, "Sinn Féin");
        assert_eq!(row.constituency, "Dublin Central");
        assert_eq!(row.house, "Dáil Éireann");
    }

    #[test]
    fn test_bill_row_from_result() {
        let json = r#"{
            "bill": {
                "shortTitleEn": "Planning Bill",
                "billNo": "42",
                "billYear": "2025",
                "status": "Current",
                "sponsors": [{ "sponsor": { "by": { "showAs": "Minister" }, "isPrimary": true } }]
            }
        }"#;
        let result: BillResult = serde_json::from_str(json).unwrap();
        let row = BillRow::from_result(&result);
        assert_eq!(row.title, "Planning Bill");
        assert_eq!(row.bill_no, "42");
        assert_eq!(row.status, "Current");
        assert_eq!(row.sponsor, "Minister");
    }

    #[test]
    fn test_division_row_from_result() {
        let json = r#"{
            "date": "2025-03-01",
            "outcome": "Carried",
            "subject": { "showAs": "Test Motion" },
            "house": { "showAs": "Dáil Éireann" },
            "tallies": {
                "taVotes": { "tally": 80 },
                "nilVotes": { "tally": 50 },
                "staonVotes": { "tally": 5 }
            }
        }"#;
        let result: DivisionResult = serde_json::from_str(json).unwrap();
        let row = DivisionRow::from_result(&result);
        assert_eq!(row.date, "2025-03-01");
        assert_eq!(row.ta, "80");
        assert_eq!(row.nil, "50");
        assert_eq!(row.outcome, "Carried");
    }

    #[test]
    fn test_question_row_from_result() {
        let json = r#"{
            "question": {
                "showAs": "Housing policy",
                "questionNumber": 42,
                "questionType": "oral",
                "by": { "showAs": "Mary Lou McDonald", "memberCode": "MLM" },
                "date": "2025-03-01"
            }
        }"#;
        let result: QuestionResult = serde_json::from_str(json).unwrap();
        let row = QuestionRow::from_result(&result);
        assert_eq!(row.number, "42");
        assert_eq!(row.asked_by, "Mary Lou McDonald");
        assert_eq!(row.topic, "Housing policy");
    }

    #[test]
    fn test_member_row_missing_fields() {
        let json = r#"{ "member": {} }"#;
        let result: MemberResult = serde_json::from_str(json).unwrap();
        let row = MemberRow::from_result(&result);
        assert_eq!(row.name, "");
        assert_eq!(row.party, "");
    }
}
