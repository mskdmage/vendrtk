pub enum VendorReconciliationOutput {
    Invoice(String),
    SoW(String),
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_and_sow_variants_hold_keys() {
        let invoice = VendorReconciliationOutput::Invoice("inv".into());
        let sow = VendorReconciliationOutput::SoW("sow".into());
        let unknown = VendorReconciliationOutput::Unknown("unknown".into());

        assert!(matches!(invoice, VendorReconciliationOutput::Invoice(ref key) if key == "inv"));
        assert!(matches!(sow, VendorReconciliationOutput::SoW(ref key) if key == "sow"));
        assert!(
            matches!(unknown, VendorReconciliationOutput::Unknown(ref key) if key == "unknown")
        );
    }
}
