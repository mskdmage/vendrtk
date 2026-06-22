use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::DirBuilder;
use tracing::debug;

use crate::{
    error::{Error, Result},
    models::ledger::LedgerEntry,
    traits::{
        ocr_processed_document::OcrProcessedDocument,
        store::{ProcessedDocumentStore, Store},
    },
};

pub struct LocalOcrProcessedStore<T: OcrProcessedDocument> {
    store_root: PathBuf,
    ledger_path: PathBuf,
    ledger: HashMap<String, LedgerEntry>,
    _marker: PhantomData<T>,
}

impl<T: OcrProcessedDocument> LocalOcrProcessedStore<T> {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let store_root = path.as_ref().to_path_buf();
        let ledger_path = store_root.join(".ledger.json");

        DirBuilder::new().recursive(true).create(&store_root).map_err(Error::Io)?;

        if !ledger_path.exists() {
            std::fs::write(&ledger_path, "{}").map_err(Error::Io)?;
        }

        let ledger = Self::load_ledger(&ledger_path)?;
        debug!("Loaded {} OCR entries from ledger.", ledger.len());

        Ok(Self { store_root, ledger_path, ledger, _marker: PhantomData })
    }

    fn load_ledger(path: &PathBuf) -> Result<HashMap<String, LedgerEntry>> {
        let raw = std::fs::read_to_string(path).map_err(Error::Io)?;
        if raw.trim().is_empty() {
            return Ok(HashMap::new());
        }
        serde_json::from_str(&raw).map_err(Error::Json)
    }

    fn save_ledger(&self) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&self.ledger).map_err(Error::Json)?;
        std::fs::write(&self.ledger_path, serialized).map_err(Error::Io)?;
        Ok(())
    }

    fn payload_path(&self, key: &str) -> PathBuf {
        self.store_root.join(format!("{key}.json"))
    }
}

impl<T: OcrProcessedDocument> Store<LedgerEntry> for LocalOcrProcessedStore<T> {
    fn create(&mut self, key: &str, entry: LedgerEntry) -> Result<()> {
        self.ledger.insert(key.to_string(), entry);
        self.save_ledger()
    }

    fn get(&self, key: &str) -> Result<Option<LedgerEntry>> {
        Ok(self.ledger.get(key).cloned())
    }

    fn update(&mut self, key: &str, entry: LedgerEntry) -> Result<Option<LedgerEntry>> {
        if !self.ledger.contains_key(key) {
            return Ok(None);
        }
        let previous = self.ledger.insert(key.to_string(), entry);
        self.save_ledger()?;
        Ok(previous)
    }

    fn delete(&mut self, key: &str) -> Result<Option<LedgerEntry>> {
        let removed = self.ledger.remove(key);
        if removed.is_some() {
            let path = self.payload_path(key);
            if path.exists() {
                std::fs::remove_file(&path).map_err(Error::Io)?;
            }
            self.save_ledger()?;
        }
        Ok(removed)
    }

    fn list(&self) -> Result<Vec<LedgerEntry>> {
        Ok(self.ledger.values().cloned().collect())
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.ledger.contains_key(key))
    }
}

impl<T> ProcessedDocumentStore<T> for LocalOcrProcessedStore<T>
where
    T: OcrProcessedDocument + Serialize + for<'de> Deserialize<'de>,
{
    fn save(&mut self, payload: T) -> Result<()> {
        let key = payload.key().to_string();
        let now = Utc::now();
        let entry = match self.ledger.get(&key) {
            Some(e) => LedgerEntry { key: key.clone(), created_at: e.created_at, updated_at: now },
            None    => LedgerEntry { key: key.clone(), created_at: now, updated_at: now },
        };
        debug!("Saving OCR payload: {key}");
        std::fs::write(
            self.payload_path(&key),
            serde_json::to_string_pretty(&payload).map_err(Error::Json)?,
        ).map_err(Error::Io)?;
        self.ledger.insert(key, entry);
        self.save_ledger()
    }

    fn load_payload(&self, key: &str) -> Result<Option<T>> {
        if !self.ledger.contains_key(key) {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(self.payload_path(key)).map_err(Error::Io)?;
        Ok(Some(serde_json::from_str(&raw).map_err(Error::Json)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestOcrDoc { id: String, raw: String }

    impl OcrProcessedDocument for TestOcrDoc {
        fn key(&self) -> &str { &self.id }
        fn raw_content(&self) -> Result<String> { Ok(self.raw.clone()) }
        fn pages(&self) -> Result<Vec<String>> { Ok(vec![]) }
    }

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("vendrtk-ocr-{label}-{}-{unique}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &std::path::Path { &self.0 }
    }
    impl Drop for TempDir { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

    fn sample(id: &str) -> TestOcrDoc { TestOcrDoc { id: id.into(), raw: format!("raw {id}") } }

    #[test]
    fn new_creates_empty_store_with_ledger() {
        let dir = TempDir::new("new");
        let store = LocalOcrProcessedStore::<TestOcrDoc>::new(dir.path()).unwrap();
        assert!(dir.0.join(".ledger.json").is_file());
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn save_writes_payload_file_and_ledger_entry() {
        let dir = TempDir::new("save");
        let mut store = LocalOcrProcessedStore::new(dir.path()).unwrap();
        store.save(sample("fp-123")).unwrap();
        assert!(store.exists("fp-123").unwrap());
        assert!(dir.0.join("fp-123.json").is_file());
        assert_eq!(store.load_payload("fp-123").unwrap(), Some(sample("fp-123")));
    }

    #[test]
    fn save_upserts_existing_entry_preserving_created_at() {
        let dir = TempDir::new("upsert");
        let mut store = LocalOcrProcessedStore::new(dir.path()).unwrap();
        store.save(sample("fp-123")).unwrap();
        let created_at = store.get("fp-123").unwrap().unwrap().created_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        store.save(TestOcrDoc { id: "fp-123".into(), raw: "updated".into() }).unwrap();
        let entry = store.get("fp-123").unwrap().unwrap();
        assert_eq!(entry.created_at, created_at);
        assert!(entry.updated_at > created_at);
    }

    #[test]
    fn delete_removes_ledger_entry_and_payload_file() {
        let dir = TempDir::new("delete");
        let mut store = LocalOcrProcessedStore::new(dir.path()).unwrap();
        store.save(sample("fp-123")).unwrap();
        store.delete("fp-123").unwrap();
        assert!(!store.exists("fp-123").unwrap());
        assert!(!dir.0.join("fp-123.json").exists());
    }

    #[test]
    fn load_payload_returns_none_for_missing_key() {
        let dir = TempDir::new("missing");
        let store = LocalOcrProcessedStore::<TestOcrDoc>::new(dir.path()).unwrap();
        assert_eq!(store.load_payload("nope").unwrap(), None);
    }

    #[test]
    fn entries_persist_across_store_reopen() {
        let dir = TempDir::new("persist");
        let doc = sample("fp-123");
        { let mut s = LocalOcrProcessedStore::new(dir.path()).unwrap(); s.save(doc.clone()).unwrap(); }
        let store = LocalOcrProcessedStore::new(dir.path()).unwrap();
        assert_eq!(store.load_payload("fp-123").unwrap(), Some(doc));
    }
}
