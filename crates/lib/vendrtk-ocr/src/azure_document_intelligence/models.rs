use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeOperationResponse {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated_date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analyze_result: Option<AnalyzeResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<AnalyzeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeError {
    pub code: Option<String>,
    pub message: Option<String>,
}

/// `analyzeResult` payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResult {
    pub api_version: Option<String>,
    pub model_id: Option<String>,
    pub string_index_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default)]
    pub pages: Vec<DocumentPage>,
    #[serde(default)]
    pub tables: Vec<DocumentTable>,
    #[serde(default)]
    pub paragraphs: Vec<DocumentParagraph>,
    #[serde(default)]
    pub key_value_pairs: Vec<DocumentKeyValuePair>,
    #[serde(default)]
    pub styles: Vec<DocumentStyle>,
    #[serde(default)]
    pub documents: Vec<AnalyzedDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPage {
    pub page_number: i32,
    #[serde(default)]
    pub angle: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub words: Vec<DocumentWord>,
    #[serde(default)]
    pub lines: Vec<DocumentLine>,
    #[serde(default)]
    pub selection_marks: Vec<DocumentSelectionMark>,
    #[serde(default)]
    pub barcodes: Vec<DocumentBarcode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentWord {
    pub content: String,
    #[serde(default)]
    pub polygon: Vec<f64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub span: Option<DocumentSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLine {
    pub content: String,
    #[serde(default)]
    pub polygon: Vec<f64>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSelectionMark {
    pub state: String,
    #[serde(default)]
    pub polygon: Vec<f64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub span: Option<DocumentSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentBarcode {
    pub kind: String,
    pub value: String,
    #[serde(default)]
    pub polygon: Vec<f64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub span: Option<DocumentSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentParagraph {
    pub content: String,
    #[serde(default)]
    pub bounding_regions: Vec<BoundingRegion>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTable {
    pub row_count: i32,
    pub column_count: i32,
    #[serde(default)]
    pub cells: Vec<DocumentTableCell>,
    #[serde(default)]
    pub bounding_regions: Vec<BoundingRegion>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTableCell {
    #[serde(default)]
    pub kind: Option<String>,
    pub row_index: i32,
    pub column_index: i32,
    #[serde(default)]
    pub row_span: Option<i32>,
    #[serde(default)]
    pub column_span: Option<i32>,
    pub content: String,
    #[serde(default)]
    pub bounding_regions: Vec<BoundingRegion>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentKeyValuePair {
    pub key: DocumentKeyValueElement,
    #[serde(default)]
    pub value: Option<DocumentKeyValueElement>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentKeyValueElement {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub bounding_regions: Vec<BoundingRegion>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentStyle {
    #[serde(default)]
    pub is_handwritten: Option<bool>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
}

/// Extracted document (e.g. `docType: "invoice"` for prebuilt-invoice).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzedDocument {
    pub doc_type: String,
    #[serde(default)]
    pub bounding_regions: Vec<BoundingRegion>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
    #[serde(default)]
    pub fields: HashMap<String, DocumentField>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Field from a prebuilt or custom model (`VendorName`, `Items`, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentField {
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub bounding_regions: Vec<BoundingRegion>,
    #[serde(default)]
    pub spans: Vec<DocumentSpan>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub value_string: Option<String>,
    #[serde(default)]
    pub value_date: Option<String>,
    #[serde(default)]
    pub value_time: Option<String>,
    #[serde(default)]
    pub value_phone_number: Option<String>,
    #[serde(default)]
    pub value_number: Option<f64>,
    #[serde(default)]
    pub value_integer: Option<i64>,
    #[serde(default)]
    pub value_array: Vec<DocumentField>,
    #[serde(default)]
    pub value_object: HashMap<String, DocumentField>,
    #[serde(default)]
    pub value_currency: Option<CurrencyValue>,
    #[serde(default)]
    pub value_address: Option<AddressValue>,
    #[serde(default)]
    pub value_country_region: Option<String>,
    #[serde(default)]
    pub value_selection_mark: Option<String>,
    #[serde(default)]
    pub value_signature: Option<String>,
    #[serde(default)]
    pub value_boolean: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyValue {
    pub amount: f64,
    #[serde(default)]
    pub currency_code: Option<String>,
    #[serde(default)]
    pub currency_symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressValue {
    #[serde(default)]
    pub house_number: Option<String>,
    #[serde(default)]
    pub road: Option<String>,
    #[serde(default)]
    pub street_address: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub country_region: Option<String>,
    #[serde(default)]
    pub po_box: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingRegion {
    pub page_number: i32,
    #[serde(default)]
    pub polygon: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSpan {
    pub offset: i32,
    pub length: i32,
}
