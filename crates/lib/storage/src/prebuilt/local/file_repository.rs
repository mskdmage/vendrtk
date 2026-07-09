use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::models::{File, FileKind, FileRef};
use crate::traits::{Blob, Repository};
use tracing::debug;

pub struct LocalRepository {
    root: PathBuf,
}

impl LocalRepository {
    pub fn new(path: &str) -> Result<Self> {
        let root = PathBuf::from(path);
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    // Unsure if this will make a huge difference, but it's a good idea to shard the files
    // into subfolders to avoid the risk of having too many files in a single folder
    // This is a simple implementation that splits the key into 2 characters and uses the first 2 characters as the folder name
    // For example, if the key is "1234567890", the folder name will be "12"
    fn shard(key: &str) -> &str {
        if key.len() >= 2 { &key[..2] } else { "00" }
    }

    fn path_for(&self, kind: FileKind, key: &str) -> PathBuf {
        self.root
            .join(kind.folder())
            .join(Self::shard(key))
            .join(key)
    }

    fn write_file(&self, path: &Path, payload: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, payload)?;
        Ok(())
    }
}

impl Repository<File> for LocalRepository {
    fn put(&self, bytes: &[u8]) -> Result<FileRef> {
        let file = File::from_bytes(bytes.to_vec());
        let key = file.key();
        let kind = file.kind();
        let path = self.path_for(kind, &key);

        if !path.exists() {
            debug!(
                "writing blob: key={} kind={:?} path={}",
                key,
                kind,
                path.display()
            );
            self.write_file(&path, bytes)?;
        } else {
            debug!("blob already exists: key={} kind={:?}", key, kind);
        }

        Ok(FileRef { key, kind })
    }

    fn get(&self, kind: FileKind, key: &str) -> Result<Option<File>> {
        let path = self.path_for(kind, key);
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path)?;
        Ok(Some(File::from_bytes(bytes)))
    }

    fn exists(&self, kind: FileKind, key: &str) -> Result<bool> {
        Ok(self.path_for(kind, key).exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Blob;

    fn sample_pdf() -> Vec<u8> {
        b"%PDF-1.4 test".to_vec()
    }

    fn temp_root() -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("local_repo_{n}"))
    }

    #[test]
    fn test_put_writes_content_addressed_blob() {
        let root = temp_root();
        let repo = LocalRepository::new(root.to_str().unwrap()).unwrap();

        let file_ref = repo.put(&sample_pdf()).unwrap();

        assert_eq!(file_ref.kind, FileKind::Pdf);
        assert!(repo.exists(file_ref.kind, &file_ref.key).unwrap());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_put_is_idempotent() {
        let root = temp_root();
        let repo = LocalRepository::new(root.to_str().unwrap()).unwrap();

        let first = repo.put(&sample_pdf()).unwrap();
        let second = repo.put(&sample_pdf()).unwrap();

        assert_eq!(first, second);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_get_returns_none_for_missing_key() {
        let root = temp_root();
        let repo = LocalRepository::new(root.to_str().unwrap()).unwrap();

        assert!(repo
            .get(FileKind::Pdf, "missing")
            .unwrap()
            .is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_get_round_trips_stored_file() {
        let root = temp_root();
        let repo = LocalRepository::new(root.to_str().unwrap()).unwrap();
        let bytes = sample_pdf();
        let file_ref = repo.put(&bytes).unwrap();

        let loaded = repo.get(file_ref.kind, &file_ref.key).unwrap().unwrap();
        assert_eq!(loaded.bytes(), bytes);
        let _ = fs::remove_dir_all(root);
    }
}
