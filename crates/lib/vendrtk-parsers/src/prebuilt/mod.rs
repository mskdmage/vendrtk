use crate::error::{Error, Result};
use crate::models::invoice::{Invoice, InvoiceHeader, ParsedInvoices};
use crate::traits::parser::Parser;
use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;

pub struct SampleInvoiceParser;

impl<O> Parser<O> for SampleInvoiceParser
where
    O: OcrProcessedDocument + Send,
{
    type Output = ParsedInvoices;

    fn parse(
        &self,
        ocr_result: Option<O>,
        _bytes: Option<&[u8]>,
    ) -> impl Future<Output = Result<ParsedInvoices>> + Send {
        async move {
            let ocr = ocr_result
                .ok_or_else(|| Error::Parsing("OCR result is required".into()))?;

            let key = ocr.key().to_string();
            let content = ocr
                .raw_content()
                .map_err(|e| Error::Parsing(e.to_string()))?;

            Ok(ParsedInvoices {
                key,
                results: vec![parse_invoice_from_text(&content)?],
            })
        }
    }
}

fn parse_invoice_from_text(content: &str) -> Result<Invoice> {
    let invoice_number = content
        .lines()
        .find_map(|line| line.trim().strip_prefix("INVOICE ").map(str::trim))
        .ok_or_else(|| Error::Parsing("invoice number not found".into()))?
        .to_string();

    let vendor = content
        .lines()
        .skip_while(|line| !line.trim().starts_with("INVOICE "))
        .nth(1)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .ok_or_else(|| Error::Parsing("vendor not found".into()))?
        .to_string();

    let invoice_amount = content
        .lines()
        .map(str::trim)
        .rev()
        .find_map(|line| line.strip_prefix("USD ").and_then(|amount| amount.parse().ok()))
        .ok_or_else(|| Error::Parsing("invoice total not found".into()))?;

    Ok(Invoice {
        header: InvoiceHeader {
            vendor,
            invoice_number,
            invoice_date: String::new(),
            facility: String::new(),
            billing_start: String::new(),
            billing_end: String::new(),
            invoice_amount,
        },
        details: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::parsed_document::{ParsedDocument, ParsedPayload};

    struct TestOcrDoc {
        key: String,
        content: String,
    }

    impl OcrProcessedDocument for TestOcrDoc {
        fn key(&self) -> &str {
            &self.key
        }

        fn raw_content(&self) -> vendrtk_ocr::error::Result<String> {
            Ok(self.content.clone())
        }

        fn pages(&self) -> vendrtk_ocr::error::Result<Vec<String>> {
            Ok(vec![self.content.clone()])
        }
    }

    #[tokio::test]
    async fn parses_invoice_fields_from_ocr_text() {
        let content = "INVOICE 19424831\nVENDOR LLC\nTOTAL\nUSD 99.99";
        let parser = SampleInvoiceParser;
        let parsed = parser
            .parse(
                Some(TestOcrDoc {
                    key: "test-123".into(),
                    content: content.into(),
                }),
                None,
            )
            .await
            .unwrap();

        assert_eq!(parsed.key(), "test-123");
        let invoices = parsed.results().unwrap();
        assert_eq!(invoices.len(), 1);
        assert_eq!(invoices[0].header.invoice_number, "19424831");
        assert_eq!(invoices[0].header.vendor, "VENDOR LLC");
        assert!((invoices[0].header.invoice_amount - 99.99).abs() < f64::EPSILON);
    }
}
