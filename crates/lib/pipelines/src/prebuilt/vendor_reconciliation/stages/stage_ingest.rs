use crate::error::Result;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::input::VendorReconciliationInput;
use crate::prebuilt::vendor_reconciliation::stages::stage_ocr::OcrStage;
use crate::traits::stage::Stage;

pub struct IngestStage {
    pub filename: String,
    pub bytes: Vec<u8>,
}

impl From<VendorReconciliationInput> for IngestStage {
    fn from(input: VendorReconciliationInput) -> Self {
        Self {
            filename: input.filename,
            bytes: input.bytes,
        }
    }
}

impl Stage<VendorReconciliationContext> for IngestStage {
    type Output = OcrStage;

    async fn run(self, _context: &mut VendorReconciliationContext) -> Result<Self::Output> {
        Ok(OcrStage { key: self.filename })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> VendorReconciliationInput {
        VendorReconciliationInput {
            filename: "doc.pdf".into(),
            bytes: vec![10, 20, 30],
        }
    }

    #[test]
    fn test_from_maps_input_fields() {
        let stage = IngestStage::from(sample_input());

        assert_eq!(stage.filename, "doc.pdf");
        assert_eq!(stage.bytes, vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn test_run_produces_ocr_stage_with_filename_as_key() {
        let stage = IngestStage::from(sample_input());
        let mut ctx = VendorReconciliationContext::default();

        let ocr = stage.run(&mut ctx).await.unwrap();

        assert_eq!(ocr.key, "doc.pdf");
    }
}
