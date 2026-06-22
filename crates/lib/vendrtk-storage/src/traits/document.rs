use std::path::Path;

pub trait Document {
    fn key(&self) -> &str;
    fn path(&self) -> &Path;
    fn size(&self) -> u64;
    fn extension(&self) -> &str;
}
