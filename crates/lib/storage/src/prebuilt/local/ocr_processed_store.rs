use std::fs;
use std::path::PathBuf;

use ocr::prebuilt::azure::models::AnalyzeOperationResponse;
use tracing::debug;

use crate::error::{Error, Result};
use crate::traits::Store;

pub struct LocalOcrProcessedStore {
    root: PathBuf,
}

impl LocalOcrProcessedStore {
    pub fn new(path: &str) -> Result<Self> {
        let root = PathBuf::from(path);
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn shard(key: &str) -> &str {
        if key.len() >= 2 { &key[..2] } else { "00" }
    }

    fn path_for(&self, key: &str) -> PathBuf {
        self.root
            .join(Self::shard(key))
            .join(format!("{key}.json"))
    }

    fn write_entity(&self, key: &str, entity: &AnalyzeOperationResponse) -> Result<()> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec(entity)?;
        debug!(
            "writing ocr result: key={} path={}",
            key,
            path.display()
        );
        fs::write(path, bytes)?;
        Ok(())
    }

    fn read_entity(&self, key: &str) -> Result<Option<AnalyzeOperationResponse>> {
        let path = self.path_for(key);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path)?;
        Ok(Some(serde_json::from_slice(&bytes)?))
    }
}

impl Store<AnalyzeOperationResponse> for LocalOcrProcessedStore {
    fn create(&mut self, key: &str, entity: AnalyzeOperationResponse) -> Result<()> {
        if self.exists(key)? {
            return Err(Error::Repository(format!("key already exists: {key}")));
        }
        self.write_entity(key, &entity)
    }

    fn get(&self, key: &str) -> Result<Option<AnalyzeOperationResponse>> {
        self.read_entity(key)
    }

    fn update(&mut self, key: &str, entity: AnalyzeOperationResponse) -> Result<Option<AnalyzeOperationResponse>> {
        let previous = self.get(key)?;
        if previous.is_none() {
            return Ok(None);
        }
        self.write_entity(key, &entity)?;
        Ok(previous)
    }

    fn delete(&mut self, key: &str) -> Result<Option<AnalyzeOperationResponse>> {
        let previous = self.get(key)?;
        if previous.is_some() {
            fs::remove_file(self.path_for(key))?;
        }
        Ok(previous)
    }

    fn list(&self) -> Result<Vec<AnalyzeOperationResponse>> {
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

    fn sample_response() -> AnalyzeOperationResponse {
        AnalyzeOperationResponse {
            status: "succeeded".into(),
            created_date_time: None,
            last_updated_date_time: None,
            analyze_result: None,
            error: None,
        }
    }

    fn temp_root() -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("ocr_processed_store_{n}"))
    }

    #[test]
    fn create_and_get_round_trip() {
        let root = temp_root();
        let mut store = LocalOcrProcessedStore::new(root.to_str().unwrap()).unwrap();
        let entity = sample_response();

        store.create("abc123", entity.clone()).unwrap();
        let loaded = store.get("abc123").unwrap().unwrap();

        assert_eq!(loaded.status, entity.status);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn create_rejects_duplicate_key() {
        let root = temp_root();
        let mut store = LocalOcrProcessedStore::new(root.to_str().unwrap()).unwrap();
        let entity = sample_response();

        store.create("abc123", entity.clone()).unwrap();
        assert!(store.create("abc123", entity).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_returns_previous_value() {
        let root = temp_root();
        let mut store = LocalOcrProcessedStore::new(root.to_str().unwrap()).unwrap();
        let original = sample_response();
        let mut updated = sample_response();
        updated.status = "failed".into();

        store.create("abc123", original.clone()).unwrap();
        let previous = store.update("abc123", updated.clone()).unwrap().unwrap();

        assert_eq!(previous.status, original.status);
        assert_eq!(store.get("abc123").unwrap().unwrap().status, updated.status);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_missing_key_returns_none() {
        let root = temp_root();
        let mut store = LocalOcrProcessedStore::new(root.to_str().unwrap()).unwrap();

        assert!(store
            .update("missing", sample_response())
            .unwrap()
            .is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn delete_removes_entry() {
        let root = temp_root();
        let mut store = LocalOcrProcessedStore::new(root.to_str().unwrap()).unwrap();
        let entity = sample_response();

        store.create("abc123", entity.clone()).unwrap();
        let deleted = store.delete("abc123").unwrap().unwrap();

        assert_eq!(deleted.status, entity.status);
        assert!(!store.exists("abc123").unwrap());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn list_returns_all_entries() {
        let root = temp_root();
        let mut store = LocalOcrProcessedStore::new(root.to_str().unwrap()).unwrap();

        store.create("abc123", sample_response()).unwrap();
        store.create("def456", sample_response()).unwrap();

        assert_eq!(store.list().unwrap().len(), 2);
        let _ = fs::remove_dir_all(root);
    }
}
