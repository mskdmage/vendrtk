use crate::error::Result;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::stage_classify::ClassifyStage;
use crate::traits::stage::Stage;

pub struct OcrStage {
    pub key: String,
}

impl Stage<VendorReconciliationContext> for OcrStage {
    type Output = ClassifyStage;

    async fn run(self, _context: &mut VendorReconciliationContext) -> Result<Self::Output> {
        Ok(ClassifyStage { key: self.key })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_produces_classify_stage_with_same_key() {
        let stage = OcrStage {
            key: "ocr:doc.pdf".into(),
        };
        let mut ctx = VendorReconciliationContext::default();

        let classify = stage.run(&mut ctx).await.unwrap();

        assert_eq!(classify.key, "ocr:doc.pdf");
    }
}
