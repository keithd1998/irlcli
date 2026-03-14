// Maps historical Irish constituency names to their current equivalents
// after the 2016 and 2020 boundary commission redistricting.

pub struct ConstituencyMapping {
    /// The historical name (no longer in use)
    pub historical: &'static str,
    /// Current constituency/constituencies that cover the same area
    pub current: &'static [&'static str],
    /// Year the change took effect
    pub changed: u16,
}

/// Historical constituency names and their current equivalents.
pub const HISTORICAL_MAPPINGS: &[ConstituencyMapping] = &[
    // 2016 boundary changes
    ConstituencyMapping {
        historical: "Dublin North-Central",
        current: &["Dublin Central", "Dublin Bay North"],
        changed: 2016,
    },
    ConstituencyMapping {
        historical: "Dublin South-East",
        current: &["Dublin Bay South"],
        changed: 2016,
    },
    ConstituencyMapping {
        historical: "Dublin North-East",
        current: &["Dublin Bay North"],
        changed: 2016,
    },
    ConstituencyMapping {
        historical: "Dublin North",
        current: &["Dublin Fingal East", "Dublin Fingal West"],
        changed: 2024,
    },
    ConstituencyMapping {
        historical: "Dublin South",
        current: &["Dublin Rathdown"],
        changed: 2016,
    },
    ConstituencyMapping {
        historical: "Dún Laoghaire",
        current: &["Dublin Rathdown", "Dublin Bay South"],
        changed: 2016,
    },
    ConstituencyMapping {
        historical: "Dublin South-West",
        current: &["Dublin South-West", "Dublin Mid-West"],
        changed: 2016,
    },
];

/// All current constituency names (34th Dáil, 2024).
pub const CURRENT_CONSTITUENCIES: &[&str] = &[
    "Carlow-Kilkenny",
    "Cavan-Monaghan",
    "Clare",
    "Cork East",
    "Cork North-Central",
    "Cork North-West",
    "Cork South-Central",
    "Cork South-West",
    "Donegal",
    "Dublin Bay North",
    "Dublin Bay South",
    "Dublin Central",
    "Dublin Fingal East",
    "Dublin Fingal West",
    "Dublin Mid-West",
    "Dublin North-West",
    "Dublin Rathdown",
    "Dublin South-Central",
    "Dublin South-West",
    "Dublin West",
    "Galway East",
    "Galway West",
    "Kerry",
    "Kildare North",
    "Kildare South",
    "Laois-Offaly",
    "Limerick City",
    "Limerick County",
    "Longford-Westmeath",
    "Louth",
    "Mayo",
    "Meath East",
    "Meath West",
    "Roscommon-Galway",
    "Sligo-Leitrim",
    "Tipperary North",
    "Tipperary South",
    "Waterford",
    "Wexford",
    "Wicklow",
];

/// Normalize a constituency name for comparison: lowercase, collapse hyphens/spaces.
fn normalize(s: &str) -> String {
    s.to_lowercase().replace('-', " ").replace("  ", " ")
}

/// Resolve a constituency name, checking for historical names first.
/// Returns (resolved_names, was_historical).
/// Handles variations in hyphenation (e.g., "Dublin North Central" matches "Dublin North-Central").
pub fn resolve_constituency(input: &str) -> (Vec<&'static str>, bool) {
    let normalized = normalize(input);

    // Check historical mappings first (with hyphen normalization)
    for mapping in HISTORICAL_MAPPINGS {
        if normalize(mapping.historical) == normalized {
            return (mapping.current.to_vec(), true);
        }
    }

    // Check if it matches a current constituency
    for &name in CURRENT_CONSTITUENCIES {
        if normalize(name) == normalized {
            return (vec![name], false);
        }
    }

    // Substring match on current constituencies
    let matches: Vec<&str> = CURRENT_CONSTITUENCIES
        .iter()
        .filter(|&&name| normalize(name).contains(&normalized))
        .copied()
        .collect();

    if !matches.is_empty() {
        return (matches, false);
    }

    (vec![], false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_current_constituency() {
        let (names, historical) = resolve_constituency("Dublin Central");
        assert_eq!(names, vec!["Dublin Central"]);
        assert!(!historical);
    }

    #[test]
    fn test_resolve_historical_constituency() {
        let (names, historical) = resolve_constituency("Dublin North-Central");
        assert_eq!(names, vec!["Dublin Central", "Dublin Bay North"]);
        assert!(historical);
    }

    #[test]
    fn test_resolve_historical_case_insensitive() {
        let (names, historical) = resolve_constituency("dublin north-central");
        assert_eq!(names, vec!["Dublin Central", "Dublin Bay North"]);
        assert!(historical);
    }

    #[test]
    fn test_resolve_unknown_constituency() {
        let (names, _) = resolve_constituency("Nonexistent Place");
        assert!(names.is_empty());
    }

    #[test]
    fn test_resolve_dublin_south_east() {
        let (names, historical) = resolve_constituency("Dublin South-East");
        assert_eq!(names, vec!["Dublin Bay South"]);
        assert!(historical);
    }

    #[test]
    fn test_resolve_historical_no_hyphen() {
        // Users often omit the hyphen
        let (names, historical) = resolve_constituency("Dublin North Central");
        assert_eq!(names, vec!["Dublin Central", "Dublin Bay North"]);
        assert!(historical);
    }

    #[test]
    fn test_resolve_substring_match() {
        let (names, historical) = resolve_constituency("Cork");
        assert!(names.len() >= 3); // Cork East, Cork North-Central, etc.
        assert!(!historical);
    }
}
