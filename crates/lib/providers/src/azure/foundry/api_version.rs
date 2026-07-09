#[derive(Clone, Copy)]
pub enum ApiVersion {
    V20250401Preview,
    Default,
}

impl AsRef<str> for ApiVersion {
    fn as_ref(&self) -> &str {
        match self {
            ApiVersion::V20250401Preview => "2025-04-01-preview",
            ApiVersion::Default => "2025-04-01-preview",
        }
    }
}
