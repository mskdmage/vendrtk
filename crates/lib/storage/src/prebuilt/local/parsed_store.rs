use std::fs;
use std::marker::PhantomData;
use std::path::PathBuf;
use tracing::debug;
use parsers::traits::ParsedPayload;

use crate::error::{Error, Result};
use crate::traits::Store;

pub struct LocalParsedStore<T: ParsedPayload> {
    root: PathBuf,
    _marker: PhantomData<T>,
}

impl<T: ParsedPayload> LocalParsedStore<T> {
    pub fn new(path: &str) -> Result<Self> {
        let root = PathBuf::from(path);
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            _marker: PhantomData::<T>,
        })
    }

    fn shard(key: &str) -> &str {
        if key.len() >= 2 { &key[..2] } else { "00" }
    }

    fn path_for(&self, key: &str) -> PathBuf {
        self.root
            .join(Self::shard(key))
            .join(format!("{key}.json"))
    }

    fn write_entity(&self, key: &str, entity: &T) -> Result<()> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec(entity)?;
        debug!(
            "writing parsed result: key={} path={}",
            key,
            path.display()
        );
        fs::write(path, bytes)?;
        Ok(())
    }

    fn read_entity(&self, key: &str) -> Result<Option<T>> {
        let path = self.path_for(key);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path)?;
        Ok(Some(serde_json::from_slice(&bytes)?))
    }
}

impl<T: ParsedPayload> Store<T> for LocalParsedStore<T> {
    fn create(&mut self, key: &str, entity: T) -> Result<()> {
        if self.exists(key)? {
            return Err(Error::Repository(format!("key already exists: {key}")));
        }
        self.write_entity(key, &entity)
    }

    fn get(&self, key: &str) -> Result<Option<T>> {
        self.read_entity(key)
    }

    fn update(&mut self, key: &str, entity: T) -> Result<Option<T>> {
        let previous = self.get(key)?;
        if previous.is_none() {
            return Ok(None);
        }
        self.write_entity(key, &entity)?;
        Ok(previous)
    }

    fn delete(&mut self, key: &str) -> Result<Option<T>> {
        let previous = self.get(key)?;
        if previous.is_some() {
            fs::remove_file(self.path_for(key))?;
        }
        Ok(previous)
    }

    fn list(&self) -> Result<Vec<T>> {
        let mut items = Vec::new();
        if !self.root.exists() {
            return Ok(items);
        }

        for shard_entry in fs::read_dir(&self.root)? {
            let shard_path = shard_entry?.path();
            if !shard_path.is_dir() {
                continue;
            }
            for file_entry in fs::read_dir(shard_path)? {
                let path = file_entry?.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let bytes = fs::read(&path)?;
                items.push(serde_json::from_slice(&bytes)?);
            }
        }

        Ok(items)
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.path_for(key).exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct SampleEntity {
        key: String,
        status: String,
    }

    impl ParsedPayload for SampleEntity {
        fn key(&self) -> &str {
            &self.key
        }
    }

    fn sample_entity() -> SampleEntity {
        SampleEntity {
            key: "sample".into(),
            status: "succeeded".into(),
        }
    }

    fn temp_root() -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("parsed_store_{n}"))
    }

    #[test]
    fn create_and_get_round_trip() {
        let root = temp_root();
        let mut store = LocalParsedStore::<SampleEntity>::new(root.to_str().unwrap()).unwrap();
        let entity = sample_entity();

        store.create("abc123", entity.clone()).unwrap();
        let loaded = store.get("abc123").unwrap().unwrap();

        assert_eq!(loaded, entity);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn create_rejects_duplicate_key() {
        let root = temp_root();
        let mut store = LocalParsedStore::<SampleEntity>::new(root.to_str().unwrap()).unwrap();
        let entity = sample_entity();

        store.create("abc123", entity.clone()).unwrap();
        assert!(store.create("abc123", entity).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_returns_previous_value() {
        let root = temp_root();
        let mut store = LocalParsedStore::<SampleEntity>::new(root.to_str().unwrap()).unwrap();
        let original = sample_entity();
        let mut updated = sample_entity();
        updated.status = "failed".into();

        store.create("abc123", original.clone()).unwrap();
        let previous = store.update("abc123", updated.clone()).unwrap().unwrap();

        assert_eq!(previous, original);
        assert_eq!(store.get("abc123").unwrap().unwrap(), updated);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_missing_key_returns_none() {
        let root = temp_root();
        let mut store = LocalParsedStore::<SampleEntity>::new(root.to_str().unwrap()).unwrap();

        assert!(store
            .update("missing", sample_entity())
            .unwrap()
            .is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn delete_removes_entry() {
        let root = temp_root();
        let mut store = LocalParsedStore::<SampleEntity>::new(root.to_str().unwrap()).unwrap();
        let entity = sample_entity();

        store.create("abc123", entity.clone()).unwrap();
        let deleted = store.delete("abc123").unwrap().unwrap();

        assert_eq!(deleted, entity);
        assert!(!store.exists("abc123").unwrap());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn list_returns_all_entries() {
        let root = temp_root();
        let mut store = LocalParsedStore::<SampleEntity>::new(root.to_str().unwrap()).unwrap();

        store.create("abc123", sample_entity()).unwrap();
        store.create("def456", sample_entity()).unwrap();

        assert_eq!(store.list().unwrap().len(), 2);
        let _ = fs::remove_dir_all(root);
    }
}
