use crate::error::Result;
use crate::traits::parsed_document::{ParsedDocument, ParsedPayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedSoWs {
    pub key: String,
    pub results: Vec<SoW>,
}

impl ParsedPayload for ParsedSoWs {
    fn key(&self) -> &str {
        &self.key
    }
}

impl ParsedDocument<SoW> for ParsedSoWs {
    fn results(&self) -> Result<Vec<SoW>> {
        Ok(self.results.clone())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoW {
    pub header: SoWHeader,
    pub rates: Vec<SoWRateLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoWHeader {
    pub vendor: String,
    pub valid_from: String,
    pub valid_until: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoWRateLine {
    pub service_name: String,
    pub is_rate_range: bool,
    pub rate: Option<f64>,
    pub rate_range_min: Option<f64>,
    pub rate_range_max: Option<f64>,
    pub unit_of_measure: String,
    pub language_location: Option<String>,
    pub comment: Option<String>,
}
