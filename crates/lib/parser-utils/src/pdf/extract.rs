use crate::error::Result;
use pdf_inspector::{PdfType, types::TextLine};

pub fn extract_lines(bytes: &[u8]) -> Result<Option<Vec<TextLine>>> {
    let classification = pdf_inspector::detect_pdf_type_mem(bytes)?;

    match classification.pdf_type {
        PdfType::TextBased | PdfType::Mixed => {
            let items = pdf_inspector::extract_text_with_positions_mem(bytes)?;

            if items.is_empty() {
                return Ok(None);
            }

            Ok(Some(pdf_inspector::extractor::group_into_lines(items)))
        }
        _ => Ok(None),
    }
}

pub fn extract_pages(bytes: &[u8]) -> Result<Option<Vec<Vec<String>>>> {
    let Some(lines) = extract_lines(bytes)? else {
        return Ok(None);
    };

    let mut pages: Vec<Vec<String>> = Vec::new();
    for line in lines {
        let page_index = line.page.saturating_sub(1) as usize;
        if page_index >= pages.len() {
            pages.resize(page_index + 1, Vec::new());
        }
        pages[page_index].push(line.text());
    }

    Ok(Some(pages))
}

pub fn group_lines_by_page(lines: &[TextLine]) -> Vec<Vec<&TextLine>> {
    let mut pages: Vec<Vec<&TextLine>> = Vec::new();
    for line in lines {
        let index = line.page.saturating_sub(1) as usize;
        if index >= pages.len() {
            pages.resize(index + 1, Vec::new());
        }
        pages[index].push(line);
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(page: u32) -> TextLine {
        TextLine {
            items: Vec::new(),
            y: 0.0,
            page,
            adaptive_threshold: 0.1,
        }
    }

    #[test]
    fn groups_lines_by_page_preserving_order() {
        let lines = vec![line(1), line(1), line(2), line(1), line(3)];

        let pages = group_lines_by_page(&lines);

        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].len(), 3);
        assert_eq!(pages[1].len(), 1);
        assert_eq!(pages[2].len(), 1);
    }

    #[test]
    fn empty_input_yields_no_pages() {
        assert!(group_lines_by_page(&[]).is_empty());
    }
}
