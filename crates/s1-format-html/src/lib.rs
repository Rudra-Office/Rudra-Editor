//! HTML reader/writer for s1engine.
//!
//! Reads HTML5 into a [`DocumentModel`] and writes a [`DocumentModel`] back to
//! valid HTML5 markup. Supports headings, paragraphs, inline formatting (bold,
//! italic, underline, strikethrough), hyperlinks, tables, lists, images, line
//! breaks, and horizontal rules.

mod error;
mod reader;
mod writer;

pub use error::HtmlError;
pub use reader::read_html;
pub use writer::write_html;

use s1_model::DocumentModel;

/// Read HTML bytes into a [`DocumentModel`].
///
/// The input is interpreted as UTF-8 (with lossy conversion for invalid bytes).
///
/// # Errors
///
/// Returns [`HtmlError`] if the document model cannot be constructed.
pub fn read_bytes(input: &[u8]) -> Result<DocumentModel, HtmlError> {
    read_html(input)
}

/// Write a [`DocumentModel`] to HTML bytes (UTF-8).
///
/// # Errors
///
/// Returns [`HtmlError`] if the document cannot be serialized to HTML.
pub fn write_bytes(doc: &DocumentModel) -> Result<Vec<u8>, HtmlError> {
    write_html(doc)
}

/// Write a [`DocumentModel`] to an HTML string.
///
/// # Errors
///
/// Returns [`HtmlError`] if the document cannot be serialized to HTML.
pub fn write_string(doc: &DocumentModel) -> Result<String, HtmlError> {
    let bytes = write_html(doc)?;
    String::from_utf8(bytes).map_err(|e| HtmlError::Write(e.to_string()))
}
