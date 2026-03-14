/// Lookup table mapping common location names to Met Eireann API station names.
///
/// The prodapi.metweb.ie API uses station names directly in the URL path,
/// e.g. `https://prodapi.metweb.ie/observations/Dublin%20Airport/today`.
pub struct Station {
    /// Common name users would type (lowercase for matching)
    pub alias: &'static str,
    /// API station name as used in the URL path
    pub api_name: &'static str,
    /// County for display purposes
    pub county: &'static str,
    /// Latitude (WGS84)
    pub lat: f64,
    /// Longitude (WGS84)
    pub lon: f64,
}

/// All known stations with their aliases and API names.
pub const STATIONS: &[Station] = &[
    Station {
        alias: "dublin",
        api_name: "Dublin Airport",
        county: "Dublin",
        lat: 53.4264,
        lon: -6.2499,
    },
    Station {
        alias: "cork",
        api_name: "Cork Airport",
        county: "Cork",
        lat: 51.8413,
        lon: -8.4911,
    },
    Station {
        alias: "galway",
        api_name: "Athenry",
        county: "Galway",
        lat: 53.2964,
        lon: -8.7483,
    },
    Station {
        alias: "limerick",
        api_name: "Shannon Airport",
        county: "Clare/Limerick",
        lat: 52.7019,
        lon: -8.9246,
    },
    Station {
        alias: "waterford",
        api_name: "Johnstown Castle",
        county: "Wexford/Waterford",
        lat: 52.2942,
        lon: -6.4969,
    },
    Station {
        alias: "belfast",
        api_name: "Malin Head",
        county: "Donegal",
        lat: 55.3717,
        lon: -7.3392,
    },
    Station {
        alias: "killarney",
        api_name: "Valentia Observatory",
        county: "Kerry",
        lat: 51.9381,
        lon: -10.2411,
    },
    Station {
        alias: "sligo",
        api_name: "Markree",
        county: "Sligo",
        lat: 54.1575,
        lon: -8.4575,
    },
    Station {
        alias: "athlone",
        api_name: "Mullingar",
        county: "Westmeath",
        lat: 53.5361,
        lon: -7.3617,
    },
    Station {
        alias: "letterkenny",
        api_name: "Malin Head",
        county: "Donegal",
        lat: 55.3717,
        lon: -7.3392,
    },
    Station {
        alias: "wexford",
        api_name: "Johnstown Castle",
        county: "Wexford",
        lat: 52.2942,
        lon: -6.4969,
    },
    Station {
        alias: "dundalk",
        api_name: "Casement Aerodrome",
        county: "Dublin/Louth",
        lat: 53.3017,
        lon: -6.4494,
    },
    Station {
        alias: "drogheda",
        api_name: "Casement Aerodrome",
        county: "Dublin/Louth",
        lat: 53.3017,
        lon: -6.4494,
    },
    Station {
        alias: "kilkenny",
        api_name: "Kilkenny",
        county: "Kilkenny",
        lat: 52.6541,
        lon: -7.2448,
    },
    Station {
        alias: "ennis",
        api_name: "Shannon Airport",
        county: "Clare",
        lat: 52.7019,
        lon: -8.9246,
    },
    Station {
        alias: "tralee",
        api_name: "Valentia Observatory",
        county: "Kerry",
        lat: 51.9381,
        lon: -10.2411,
    },
    Station {
        alias: "carlow",
        api_name: "Oak Park",
        county: "Carlow",
        lat: 52.8600,
        lon: -6.9150,
    },
    Station {
        alias: "tullamore",
        api_name: "Mullingar",
        county: "Westmeath/Offaly",
        lat: 53.5361,
        lon: -7.3617,
    },
    Station {
        alias: "derry",
        api_name: "Malin Head",
        county: "Donegal",
        lat: 55.3717,
        lon: -7.3392,
    },
    Station {
        alias: "newry",
        api_name: "Casement Aerodrome",
        county: "Dublin",
        lat: 53.3017,
        lon: -6.4494,
    },
];

/// Resolve a user-supplied location name to the Met Eireann API station name.
/// Performs case-insensitive matching against aliases and API names,
/// with fuzzy matching fallback for typos.
pub fn resolve_location(input: &str) -> Option<&'static str> {
    let lower = input.to_lowercase();

    if lower.is_empty() {
        return None;
    }

    // Exact alias match
    for station in STATIONS {
        if station.alias == lower {
            return Some(station.api_name);
        }
    }

    // Check if input matches an API name directly (case-insensitive)
    for station in STATIONS {
        if station.api_name.to_lowercase() == lower {
            return Some(station.api_name);
        }
    }

    // Substring match on alias
    for station in STATIONS {
        if station.alias.contains(&lower) || lower.contains(station.alias) {
            return Some(station.api_name);
        }
    }

    // Fuzzy match on aliases as last resort
    let aliases: Vec<&str> = STATIONS.iter().map(|s| s.alias).collect();
    let matches = irl_core::fuzzy::fuzzy_match(input, &aliases, 0.85);
    if let Some(best) = matches.first() {
        for station in STATIONS {
            if station.alias == best.candidate {
                return Some(station.api_name);
            }
        }
    }

    None
}

/// Get fuzzy match suggestions for a location name that didn't resolve.
/// Returns a formatted hint string, or empty if no good matches.
pub fn suggest_location(input: &str) -> String {
    let aliases: Vec<&str> = STATIONS.iter().map(|s| s.alias).collect();
    let api_names: Vec<&str> = STATIONS.iter().map(|s| s.api_name).collect();

    let mut all_candidates = aliases;
    all_candidates.extend(api_names);

    let matches = irl_core::fuzzy::fuzzy_match(input, &all_candidates, 0.7);
    irl_core::fuzzy::format_suggestions(&matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_exact_alias() {
        assert_eq!(resolve_location("dublin"), Some("Dublin Airport"));
        assert_eq!(resolve_location("cork"), Some("Cork Airport"));
        assert_eq!(resolve_location("galway"), Some("Athenry"));
    }

    #[test]
    fn test_resolve_case_insensitive() {
        assert_eq!(resolve_location("Dublin"), Some("Dublin Airport"));
        assert_eq!(resolve_location("CORK"), Some("Cork Airport"));
        assert_eq!(resolve_location("Galway"), Some("Athenry"));
    }

    #[test]
    fn test_resolve_api_name_direct() {
        assert_eq!(resolve_location("Dublin Airport"), Some("Dublin Airport"));
        assert_eq!(resolve_location("Shannon Airport"), Some("Shannon Airport"));
    }

    #[test]
    fn test_resolve_unknown_location() {
        assert_eq!(resolve_location("timbuktu"), None);
        assert_eq!(resolve_location(""), None);
    }

    #[test]
    fn test_all_stations_have_data() {
        for station in STATIONS {
            assert!(!station.alias.is_empty());
            assert!(!station.api_name.is_empty());
            assert!(!station.county.is_empty());
        }
    }

    #[test]
    fn test_station_count() {
        assert_eq!(STATIONS.len(), 20);
    }
}
