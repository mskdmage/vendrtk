//! Loose types produced by LLM / Rig extraction (`String`, `f64`).
use crate::models::invoice::{Invoice, InvoiceDetail, InvoiceHeader, ParsedInvoices};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedInvoiceList {
    #[schemars(
        title = "Invoices",
        description = "List of invoices extracted from the invoice text."
    )]
    pub invoices: Vec<ExtractedInvoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedInvoice {
    pub header: ExtractedInvoiceHeader,
    #[schemars(
        title = "Line Items",
        description = "One object per billable row in the invoice detail table. Do not put invoice-level dates or totals here."
    )]
    pub details: Vec<ExtractedInvoiceDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedInvoiceHeader {
    #[schemars(
        title = "Vendor Name",
        description = "Name of the vendor who issued the invoice."
    )]
    pub vendor_name: String,

    #[schemars(
        title = "Invoice Number",
        description = "Number of the invoice."
    )]
    pub invoice_number: String,

    #[schemars(
        title = "Invoice Date",
        description = "Date of invoice issuance. Prefer ISO YYYY-MM-DD. Can be found in the invoice header or footer. May be named 'Period Ending'"
    )]
    pub invoice_date: String,

    #[schemars(
        title = "Facility",
        description = "Bill-to or account facility for the whole invoice (header). Not the per-line origin/site."
    )]
    pub facility: String,

    #[schemars(
        title = "Billing Start",
        description = "Invoice-level start of the billing period (earliest service date on the invoice). Prefer YYYY-MM-DD. Not a line-item field."
    )]
    pub billing_start: String,

    #[schemars(
        title = "Billing End",
        description = "Invoice-level end of the billing period (e.g. period ending / invoice date). Prefer YYYY-MM-DD. Not a line-item field."
    )]
    pub billing_end: String,

    #[schemars(
        title = "Invoice Amount",
        description = "Total amount as read from the invoice. Corresponds to the complete invoice. Do not calculate."
    )]
    pub invoice_amount: f64,
}

/// One row from the invoice line-item / detail table (not header-level fields).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(
    title = "Invoice Line Item",
    description = "A single billable line from the invoice detail section. Use header fields for vendor, billing period, and invoice total."
)]
pub struct ExtractedInvoiceDetail {
    #[schemars(
        title = "Service Name",
        description = "Line item: primary service or charge label for this row (e.g. procedure name, delivery type). Not the invoice vendor."
    )]
    pub service_name: String,

    #[schemars(
        title = "Coder Name",
        description = "Line item: coder who performed work on this row, if shown on that line. Leave empty if only in header. Not invoice-level."
    )]
    pub coder_name: Option<String>,

    #[schemars(
        title = "Account Number",
        description = "Line item: patient medical identifier on this row only — MRN, HPBN, HAR, CSN, visit/account number tied to a patient encounter. Omit vendor or bill-to numbers (e.g. Acct. No. 60005), invoice numbers, and OrdTrkID/tracking IDs."
    )]
    pub account_number: Option<String>,

    #[schemars(
        title = "Service Facility",
        description = "Line item: site, route, or origin label for this row only (e.g. campus, 'JOHNS CREEK', department on that trip). Do not use the bill-to facility from the invoice header."
    )]
    pub service_facility: Option<String>,

    #[schemars(
        title = "Service Date",
        description = "Line item: date the service was performed for this row (e.g. trip date, service day column). Prefer YYYY-MM-DD. Not billing_start/billing_end (invoice period) or invoice_date."
    )]
    pub service_date: Option<String>,

    #[schemars(
        title = "Service Description",
        description = "Line item: full text for this row (origin/destination, tracking ID, charges breakdown). Do not copy the entire invoice header."
    )]
    pub service_description: Option<String>,

    #[schemars(
        title = "Patient Type",
        description = "Line item: patient category for this row (inpatient/outpatient/etc.) when listed on that line. Omit if not on the line."
    )]
    pub patient_type: Option<String>,

    #[schemars(
        title = "Admit Date",
        description = "Line item: patient admit date for this row only. Prefer YYYY-MM-DD. Not billing_start (invoice period)."
    )]
    pub admit_date: Option<String>,

    #[schemars(
        title = "Discharge Date",
        description = "Line item: patient discharge date for this row only. Prefer YYYY-MM-DD. Not billing_end (invoice period)."
    )]
    pub discharge_date: Option<String>,

    #[schemars(
        title = "Final Coded DRG",
        description = "Line item: DRG code for this row when shown on that line. Omit if not on the line."
    )]
    pub final_coded_drg: Option<String>,

    #[schemars(
        title = "Quantity",
        description = "Line item: quantity for this row (e.g. pieces, hours, trips). Not a count of lines on the invoice."
    )]
    pub quantity: Option<f64>,

    #[schemars(
        title = "Unit of Measure",
        description = "Line item: unit for this row (each, hour, trip, page, etc.). Required on every line."
    )]
    pub unit_of_measure: String,

    #[schemars(
        title = "Rate",
        description = "Line item: unit rate for this row before extensions. Omit only if the PDF shows amount without rate."
    )]
    pub rate: Option<f64>,

    #[schemars(
        title = "Amount",
        description = "Line item: total charge for this row only. Must not be the invoice grand total."
    )]
    pub amount: f64,
}

impl ExtractedInvoiceList {
    pub fn into_parsed_invoices(self, key: impl Into<String>) -> ParsedInvoices {
        ParsedInvoices {
            key: key.into(),
            results: self
                .invoices
                .into_iter()
                .map(|invoice| Invoice {
                    header: InvoiceHeader {
                        vendor: invoice.header.vendor_name,
                        invoice_number: invoice.header.invoice_number,
                        invoice_date: invoice.header.invoice_date,
                        facility: invoice.header.facility,
                        billing_start: invoice.header.billing_start,
                        billing_end: invoice.header.billing_end,
                        invoice_amount: invoice.header.invoice_amount,
                    },
                    details: invoice
                        .details
                        .into_iter()
                        .map(|detail| InvoiceDetail {
                            service_name: detail.service_name,
                            coder_name: detail.coder_name,
                            account_number: detail.account_number,
                            service_facility: detail.service_facility,
                            service_date: detail.service_date,
                            service_description: detail.service_description,
                            patient_type: detail.patient_type,
                            admit_date: detail.admit_date,
                            discharge_date: detail.discharge_date,
                            final_coded_drg: detail.final_coded_drg,
                            quantity: detail.quantity,
                            unit_of_measure: detail.unit_of_measure,
                            rate: detail.rate,
                            amount: detail.amount,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}