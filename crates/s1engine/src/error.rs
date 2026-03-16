//! Unified error type for s1engine.

/// Top-level error type for s1engine operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Generic format error (used when a specific variant is not available).
    #[error("Format error: {0}")]
    Format(String),
    /// DOCX format error (preserves the original error for inspection).
    #[cfg(feature = "docx")]
    #[error("DOCX format error: {0}")]
    Docx(#[from] s1_format_docx::DocxError),
    /// ODT format error (preserves the original error for inspection).
    #[cfg(feature = "odt")]
    #[error("ODT format error: {0}")]
    Odt(#[from] s1_format_odt::OdtError),
    /// TXT format error (preserves the original error for inspection).
    #[cfg(feature = "txt")]
    #[error("TXT format error: {0}")]
    Txt(#[from] s1_format_txt::TxtError),
    /// PDF export error (preserves the original error for inspection).
    #[cfg(feature = "pdf")]
    #[error("PDF error: {0}")]
    Pdf(#[from] s1_format_pdf::PdfError),
    /// Error from an operation (insert, delete, etc.).
    #[error("Operation error: {0}")]
    Operation(#[from] s1_ops::OperationError),
    /// I/O error (file read/write).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The requested format is not supported or not enabled via feature flags.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    /// Error from the CRDT collaboration subsystem.
    #[cfg(feature = "crdt")]
    #[error("CRDT error: {0}")]
    Crdt(#[from] s1_crdt::CrdtError),
    /// Error from the layout engine.
    #[cfg(feature = "layout")]
    #[error("Layout error: {0}")]
    Layout(#[from] s1_layout::LayoutError),
}
