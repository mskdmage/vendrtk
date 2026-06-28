use super::{
    context::VendorReconciliationContext, input::VendorReconciliationInput,
    output::VendorReconciliationOutput, stages::stage_ingest::IngestStage,
};
use crate::error::Error;
use crate::error::Result;
use crate::traits::{pipeline::Pipeline, stage::Stage};
use std::sync::Arc;
use std::sync::Mutex;

pub struct VendorReconciliationPipeline {
    ctx: Arc<Mutex<VendorReconciliationContext>>,
}

impl VendorReconciliationPipeline {
    pub fn new() -> Self {
        Self {
            ctx: Arc::new(Mutex::new(VendorReconciliationContext::default())),
        }
    }
}

impl Default for VendorReconciliationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline for VendorReconciliationPipeline {
    type Input = VendorReconciliationInput;
    type Output = VendorReconciliationOutput;

    async fn run(&self, input: Self::Input) -> Result<Self::Output> {
        // Getting a clippy message about the lock being poisoned,
        // but we're not using the lock in a multi-threaded context for now (we will),
        // async friendly lock later.
        let mut ctx = self.ctx.lock().map_err(|_| Error::Pipeline)?;

        let ocr = IngestStage::from(input).run(&mut ctx).await?;
        let classify = ocr.run(&mut ctx).await?;
        let parse = classify.run(&mut ctx).await?;
        parse.run(&mut ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prebuilt::vendor_reconciliation::output::VendorReconciliationOutput;

    fn sample_input() -> VendorReconciliationInput {
        VendorReconciliationInput {
            filename: "invoice.pdf".into(),
            bytes: vec![1, 2, 3],
        }
    }

    #[tokio::test]
    async fn test_run_returns_invoice_output_for_sample_input() {
        let pipeline = VendorReconciliationPipeline::new();
        let output = pipeline.run(sample_input()).await.unwrap();

        assert!(
            matches!(output, VendorReconciliationOutput::Invoice(ref key) if key == "invoice.pdf")
        );
    }

    #[tokio::test]
    async fn test_run_uses_shared_context_across_stages() {
        let pipeline = VendorReconciliationPipeline::new();
        {
            let mut ctx = pipeline.ctx.lock().unwrap();
            ctx.dummy_store.insert("seed".into(), "value".into());
        }

        pipeline.run(sample_input()).await.unwrap();

        let ctx = pipeline.ctx.lock().unwrap();
        assert_eq!(ctx.dummy_store.get("seed"), Some(&"value".to_string()));
    }

    #[test]
    fn test_default_constructs_pipeline() {
        let pipeline = VendorReconciliationPipeline::default();
        assert!(pipeline.ctx.lock().is_ok());
    }
}
