use crate::utils::column::item_in_x_range;
use crate::utils::text::{looks_like_section_label, normalize_whitespace};
use pdf_inspector::types::{TextItem, TextLine};

const COLUMN_ALIGNMENT_SLACK: f32 = 5.0;

pub fn detect_header_row(lines: &[TextLine], lookahead: usize) -> Option<Vec<String>> {
    for (i, line) in lines.iter().enumerate() {
        if line.items.len() < 3 {
            continue;
        }

        if !line
            .items
            .iter()
            .all(|item| looks_like_section_label(item.text.trim(), 30))
        {
            continue;
        }

        let end = (i + 1 + lookahead).min(lines.len());
        let later_lines = &lines[i + 1..end];
        let has_aligned_data_below = line.items.iter().any(|header_item| {
            let x0 = header_item.x - COLUMN_ALIGNMENT_SLACK;
            let x1 = header_item.x + header_item.width + COLUMN_ALIGNMENT_SLACK;
            later_lines.iter().any(|later| {
                item_in_x_range(&later.items, x0, x1)
                    .is_some_and(|found| found.text.chars().any(|c| c.is_ascii_digit()))
            })
        });

        if has_aligned_data_below {
            let mut items: Vec<&TextItem> = line.items.iter().collect();
            items.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
            return Some(
                items
                    .into_iter()
                    .map(|item| normalize_whitespace(item.text.trim()))
                    .collect(),
            );
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_inspector::extractor::ItemType;

    fn item(text: &str, x: f32, width: f32) -> TextItem {
        TextItem {
            text: text.to_string(),
            x,
            y: 100.0,
            width,
            height: 12.0,
            font: "F1".to_string(),
            font_size: 12.0,
            page: 1,
            is_bold: false,
            is_italic: false,
            is_underline: false,
            is_strikeout: false,
            item_type: ItemType::Text,
            mcid: None,
        }
    }

    fn text_line(items: Vec<TextItem>, y: f32) -> TextLine {
        TextLine {
            items,
            y,
            page: 1,
            adaptive_threshold: 0.1,
        }
    }

    #[test]
    fn test_detect_a_header_row_aligned_numeric_data_below() {
        let lines = vec![
            text_line(
                vec![
                    item("ACTIVITY", 0.0, 40.0),
                    item("DESCRIPTION", 50.0, 60.0),
                    item("QTY", 120.0, 20.0),
                    item("RATE", 150.0, 20.0),
                    item("AMOUNT", 180.0, 30.0),
                ],
                100.0,
            ),
            text_line(
                vec![
                    item("Coding services", 50.0, 60.0),
                    item("2", 122.0, 10.0),
                    item("75.00", 150.0, 20.0),
                    item("150.00", 180.0, 30.0),
                ],
                90.0,
            ),
        ];
        let header = detect_header_row(&lines, 5).unwrap();
        assert_eq!(
            header,
            vec!["ACTIVITY", "DESCRIPTION", "QTY", "RATE", "AMOUNT"]
        );
    }

    #[test]
    fn test_ignores_section_title_no_data_row_below() {
        let lines = vec![
            text_line(
                vec![
                    item("Inpatient", 0.0, 40.0),
                    item("Coding", 50.0, 40.0),
                    item("Services", 100.0, 40.0),
                ],
                100.0,
            ),
            text_line(vec![item("Some other label text", 0.0, 80.0)], 90.0),
        ];
        assert!(detect_header_row(&lines, 5).is_none());
    }

    #[test]
    fn test_requires_min_three_items_on_line() {
        let lines = vec![
            text_line(
                vec![item("RATE", 0.0, 20.0), item("AMOUNT", 30.0, 30.0)],
                100.0,
            ),
            text_line(vec![item("75.00", 0.0, 20.0)], 90.0),
        ];
        assert!(detect_header_row(&lines, 5).is_none());
    }

    #[test]
    fn test_does_not_go_past_lookahead() {
        let lines = vec![
            text_line(
                vec![
                    item("QTY", 0.0, 20.0),
                    item("RATE", 30.0, 20.0),
                    item("AMOUNT", 60.0, 30.0),
                ],
                100.0,
            ),
            text_line(vec![item("no digits here", 0.0, 60.0)], 90.0),
            text_line(vec![item("still no digits", 0.0, 60.0)], 80.0),
            text_line(vec![item("2", 0.0, 20.0)], 70.0),
        ];
        assert!(detect_header_row(&lines, 2).is_none());
        assert!(detect_header_row(&lines, 3).is_some());
    }

    #[test]
    fn test_returns_header_items_left_to_right() {
        let lines = vec![
            text_line(
                vec![
                    item("AMOUNT", 180.0, 30.0),
                    item("QTY", 0.0, 20.0),
                    item("RATE", 90.0, 20.0),
                ],
                100.0,
            ),
            text_line(vec![item("2", 5.0, 10.0)], 90.0),
        ];
        let header = detect_header_row(&lines, 5).unwrap();
        assert_eq!(header, vec!["QTY", "RATE", "AMOUNT"]);
    }
}
