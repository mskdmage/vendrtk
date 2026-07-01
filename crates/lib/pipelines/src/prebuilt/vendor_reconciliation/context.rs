use crate::traits::context::Context;
use std::collections::HashMap;

#[derive(Default)]
pub struct VendorReconciliationContext {
    pub dummy_store: HashMap<String, String>,
}

impl Context for VendorReconciliationContext {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_empty() {
        let ctx = VendorReconciliationContext::default();
        assert!(ctx.dummy_store.is_empty());
    }

    #[test]
    fn test_stores_and_retrieves_values() {
        let mut ctx = VendorReconciliationContext::default();
        ctx.dummy_store.insert("key".into(), "value".into());

        assert_eq!(ctx.dummy_store.get("key"), Some(&"value".to_string()));
    }
}
