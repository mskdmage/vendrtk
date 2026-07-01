use crate::error::Result;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::output::VendorReconciliationOutput;
use crate::traits::stage::Stage;

pub enum ParseStage {
    ParseInvoice { key: String },
    ParseSoW { key: String },
}

impl Stage<VendorReconciliationContext> for ParseStage {
    type Output = VendorReconciliationOutput;

    async fn run(self, _context: &mut VendorReconciliationContext) -> Result<Self::Output> {
        match self {
            ParseStage::ParseInvoice { key } => Ok(VendorReconciliationOutput::Invoice(key)),
            ParseStage::ParseSoW { key } => Ok(VendorReconciliationOutput::SoW(key)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_parse_invoice_produces_invoice_output() {
        let stage = ParseStage::ParseInvoice {
            key: "parsed-invoice".into(),
        };
        let mut ctx = VendorReconciliationContext::default();

        let output = stage.run(&mut ctx).await.unwrap();

        assert!(matches!(
            output,
            VendorReconciliationOutput::Invoice(ref key) if key == "parsed-invoice"
        ));
    }

    #[tokio::test]
    async fn test_run_parse_sow_produces_sow_output() {
        let stage = ParseStage::ParseSoW {
            key: "parsed-sow".into(),
        };
        let mut ctx = VendorReconciliationContext::default();

        let output = stage.run(&mut ctx).await.unwrap();

        assert!(matches!(
            output,
            VendorReconciliationOutput::SoW(ref key) if key == "parsed-sow"
        ));
    }
}
