#[derive(Clone, Copy)]
pub enum ApiVersion {
    V20241130,
    Default,
}

impl AsRef<str> for ApiVersion {
    fn as_ref(&self) -> &str {
        match self {
            ApiVersion::V20241130 => "2024-11-30",
            ApiVersion::Default => "2024-11-30",
        }
    }
}