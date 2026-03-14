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
}

/// All known stations with their aliases and API names.
pub const STATIONS: &[Station] = &[
    Station {
        alias: "dublin",
        api_name: "Dublin Airport",
        county: "Dublin",
    },
    Station {
        alias: "cork",
        api_name: "Cork Airport",
        county: "Cork",
    },
    Station {
        alias: "galway",
        api_name: "Athenry",
        county: "Galway",
    },
    Station {
        alias: "limerick",
        api_name: "Shannon Airport",
        county: "Clare/Limerick",
    },
    Station {
        alias: "waterford",
        api_name: "Johnstown Castle",
        county: "Wexford/Waterford",
    },
    Station {
        alias: "belfast",
        api_name: "Malin Head",
        county: "Donegal",
    },
    Station {
        alias: "killarney",
        api_name: "Valentia Observatory",
        county: "Kerry",
    },
    Station {
        alias: "sligo",
        api_name: "Markree",
        county: "Sligo",
    },
    Station {
        alias: "athlone",
        api_name: "Mullingar",
        county: "Westmeath",
    },
    Station {
        alias: "letterkenny",
        api_name: "Malin Head",
        county: "Donegal",
    },
    Station {
        alias: "wexford",
        api_name: "Johnstown Castle",
        county: "Wexford",
    },
    Station {
        alias: "dundalk",
        api_name: "Casement Aerodrome",
        county: "Dublin/Louth",
    },
    Station {
        alias: "drogheda",
        api_name: "Casement Aerodrome",
        county: "Dublin/Louth",
    },
    Station {
        alias: "kilkenny",
        api_name: "Kilkenny",
        county: "Kilkenny",
    },
    Station {
        alias: "ennis",
        api_name: "Shannon Airport",
        county: "Clare",
    },
    Station {
        alias: "tralee",
        api_name: "Valentia Observatory",
        county: "Kerry",
    },
    Station {
        alias: "carlow",
        api_name: "Oak Park",
        county: "Carlow",
    },
    Station {
        alias: "tullamore",
        api_name: "Mullingar",
        county: "Westmeath/Offaly",
    },
    Station {
        alias: "derry",
        api_name: "Malin Head",
        county: "Donegal",
    },
    Station {
        alias: "newry",
        api_name: "Casement Aerodrome",
        county: "Dublin",
    },
];

/// Resolve a user-supplied location name to the Met Eireann API station name.
/// Performs case-insensitive matching against aliases and API names.
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

    None
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
