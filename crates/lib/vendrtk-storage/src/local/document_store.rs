use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use std::fs::DirBuilder;
use tracing::debug;

use crate::{
    error::{Error, Result},
    models::documents::{PdfDocument, pdf_from_bytes},
    traits::{document::Document, store::Store},
};

pub struct LocalDocumentStore<T: Document> {
    store_root: PathBuf,
    ledger_path: PathBuf,
    ledger: HashMap<String, T>,
}

impl<T: Document> LocalDocumentStore<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let store_root = path.as_ref().to_path_buf();
        let ledger_path = store_root.join(".ledger.json");

        DirBuilder::new()
            .recursive(true)
            .create(&store_root)
            .map_err(Error::Io)?;

        if !ledger_path.exists() {
            std::fs::write(&ledger_path, "{}").map_err(Error::Io)?;
        }

        let ledger = Self::load_ledger(&ledger_path)?;
        debug!("Loaded {} entries from ledger.", ledger.len());

        Ok(Self {
            store_root,
            ledger_path,
            ledger,
        })
    }

    fn load_ledger(path: &PathBuf) -> Result<HashMap<String, T>> {
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
}

impl LocalDocumentStore<PdfDocument> {
    pub fn register(&mut self, filename: impl AsRef<std::path::Path>) -> Result<()> {
        let path = self.store_root.join(filename);
        let bytes = std::fs::read(&path).map_err(Error::Io)?;
        let doc = pdf_from_bytes(&path, &bytes)?;
        debug!("Registering document: {}", doc.key);
        self.ledger.insert(doc.key.clone(), doc);
        self.save_ledger()
    }
}

impl<T: Document> Store<T> for LocalDocumentStore<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    fn create(&mut self, key: &str, entity: T) -> Result<()> {
        debug!("Creating new entry for document: {key}");
        self.ledger.insert(key.to_string(), entity);
        self.save_ledger()?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<T>> {
        let found = self.ledger.get(key).cloned();
        debug!("Get document {key}: found={}", found.is_some());
        Ok(found)
    }

    fn update(&mut self, key: &str, entity: T) -> Result<Option<T>> {
        if !self.ledger.contains_key(key) {
            return Ok(None);
        }
        let previous = self.ledger.insert(key.to_string(), entity);
        self.save_ledger()?;
        Ok(previous)
    }

    fn delete(&mut self, key: &str) -> Result<Option<T>> {
        let removed = self.ledger.remove(key);
        if removed.is_some() {
            self.save_ledger()?;
        }
        Ok(removed)
    }

    fn list(&self) -> Result<Vec<T>> {
        let entries: Vec<T> = self.ledger.values().cloned().collect();
        debug!("Listing {} entries.", entries.len());
        Ok(entries)
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let found = self.ledger.contains_key(key);
        debug!("Document {key} exists: {found}");
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestDoc {
        value: String,
    }

    impl Document for TestDoc {
        fn key(&self) -> &str {
            &self.value
        }
        fn path(&self) -> &Path {
            Path::new(&self.value)
        }
        fn size(&self) -> u64 {
            self.value.len() as u64
        }
        fn extension(&self) -> &str {
            "txt"
        }
    }

    struct TempStoreDir(PathBuf);

    impl TempStoreDir {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "vendrtk-storage-{label}-{}-{unique}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &str {
            self.0
                .to_str()
                .expect("temp store path should be valid utf-8")
        }
    }

    impl Drop for TempStoreDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn new_creates_empty_store_with_ledger() {
        let dir = TempStoreDir::new("new");
        let store = LocalDocumentStore::<TestDoc>::new(dir.path()).unwrap();

        assert!(dir.0.join(".ledger.json").is_file());
        assert_eq!(store.list().unwrap(), Vec::<TestDoc>::new());
        assert!(!store.exists("missing").unwrap());
    }

    #[test]
    fn create_get_exists() {
        let dir = TempStoreDir::new("create");
        let mut store = LocalDocumentStore::new(dir.path()).unwrap();
        let doc = TestDoc {
            value: "hello".into(),
        };

        store.create("doc-1", doc.clone()).unwrap();

        assert!(store.exists("doc-1").unwrap());
        assert_eq!(store.get("doc-1").unwrap(), Some(doc));
        assert_eq!(store.get("missing").unwrap(), None);
    }

    #[test]
    fn update_replaces_existing_and_returns_previous() {
        let dir = TempStoreDir::new("update");
        let mut store = LocalDocumentStore::new(dir.path()).unwrap();
        let original = TestDoc {
            value: "original".into(),
        };
        let updated = TestDoc {
            value: "updated".into(),
        };

        store.create("doc-1", original.clone()).unwrap();

        assert_eq!(
            store.update("doc-1", updated.clone()).unwrap(),
            Some(original)
        );
        assert_eq!(store.get("doc-1").unwrap(), Some(updated.clone()));
        assert_eq!(store.update("missing", updated).unwrap(), None);
    }

    #[test]
    fn delete_removes_entry_and_returns_removed_value() {
        let dir = TempStoreDir::new("delete");
        let mut store = LocalDocumentStore::new(dir.path()).unwrap();
        let doc = TestDoc {
            value: "goodbye".into(),
        };

        store.create("doc-1", doc.clone()).unwrap();

        assert_eq!(store.delete("doc-1").unwrap(), Some(doc));
        assert!(!store.exists("doc-1").unwrap());
        assert_eq!(store.delete("missing").unwrap(), None);
    }

    #[test]
    fn list_returns_all_stored_entries() {
        let dir = TempStoreDir::new("list");
        let mut store = LocalDocumentStore::new(dir.path()).unwrap();

        store
            .create(
                "a",
                TestDoc {
                    value: "first".into(),
                },
            )
            .unwrap();
        store
            .create(
                "b",
                TestDoc {
                    value: "second".into(),
                },
            )
            .unwrap();

        let mut entries = store.list().unwrap();
        entries.sort_by(|l, r| l.value.cmp(&r.value));

        assert_eq!(
            entries,
            vec![
                TestDoc {
                    value: "first".into()
                },
                TestDoc {
                    value: "second".into()
                },
            ]
        );
    }

    #[test]
    fn entries_persist_across_store_reopen() {
        let dir = TempStoreDir::new("persist");
        let doc = TestDoc {
            value: "persisted".into(),
        };

        {
            let mut store = LocalDocumentStore::new(dir.path()).unwrap();
            store.create("doc-1", doc.clone()).unwrap();
        }

        let store = LocalDocumentStore::new(dir.path()).unwrap();
        assert_eq!(store.get("doc-1").unwrap(), Some(doc));
    }

    #[test]
    fn register_reads_pdf_from_store_root_and_adds_to_ledger() {
        let dir = TempStoreDir::new("register");
        let mut store = LocalDocumentStore::<PdfDocument>::new(dir.path()).unwrap();

        std::fs::write(dir.0.join("invoice.pdf"), b"%PDF-1.4 sample content").unwrap();

        store.register("invoice.pdf").unwrap();

        let docs = store.list().unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].extension, "pdf");
    }
}
