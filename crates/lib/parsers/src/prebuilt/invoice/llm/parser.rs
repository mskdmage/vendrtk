use crate::error::{Error, Result};
use crate::models::invoice::ParsedInvoices;
use crate::prebuilt::invoice::schemas::ExtractedInvoiceList;
use crate::traits::{LLMClient, Parser};
use ocr::traits::OcrProcessedDocument;

const EXTRACTION_PREAMBLE: &str = "Extract structured invoice data from the OCR text. \
Return one invoice per document unless the text clearly contains multiple distinct invoices. \
Use header fields for invoice-level data; put each billable row in details.";

pub struct LLMInvoiceParser;

impl LLMInvoiceParser {
    pub const fn new() -> Self {
        Self
    }

    pub async fn parse<C, O>(&self, client: &C, ocr_result: O) -> Result<ParsedInvoices>
    where
        C: LLMClient + Send + Sync,
        O: OcrProcessedDocument + Send,
    {
        let key = ocr_result.key().to_string();
        let content = ocr_result
            .raw_content()
            .map_err(|_e| Error::EmptyOcrContent)?;
        if content.trim().is_empty() {
            return Err(Error::EmptyOcrContent);
        }

        let extracted: ExtractedInvoiceList = client.extract(EXTRACTION_PREAMBLE, &content).await?;

        Ok(extracted.into_parsed_invoices(key))
    }
}

impl Default for LLMInvoiceParser {
    fn default() -> Self {
        Self
    }
}

impl<O> Parser<O> for LLMInvoiceParser
where
    O: OcrProcessedDocument + Send,
{
    type Output = ParsedInvoices;

    async fn parse<L: LLMClient + Send + Sync>(
        &self,
        client: &L,
        ocr_result: Option<O>,
        _bytes: Option<&[u8]>,
    ) -> Result<Self::Output> {
        let ocr = ocr_result.ok_or(Error::EmptyOcrContent)?;
        LLMInvoiceParser::parse(self, client, ocr).await
    }
}
