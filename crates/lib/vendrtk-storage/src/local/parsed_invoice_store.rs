use std::collections::HashMap;
use std::path::PathBuf;

use chrono::Utc;
use std::fs::DirBuilder;
use tracing::debug;

use crate::{
    error::{Error, Result},
    models::{invoice::ParsedInvoices, ledger::LedgerEntry},
    traits::store::{ProcessedDocumentStore, Store},
};

pub struct LocalParsedInvoiceStore {
    store_root: PathBuf,
    ledger_path: PathBuf,
    ledger: HashMap<String, LedgerEntry>,
}

impl LocalParsedInvoiceStore {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let store_root = path.as_ref().to_path_buf();
        let ledger_path = store_root.join(".ledger.json");

        DirBuilder::new().recursive(true).create(&store_root).map_err(Error::Io)?;

        if !ledger_path.exists() {
            std::fs::write(&ledger_path, "{}").map_err(Error::Io)?;
        }

        let ledger = Self::load_ledger(&ledger_path)?;
        debug!("Loaded {} invoice entries from ledger.", ledger.len());

        Ok(Self { store_root, ledger_path, ledger })
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

impl Store<LedgerEntry> for LocalParsedInvoiceStore {
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

impl ProcessedDocumentStore<ParsedInvoices> for LocalParsedInvoiceStore {
    fn save(&mut self, payload: ParsedInvoices) -> Result<()> {
        let key = payload.key.clone();
        let now = Utc::now();
        let entry = match self.ledger.get(&key) {
            Some(e) => LedgerEntry { key: key.clone(), created_at: e.created_at, updated_at: now },
            None    => LedgerEntry { key: key.clone(), created_at: now, updated_at: now },
        };
        debug!("Saving invoice payload: {key}");
        std::fs::write(
            self.payload_path(&key),
            serde_json::to_string_pretty(&payload).map_err(Error::Json)?,
        ).map_err(Error::Io)?;
        self.ledger.insert(key, entry);
        self.save_ledger()
    }

    fn load_payload(&self, key: &str) -> Result<Option<ParsedInvoices>> {
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
    use crate::models::invoice::{Invoice, InvoiceDetail, InvoiceHeader};
    use std::{path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("vendrtk-inv-{label}-{}-{unique}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &std::path::Path { &self.0 }
    }
    impl Drop for TempDir { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

    fn sample(key: &str) -> ParsedInvoices {
        ParsedInvoices {
            key: key.into(),
            results: vec![Invoice {
                header: InvoiceHeader {
                    vendor: "ACME".into(),
                    invoice_number: "INV-001".into(),
                    invoice_date: "2024-01-01".into(),
                    facility: "Facility A".into(),
                    billing_start: "2024-01-01".into(),
                    billing_end: "2024-01-31".into(),
                    invoice_amount: 100.0,
                },
                details: vec![InvoiceDetail {
                    service_name: "Coding".into(),
                    coder_name: None,
                    account_number: None,
                    service_facility: None,
                    service_date: None,
                    service_description: None,
                    patient_type: None,
                    admit_date: None,
                    discharge_date: None,
                    final_coded_drg: None,
                    quantity: None,
                    unit_of_measure: "hrs".into(),
                    rate: None,
                    amount: 100.0,
                }],
            }],
        }
    }

    #[test]
    fn new_creates_empty_store_with_ledger() {
        let dir = TempDir::new("new");
        let store = LocalParsedInvoiceStore::new(dir.path()).unwrap();
        assert!(dir.0.join(".ledger.json").is_file());
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn save_and_load_payload() {
        let dir = TempDir::new("save");
        let mut store = LocalParsedInvoiceStore::new(dir.path()).unwrap();
        let doc = sample("inv-001");
        store.save(doc.clone()).unwrap();
        assert!(store.exists("inv-001").unwrap());
        assert_eq!(store.load_payload("inv-001").unwrap(), Some(doc));
    }

    #[test]
    fn delete_removes_entry_and_file() {
        let dir = TempDir::new("delete");
        let mut store = LocalParsedInvoiceStore::new(dir.path()).unwrap();
        store.save(sample("inv-001")).unwrap();
        store.delete("inv-001").unwrap();
        assert!(!store.exists("inv-001").unwrap());
        assert!(!dir.0.join("inv-001.json").exists());
    }

    #[test]
    fn entries_persist_across_store_reopen() {
        let dir = TempDir::new("persist");
        let doc = sample("inv-001");
        { let mut s = LocalParsedInvoiceStore::new(dir.path()).unwrap(); s.save(doc.clone()).unwrap(); }
        let store = LocalParsedInvoiceStore::new(dir.path()).unwrap();
        assert_eq!(store.load_payload("inv-001").unwrap(), Some(doc));
    }
}
