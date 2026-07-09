use crate::web::error::Result;

use super::web::services::vendor_reconciliation::VendorReconciliationService;

pub struct AppState {
    pub vendor_reconciliation_service: VendorReconciliationService,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            vendor_reconciliation_service: VendorReconciliationService::new().await?,
        })
    }
}
