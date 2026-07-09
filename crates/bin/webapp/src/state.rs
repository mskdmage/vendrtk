use super::web::services::vendor_reconciliation::VendorReconciliationService;

pub struct AppState {
    pub vendor_reconciliation_service: VendorReconciliationService,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            vendor_reconciliation_service: VendorReconciliationService::new(),
        }
    }
}