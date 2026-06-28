use crate::error::Result;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::stage_parse::ParseStage;
use crate::traits::stage::Stage;

pub struct ClassifyStage {
    pub key: String,
}

impl Stage<VendorReconciliationContext> for ClassifyStage {
    type Output = ParseStage;

    async fn run(self, _context: &mut VendorReconciliationContext) -> Result<Self::Output> {
        // TODO: Implement classification logic is provided by external service
        // concrete implementation coming soon.
        // For now, we'll assume the document is always an invoice
        Ok(ParseStage::ParseInvoice { key: self.key })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_produces_parse_invoice_variant() {
        let stage = ClassifyStage {
            key: "classify:doc.pdf".into(),
        };
        let mut ctx = VendorReconciliationContext::default();

        let parse = stage.run(&mut ctx).await.unwrap();

        assert!(matches!(
            parse,
            ParseStage::ParseInvoice { key } if key == "classify:doc.pdf"
        ));
    }
}
