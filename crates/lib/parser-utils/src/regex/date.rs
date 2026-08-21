use fancy_regex::Regex;
use std::sync::LazyLock;

// Patterns
static RE_DATE_MDY_SHORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<!\d)\d{1,2}/\d{1,2}/\d{2}(?!\d)").expect("valid regex"));

static RE_DATE_MDY_LONG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<!\d)\d{1,2}/\d{1,2}/\d{4}(?!\d)").expect("valid regex"));

// Functions
pub fn find_dates_mdy_short(text: &str) -> Vec<&str> {
    RE_DATE_MDY_SHORT
        .find_iter(text)
        .filter_map(|m| m.ok())
        .map(|m| m.as_str())
        .collect()
}

pub fn find_dates_mdy_long(text: &str) -> Vec<&str> {
    RE_DATE_MDY_LONG
        .find_iter(text)
        .filter_map(|m| m.ok())
        .map(|m| m.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_date() {
        assert_eq!(
            find_dates_mdy_short("Activity Date 03/04/25"),
            vec!["03/04/25"]
        );
    }

    #[test]
    fn test_long_date() {
        assert_eq!(
            find_dates_mdy_long("Covering 03/01/2025 Through 03/31/2025"),
            vec!["03/01/2025", "03/31/2025"]
        );
    }

    #[test]
    fn test_short_date_does_not_match_long_date() {
        assert_eq!(
            find_dates_mdy_short("Covering 03/01/2025 Through 03/31/2025"),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn test_long_date_does_not_match_short_date() {
        assert_eq!(
            find_dates_mdy_long("Activity Date 03/04/25"),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn test_distinguishes_short_long_same_text() {
        let text = "Service Period 03/01/2025 - 03/31/2025, Activity Date 03/04/25";
        assert_eq!(find_dates_mdy_long(text), vec!["03/01/2025", "03/31/2025"]);
        assert_eq!(find_dates_mdy_short(text), vec!["03/04/25"]);
    }

    #[test]
    fn test_finds_non_zero_padded_long_date() {
        assert_eq!(find_dates_mdy_long("DATE 3/31/2025"), vec!["3/31/2025"]);
    }

    #[test]
    fn test_non_zero_padded_long_date_no_leak_into_short_date_matches() {
        assert_eq!(find_dates_mdy_short("DATE 3/31/2025"), Vec::<&str>::new());
    }
}
