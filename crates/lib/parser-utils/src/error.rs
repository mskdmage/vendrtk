pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pdf extraction failed: {0}")]
    PdfExtraction(#[from] pdf_inspector::PdfError),
}
