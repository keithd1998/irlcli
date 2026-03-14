pub mod cache;
pub mod config;
pub mod error;
pub mod fuzzy;
pub mod http;
pub mod output;

/// Truncate a string for table display, safely handling multi-byte UTF-8 characters.
/// Appends "..." when truncation occurs.
pub fn truncate_display(s: &str, max: usize) -> String {
    if max < 4 {
        return s.chars().take(max).collect();
    }
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max - 3).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_display_short_string() {
        assert_eq!(truncate_display("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_display_exact_length() {
        assert_eq!(truncate_display("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_display_long_string() {
        let result = truncate_display("this is a long string", 10);
        assert_eq!(result, "this is...");
        assert_eq!(result.chars().count(), 10);
    }

    #[test]
    fn test_truncate_display_with_fadas() {
        // Irish names with fadas (multi-byte UTF-8)
        let name = "Seán Ó Brádaigh, Príomh-Aire na hÉireann";
        let result = truncate_display(name, 20);
        assert!(result.ends_with("..."));
        assert!(result.chars().count() <= 20);
    }

    #[test]
    fn test_truncate_display_emoji() {
        let s = "Hello 🇮🇪 Ireland";
        let result = truncate_display(s, 10);
        assert!(result.chars().count() <= 10);
    }

    #[test]
    fn test_truncate_display_empty() {
        assert_eq!(truncate_display("", 10), "");
    }
}
