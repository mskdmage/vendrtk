use crate::{error::Result, models::ledger::LedgerEntry};

pub trait Store<T> {
    fn create(&mut self, key: &str, entity: T) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<T>>;
    fn update(&mut self, key: &str, entity: T) -> Result<Option<T>>;
    fn delete(&mut self, key: &str) -> Result<Option<T>>;
    fn list(&self) -> Result<Vec<T>>;
    fn exists(&self, key: &str) -> Result<bool>;
}

pub trait ProcessedDocumentStore<P>: Store<LedgerEntry> {
    fn save(&mut self, payload: P) -> Result<()>;
    fn load_payload(&self, key: &str) -> Result<Option<P>>;
}
