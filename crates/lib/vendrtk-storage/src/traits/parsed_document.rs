use crate::error::Result;
pub trait ParsedDocument<T> {
    fn key(&self) -> &str;
    fn results(&self) -> Result<Vec<T>>;
}