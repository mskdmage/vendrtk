use vendrtk::storage::{
    models::FileRef,
    prebuilt::local::LocalRepository,
    traits::Repository,
};

use crate::web::error::{Error, Result};

pub struct VendorReconciliationService {
    pub file_repository: LocalRepository,
}

impl VendorReconciliationService {
    pub fn new() -> Self {
        Self {
            file_repository: LocalRepository::new(".landing").unwrap(),
        }
    }

    pub async fn put_file(&self, bytes: &[u8]) -> Result<FileRef> {
        let file_ref = self
            .file_repository
            .put(bytes)
            .map_err(|error| {
                tracing::error!("repository put failed: {}", error);
                Error::InternalServerError
            })?;

        tracing::info!(
            "file registered: key={} kind={:?}",
            file_ref.key,
            file_ref.kind
        );

        Ok(file_ref)
    }
}
