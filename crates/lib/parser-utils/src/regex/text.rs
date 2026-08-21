use fancy_regex::Regex;
use std::sync::LazyLock;

// Patterns
static RE_LEADING_REFERENCE_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{7,12}(?:-\d+)?)\s").expect("valid regex"));

static RE_DASHED_ACCOUNT_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{3,6}-\d{3,6}[A-Z0-9]*\b").expect("valid regex"));

// Functions

pub fn leading_reference_number(line: &str) -> Option<&str> {
    let captures = RE_LEADING_REFERENCE_NUMBER.captures(line).ok()??;
    captures.get(1).map(|m| m.as_str())
}

pub fn find_dashed_account_codes(text: &str) -> Vec<&str> {
    RE_DASHED_ACCOUNT_CODE
        .find_iter(text)
        .filter_map(|m| m.ok())
        .map(|m| m.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leading_reference_number() {
        let line = "1006254088 Some, Dud E 03/04/25 Paid To Client/EDI";
        assert_eq!(leading_reference_number(line), Some("1006254088"));
    }

    #[test]
    fn test_leading_reference_number_dash() {
        let line = "1006254088-2 Another Dud E 03/04/27";
        assert_eq!(leading_reference_number(line), Some("1006254088-2"));
    }

    #[test]
    fn test_rejects_bare_number() {
        assert_eq!(leading_reference_number("1006254088"), None);
    }

    #[test]
    fn test_rejects_not_starting_w_reference_number() {
        let line = "Account Name Date Description Agency";
        assert_eq!(leading_reference_number(line), None);
    }

    #[test]
    fn test_dashed_account_codes() {
        let text = "6780-7654E 03/31/2026 Amount Due";
        assert_eq!(find_dashed_account_codes(text), vec!["6780-7654E"]);
    }
}
