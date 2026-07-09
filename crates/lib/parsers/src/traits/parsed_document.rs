use crate::error::Result;

pub trait ParsedPayload {
    fn key(&self) -> &str;
}

pub trait ParsedDocument<T>: ParsedPayload {
    fn results(&self) -> Result<Vec<T>>;
}
