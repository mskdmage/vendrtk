use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::traits::parsed_document::{ParsedDocument, ParsedPayload};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedInvoices {
    pub key: String,
    pub results: Vec<Invoice>,
}

impl ParsedPayload for ParsedInvoices {
    fn key(&self) -> &str {
        &self.key
    }
}

impl ParsedDocument<Invoice> for ParsedInvoices {
    fn results(&self) -> Result<Vec<Invoice>> {
        Ok(self.results.clone())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invoice {
    pub header: InvoiceHeader,
    pub details: Vec<InvoiceDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceHeader {
    pub vendor: String,
    pub invoice_number: String,
    pub invoice_date: String,
    pub facility: String,
    pub billing_start: String,
    pub billing_end: String,
    pub invoice_amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceDetail {
    pub service_name: String,
    pub coder_name: Option<String>,
    pub account_number: Option<String>,
    pub service_facility: Option<String>,
    pub service_date: Option<String>,
    pub service_description: Option<String>,
    pub patient_type: Option<String>,
    pub admit_date: Option<String>,
    pub discharge_date: Option<String>,
    pub final_coded_drg: Option<String>,
    pub quantity: Option<f64>,
    pub unit_of_measure: String,
    pub rate: Option<f64>,
    pub amount: f64,
}