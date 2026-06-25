use tokio::sync::Mutex;

use crate::services::error::Result;
use crate::services::vendor_reconciliation::VendorReconciliationService;

pub struct AppState {
    pub(crate) vendor_reconciliation_service: Mutex<VendorReconciliationService>,
}

impl AppState {
    pub async fn new(
        landing_dir: &str,
        ocr_dir: &str,
        parsed_invoices_dir: &str,
        parsed_sows_dir: &str,
    ) -> Result<Self> {
        Ok(Self {
            vendor_reconciliation_service: Mutex::new(
                VendorReconciliationService::new(
                    landing_dir,
                    ocr_dir,
                    parsed_invoices_dir,
                    parsed_sows_dir,
                )
                .await?,
            ),
        })
    }
}
