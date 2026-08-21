use fancy_regex::Regex;
use std::sync::LazyLock;

// Patterns
static RE_AMOUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-?\(?\d{1,3}(?:,\d{3})*\.\d{2}\)?").expect("valid regex"));

// Functions

pub fn parse_amount(s: &str) -> Option<f64> {
    let s = s.trim().replace('$', "");
    let s = s.trim();
    let in_parens = s.starts_with('(') && s.ends_with(')');
    let inner = if in_parens { &s[1..s.len() - 1] } else { s };
    let negative_sign = inner.starts_with('-');
    let digits = if negative_sign { &inner[1..] } else { inner };
    let value: f64 = digits.trim().replace(',', "").parse().ok()?;
    Some(if in_parens || negative_sign {
        -value
    } else {
        value
    })
}

pub fn find_amounts(text: &str) -> Vec<f64> {
    RE_AMOUNT
        .find_iter(text)
        .filter_map(|m| m.ok())
        .filter_map(|m| parse_amount(m.as_str()))
        .collect()
}

pub fn parse_quantity(s: &str) -> Option<f64> {
    s.trim().replace(',', "").parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parses_plain_amount() {
        assert_eq!(parse_amount("52.50"), Some(52.50));
    }

    #[test]
    fn test_parses_amount_with_thousands_separator() {
        assert_eq!(parse_amount("1,298.06"), Some(1298.06));
    }

    #[test]
    fn test_strips_dollar_sign_prefix() {
        assert_eq!(parse_amount("$150.00"), Some(150.00));
    }

    #[test]
    fn test_strips_dollar_sign_inside_parens() {
        assert_eq!(parse_amount("($150.00)"), Some(-150.00));
    }

    #[test]
    fn test_parses_parenthesis_amount_negative() {
        assert_eq!(parse_amount("(123.45)"), Some(-123.45));
    }

    #[test]
    fn test_parses_leading_neg_as_negative() {
        assert_eq!(parse_amount("-123.45"), Some(-123.45));
    }

    #[test]
    fn test_rejects_non_amount() {
        assert_eq!(parse_amount("Paid To Client/EDI"), None);
    }

    #[test]
    fn test_finds_all_amounts_ordered() {
        let text = "0.00 150.00 35.00 52.50";
        assert_eq!(find_amounts(text), vec![0.00, 150.00, 35.00, 52.50]);
    }

    #[test]
    fn test_finds_amounts_no_separator_whitespace() {
        let text = "1,298.0635.00Paid To Client/EDI";
        assert_eq!(find_amounts(text), vec![1298.06, 35.00]);
    }

    #[test]
    fn test_parses_bare_integer_quantity() {
        assert_eq!(parse_quantity("184"), Some(184.0));
    }

    #[test]
    fn test_parses_quantity_thousands_separator() {
        assert_eq!(parse_quantity("1,046"), Some(1046.0));
    }

    #[test]
    fn test_parses_quantity_decimals() {
        assert_eq!(parse_quantity("73.00"), Some(73.0));
    }

    #[test]
    fn test_rejects_non_quantity() {
        assert_eq!(parse_quantity("Some random text"), None);
    }
}
