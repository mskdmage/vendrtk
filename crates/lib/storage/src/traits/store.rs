use crate::error::Result;
use serde::{Serialize, de::DeserializeOwned};

pub trait Store<T: Serialize + DeserializeOwned> {
    fn create(&mut self, key: &str, entity: T) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<T>>;
    fn update(&mut self, key: &str, entity: T) -> Result<Option<T>>;
    fn delete(&mut self, key: &str) -> Result<Option<T>>;
    fn list(&self) -> Result<Vec<T>>;
    fn exists(&self, key: &str) -> Result<bool>;
}
