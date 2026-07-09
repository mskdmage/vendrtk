use reqwest::header::HeaderName;

pub const SUBSCRIPTION_KEY_HEADER: HeaderName =
    HeaderName::from_static("ocp-apim-subscription-key");
pub const OPERATION_LOCATION_HEADER: &str = "operation-location";
