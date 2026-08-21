use pdf_inspector::types::TextItem;

// Position based

fn center_in_range(item: &TextItem, x0: f32, x1: f32) -> bool {
    let center = item.x + item.width / 2.0;
    center >= x0 && center <= x1
}

pub fn item_in_x_range(items: &[TextItem], x0: f32, x1: f32) -> Option<&TextItem> {
    items.iter().find(|i| center_in_range(i, x0, x1))
}

pub fn items_in_x_range(items: &[TextItem], x0: f32, x1: f32) -> Vec<&TextItem> {
    items
        .iter()
        .filter(|i| center_in_range(i, x0, x1))
        .collect()
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

    #[test]
    fn test_finds_item_inside_range() {
        let items = vec![item("name", 10.0, 50.0), item("03/04/26", 150.0, 60.0)];
        let found = item_in_x_range(&items, 140.0, 215.0).unwrap();
        assert_eq!(found.text, "03/04/26");
    }

    #[test]
    fn test_find_item_center_inside() {
        let items = vec![item("03/04/26", 130.0, 40.0)];
        assert!(item_in_x_range(&items, 140.0, 215.0).is_some());
    }

    #[test]
    fn test_ignore_neighbor_items_with_bleeding() {
        let items = vec![
            item("Some, Dude E", 99.6, 92.8),
            item("03/04/26", 173.1, 31.1),
        ];
        let found = item_in_x_range(&items, 168.1, 209.3).unwrap();
        assert_eq!(found.text, "03/04/26");
    }

    #[test]
    fn test_no_match_outside_range() {
        let items = vec![item("name", 10.0, 50.0)];
        assert!(item_in_x_range(&items, 140.0, 215.0).is_none());
    }
}
