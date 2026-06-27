use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::OnceCell;

use crate::config::config;
use crate::services::error::Result;
use vendrtk_azure::auth::{Auth, Credential};
use vendrtk_azure::document_intelligence::api_version::ApiVersion;
use vendrtk_azure::document_intelligence::client::DocumentIntelligenceClient;
use vendrtk_azure::document_intelligence::config::Config;
use vendrtk_azure::foundry::client::FoundryClient;
use vendrtk_parsers::models::invoice::ParsedInvoices;
use vendrtk_parsers::models::sow::ParsedSoWs;
use vendrtk_storage::local::document_store::LocalDocumentStore;
use vendrtk_storage::local::ocr_processed_store::LocalOcrProcessedStore;
use vendrtk_storage::local::parsed_store::LocalParsedStore;
use vendrtk_storage::models::documents::PdfDocument;
use vendrtk_storage::models::ocr_processed_document::DocumentIntelligenceOcrProcessedDocument;
use vendrtk_pipelines::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use vendrtk_pipelines::prebuilt::vendor_reconciliation::input::VendorReconciliationInput;
use vendrtk_pipelines::prebuilt::vendor_reconciliation::output::VendorReconciliationOutput;
use vendrtk_pipelines::prebuilt::vendor_reconciliation::pipeline::{
    LocalVendorReconciliationPipeline, VendorReconciliationPipeline,
};
use vendrtk_pipelines::traits::pipeline::Pipeline;

pub struct VendorReconciliationService {
    landing_dir: PathBuf,
    ocr_dir: PathBuf,
    parsed_invoices_dir: PathBuf,
    parsed_sows_dir: PathBuf,
    pipeline: OnceCell<LocalVendorReconciliationPipeline>,
}

impl VendorReconciliationService {
    pub async fn new(
        landing_dir: &str,
        ocr_dir: &str,
        parsed_invoices_dir: &str,
        parsed_sows_dir: &str,
    ) -> Result<Self> {
        Ok(Self {
            landing_dir: landing_dir.into(),
            ocr_dir: ocr_dir.into(),
            parsed_invoices_dir: parsed_invoices_dir.into(),
            parsed_sows_dir: parsed_sows_dir.into(),
            pipeline: OnceCell::new(),
        })
    }

    pub async fn run(&mut self, input: VendorReconciliationInput) -> Result<VendorReconciliationOutput> {
        Ok(self.pipeline().await?.run(input).await?)
    }

    async fn pipeline(&mut self) -> Result<&mut LocalVendorReconciliationPipeline> {
        if self.pipeline.get().is_some() {
            return Ok(self.pipeline.get_mut().expect("pipeline initialized"));
        }

        let cfg = config();
        let auth = if cfg.azure_cognitive_services_key.is_empty() {
            Auth::Credential(Arc::new(Credential::new(None, None, None)?))
        } else {
            Auth::ApiKey(cfg.azure_cognitive_services_key.clone())
        };

        let document_store = LocalDocumentStore::<PdfDocument>::new(&self.landing_dir)?;
        let ocr_store =
            LocalOcrProcessedStore::<DocumentIntelligenceOcrProcessedDocument>::new(&self.ocr_dir)?;
        let parsed_invoice_store =
            LocalParsedStore::<ParsedInvoices>::new(&self.parsed_invoices_dir)?;
        let parsed_sow_store = LocalParsedStore::<ParsedSoWs>::new(&self.parsed_sows_dir)?;

        let ctx = VendorReconciliationContext::new(
            document_store,
            ocr_store,
            parsed_invoice_store,
            parsed_sow_store,
            DocumentIntelligenceClient::new(
                cfg.azure_cognitive_services_endpoint.clone(),
                ApiVersion::Default,
                auth,
                Config::default(),
            )?,
            FoundryClient::connect(
                &cfg.azure_openai_endpoint,
                &cfg.azure_openai_api_version,
                cfg.azure_openai_deployment.clone(),
            )
            .await?,
        );

        let pipeline = VendorReconciliationPipeline::new(ctx);

        let _ = self.pipeline.set(pipeline);
        Ok(self.pipeline.get_mut().expect("pipeline just initialized"))
    }
}
