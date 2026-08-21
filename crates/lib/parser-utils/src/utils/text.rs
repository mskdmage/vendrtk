pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn looks_like_section_label(text: &str, max_len: usize) -> bool {
    text.len() <= max_len && !text.chars().any(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapses_internal_whitespace() {
        assert_eq!(normalize_whitespace("a   b\t\tc"), "a b c");
    }

    #[test]
    fn test_trims_leading_and_trailing_whitespace() {
        assert_eq!(normalize_whitespace("  a b  "), "a b");
    }

    #[test]
    fn test_collapses_newlines_too() {
        assert_eq!(normalize_whitespace("a\nb\n\nc"), "a b c");
    }

    #[test]
    fn test_empty_input_yields_empty_output() {
        assert_eq!(normalize_whitespace("   "), "");
    }

    #[test]
    fn test_short_text_no_digits_is_label() {
        assert!(looks_like_section_label("Inpatient Coding", 40));
    }

    #[test]
    fn test_data_row_digits_not_label() {
        let row = "Service;Inpatient Coding Inpatient Coding 184 73.00 13,432.00";
        assert!(!looks_like_section_label(row, 60));
    }

    #[test]
    fn test_text_is_longer_than_max_len() {
        assert!(!looks_like_section_label("Inpatient Coding", 5));
    }
}
