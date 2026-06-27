use vendrtk_parsers::models::invoice::ParsedInvoices;
use vendrtk_parsers::models::sow::ParsedSoWs;

pub enum VendorReconciliationOutput {
    Invoice(ParsedInvoices),
    Sow(ParsedSoWs),
}
