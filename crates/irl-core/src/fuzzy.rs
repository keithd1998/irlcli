// Fuzzy string matching utilities for suggesting corrections to user input.
// Uses Jaro-Winkler similarity which favours strings sharing a common prefix,
// making it well-suited for place name typos ("dubln" → "dublin").

/// Compute Jaro similarity between two strings (0.0 = no match, 1.0 = identical).
fn jaro(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    let match_distance = (a_len.max(b_len) / 2).saturating_sub(1);
    let mut a_matched = vec![false; a_len];
    let mut b_matched = vec![false; b_len];

    let mut matches = 0.0f64;
    let mut transpositions = 0.0f64;

    for i in 0..a_len {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(b_len);

        for j in start..end {
            if b_matched[j] || a_chars[i] != b_chars[j] {
                continue;
            }
            a_matched[i] = true;
            b_matched[j] = true;
            matches += 1.0;
            break;
        }
    }

    if matches == 0.0 {
        return 0.0;
    }

    let mut k = 0;
    for i in 0..a_len {
        if !a_matched[i] {
            continue;
        }
        while !b_matched[k] {
            k += 1;
        }
        if a_chars[i] != b_chars[k] {
            transpositions += 1.0;
        }
        k += 1;
    }

    (matches / a_len as f64 + matches / b_len as f64 + (matches - transpositions / 2.0) / matches)
        / 3.0
}

/// Compute Jaro-Winkler similarity (0.0 = no match, 1.0 = identical).
/// Boosts the score for strings sharing a common prefix (up to 4 chars).
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let jaro_score = jaro(a, b);

    // Count common prefix (up to 4 characters)
    let prefix_len = a
        .chars()
        .zip(b.chars())
        .take(4)
        .take_while(|(ac, bc)| ac == bc)
        .count();

    // Winkler boost: p = 0.1 (standard)
    jaro_score + (prefix_len as f64 * 0.1 * (1.0 - jaro_score))
}

/// A match result with the candidate string and its similarity score.
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub candidate: String,
    pub score: f64,
}

/// Find the best fuzzy matches for a query against a list of candidates.
/// Returns matches above the threshold, sorted by score (best first).
pub fn fuzzy_match(query: &str, candidates: &[&str], threshold: f64) -> Vec<FuzzyMatch> {
    let query_lower = query.to_lowercase();
    let mut matches: Vec<FuzzyMatch> = candidates
        .iter()
        .map(|&c| {
            let score = jaro_winkler(&query_lower, &c.to_lowercase());
            FuzzyMatch {
                candidate: c.to_string(),
                score,
            }
        })
        .filter(|m| m.score >= threshold)
        .collect();

    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    matches.truncate(3);
    matches
}

/// Format fuzzy match suggestions as a helpful hint string.
pub fn format_suggestions(matches: &[FuzzyMatch]) -> String {
    if matches.is_empty() {
        return String::new();
    }
    let suggestions: Vec<String> = matches.iter().map(|m| format!("\"{}\"", m.candidate)).collect();
    format!("Did you mean {}?", suggestions.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler_identical() {
        assert!((jaro_winkler("dublin", "dublin") - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_jaro_winkler_similar() {
        let score = jaro_winkler("dubln", "dublin");
        assert!(score > 0.9, "Expected > 0.9, got {}", score);
    }

    #[test]
    fn test_jaro_winkler_different() {
        let score = jaro_winkler("dublin", "galway");
        assert!(score < 0.7, "Expected < 0.7, got {}", score);
    }

    #[test]
    fn test_jaro_winkler_empty() {
        assert!((jaro_winkler("", "dublin") - 0.0).abs() < 0.001);
        assert!((jaro_winkler("dublin", "") - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_jaro_winkler_case_sensitive() {
        // Jaro-Winkler is case-sensitive, fuzzy_match handles lowercasing
        let score = jaro_winkler("Dublin", "dublin");
        assert!(score < 1.0);
    }

    #[test]
    fn test_fuzzy_match_finds_typo() {
        let candidates = &["dublin", "cork", "galway", "limerick"];
        let matches = fuzzy_match("dubln", candidates, 0.8);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].candidate, "dublin");
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let candidates = &["dublin", "cork", "galway"];
        let matches = fuzzy_match("xyz", candidates, 0.8);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let candidates = &["Dublin Airport", "Cork Airport"];
        let matches = fuzzy_match("dublin airport", candidates, 0.8);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].candidate, "Dublin Airport");
    }

    #[test]
    fn test_fuzzy_match_returns_max_3() {
        let candidates = &["aa", "ab", "ac", "ad", "ae"];
        let matches = fuzzy_match("aa", candidates, 0.5);
        assert!(matches.len() <= 3);
    }

    #[test]
    fn test_format_suggestions_empty() {
        assert_eq!(format_suggestions(&[]), "");
    }

    #[test]
    fn test_format_suggestions_single() {
        let matches = vec![FuzzyMatch {
            candidate: "dublin".to_string(),
            score: 0.95,
        }];
        assert_eq!(format_suggestions(&matches), "Did you mean \"dublin\"?");
    }

    #[test]
    fn test_format_suggestions_multiple() {
        let matches = vec![
            FuzzyMatch {
                candidate: "dublin".to_string(),
                score: 0.95,
            },
            FuzzyMatch {
                candidate: "dundalk".to_string(),
                score: 0.8,
            },
        ];
        let result = format_suggestions(&matches);
        assert!(result.contains("dublin"));
        assert!(result.contains("dundalk"));
    }

    #[test]
    fn test_irish_constituency_names() {
        let candidates = &[
            "Dublin Central",
            "Dublin Bay North",
            "Dublin Bay South",
            "Dublin North-West",
            "Dublin South-Central",
        ];
        let matches = fuzzy_match("Dublin North Central", candidates, 0.75);
        assert!(!matches.is_empty());
        // Should suggest Dublin Central or Dublin North-West as close matches
    }
}
