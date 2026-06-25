//! Loose SoW / rate-schedule types for LLM extraction.
use crate::models::sow::{ParsedSoWs, SoW, SoWHeader, SoWRateLine};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Statement of work: agreement fields + rate schedule as read from the document.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedSow {
    pub header: ExtractedSowHeader,
    #[schemars(
        title = "Rate schedule",
        description = "One row per billable service/rate from the SoW tables. Include the page where each rate appears. Do not repeat agreement dates on each line."
    )]
    pub rates: Vec<ExtractedSowRateLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedSowHeader {
    #[schemars(title = "Vendor Name", description = "Vendor legal or trade name.")]
    pub vendor_name: String,

    #[schemars(
        title = "Valid From",
        description = "Agreement effective start for all rates in this SoW. Prefer YYYY-MM-DD."
    )]
    pub valid_from: String,

    #[schemars(
        title = "Valid Until",
        description = "Agreement effective end for all rates in this SoW. Prefer YYYY-MM-DD."
    )]
    pub valid_until: String,

    #[schemars(title = "Comment", description = "Free-text notes for the agreement.")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedSowRateLine {
    #[schemars(title = "Service Name", description = "Billable service label from the rate table.")]
    pub service_name: String,

    #[schemars(
        title = "Is Rate Range",
        description = "True if pricing is a min/max range instead of a single rate."
    )]
    pub is_rate_range: bool,

    #[schemars(
        title = "Rate",
        description = "Unit price when is_rate_range is false."
    )]
    pub rate: Option<f64>,

    #[schemars(title = "Rate Range Min", description = "Minimum unit price when is_rate_range is true.")]
    pub rate_range_min: Option<f64>,

    #[schemars(title = "Rate Range Max", description = "Maximum unit price when is_rate_range is true.")]
    pub rate_range_max: Option<f64>,

    #[schemars(
        title = "Unit of Measure",
        description = "Unit of measure, e.g. Each, Hour, Page."
    )]
    #[serde(alias = "type", alias = "unit_type")]
    pub unit_of_measure: String,

    #[schemars(
        title = "Language / Location",
        description = "Page or section where this rate appears, e.g. 'Schedule A Page 10'."
    )]
    pub language_location: Option<String>,

    #[schemars(title = "Comment", description = "Notes for this rate line.")]
    pub comment: Option<String>,
}

impl ExtractedSow {
    pub fn into_parsed_sows(self, key: impl Into<String>) -> ParsedSoWs {
        ParsedSoWs {
            key: key.into(),
            results: vec![SoW {
                header: SoWHeader {
                    vendor: self.header.vendor_name,
                    valid_from: self.header.valid_from,
                    valid_until: self.header.valid_until,
                    comment: self.header.comment,
                },
                rates: self
                    .rates
                    .into_iter()
                    .map(|rate| SoWRateLine {
                        service_name: rate.service_name,
                        is_rate_range: rate.is_rate_range,
                        rate: rate.rate,
                        rate_range_min: rate.rate_range_min,
                        rate_range_max: rate.rate_range_max,
                        unit_of_measure: rate.unit_of_measure,
                        language_location: rate.language_location,
                        comment: rate.comment,
                    })
                    .collect(),
            }],
        }
    }
}
