use std::sync::Mutex;

use vendrtk::ocr::{
    prebuilt::azure::{
        client::DocumentIntelligenceClient,
        models::AnalyzeOperationResponse,
    },
    traits::OCRClient,
};
use vendrtk::storage::{
    models::FileRef,
    prebuilt::local::{LocalOcrProcessedStore, LocalRepository},
    traits::{Repository, Store},
};

use crate::web::error::{Error, Result};

pub struct VendorReconciliationService {
    pub file_repository: LocalRepository,
    pub ocr_client: DocumentIntelligenceClient,
    pub ocr_processed_store: Mutex<LocalOcrProcessedStore<AnalyzeOperationResponse>>,
}

impl VendorReconciliationService {
    pub fn new() -> Self {
        Self {
            file_repository: LocalRepository::new(".landing").unwrap(),
            ocr_client: DocumentIntelligenceClient::from_env(
                None,
            ).unwrap(),
            ocr_processed_store: Mutex::new(
                LocalOcrProcessedStore::<AnalyzeOperationResponse>::new(".ocr").unwrap(),
            ),
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

        if !self
            .ocr_processed_store
            .lock()
            .map_err(|_| Error::InternalServerError)?
            .exists(&file_ref.key)
            .map_err(|error| {
                tracing::error!("ocr store exists check failed: {}", error);
                Error::InternalServerError
            })?
        {
            match self.ocr_client.analyze_bytes(bytes).await {
                Ok(response) => {
                    if let Err(error) = self
                        .ocr_processed_store
                        .lock()
                        .map_err(|_| Error::InternalServerError)?
                        .create(&file_ref.key, response)
                    {
                        tracing::error!("ocr store create failed: {}", error);
                    }
                }
                Err(error) => tracing::error!("ocr analyze failed: {}", error),
            }
        }

        Ok(file_ref)
    }
    
}
