pub mod storage {
    pub mod local {
        pub use vendrtk_storage::local::document_store::LocalDocumentStore;
        pub use vendrtk_storage::local::ocr_processed_store::LocalOcrProcessedStore;
        pub use vendrtk_storage::local::parsed_store::LocalParsedStore;
    }

    pub mod models {
        pub use vendrtk_storage::models::documents::{pdf_from_bytes, PdfDocument};
        pub use vendrtk_storage::models::ledger::LedgerEntry;
        pub use vendrtk_storage::models::ocr_processed_document::DocumentIntelligenceOcrProcessedDocument;
    }

    pub mod traits {
        pub use vendrtk_storage::traits::store::{ProcessedDocumentStore, Store};
        pub use vendrtk_storage::traits::document::Document;
    }
}

pub mod ocr {
    pub mod traits {
        pub use vendrtk_ocr::traits::client::OCRClient;
        pub use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;
    }

    pub mod azure {
        pub use vendrtk_ocr::azure_document_intelligence::client::DocumentIntelligenceClient;
        pub use vendrtk_ocr::azure_document_intelligence::config::Config;
        pub use vendrtk_ocr::azure_document_intelligence::auth::{Auth, Credential};
        pub use vendrtk_ocr::azure_document_intelligence::models::AnalyzeOperationResponse;
        pub use vendrtk_ocr::azure_document_intelligence::api_version::ApiVersion;
    }
}

pub mod parsers {
    pub mod models {
        pub use vendrtk_parsers::models::invoice::{
            Invoice, InvoiceDetail, InvoiceHeader, ParsedInvoices,
        };
        pub use vendrtk_parsers::models::sow::{
            ParsedSoWs, SoW, SoWHeader, SoWRateLine,
        };
    }

    pub mod traits {
        pub use vendrtk_parsers::traits::parsed_document::{ParsedDocument, ParsedPayload};
        pub use vendrtk_parsers::traits::parser::Parser;
    }
}
