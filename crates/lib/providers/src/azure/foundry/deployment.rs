#[derive(Clone, Copy)]
pub enum Deployment {
    Gpt52,
    Default,
}

impl AsRef<str> for Deployment {
    fn as_ref(&self) -> &str {
        match self {
            Deployment::Gpt52 => "gpt-5.2",
            Deployment::Default => "gpt-5.2",
        }
    }
}
