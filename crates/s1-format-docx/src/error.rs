//! Error types for the DOCX format crate.

use std::fmt;

/// Error type for DOCX format operations.
#[derive(Debug)]
pub enum DocxError {
    /// Error reading the ZIP archive.
    Zip(String),
    /// Error parsing XML content.
    Xml(String),
    /// A required file is missing from the DOCX archive.
    MissingFile(String),
    /// The document structure is invalid.
    InvalidStructure(String),
}

impl fmt::Display for DocxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip(msg) => write!(f, "DOCX ZIP error: {msg}"),
            Self::Xml(msg) => write!(f, "DOCX XML error: {msg}"),
            Self::MissingFile(path) => write!(f, "Missing file in DOCX: {path}"),
            Self::InvalidStructure(msg) => write!(f, "Invalid DOCX structure: {msg}"),
        }
    }
}

impl std::error::Error for DocxError {}

impl From<zip::result::ZipError> for DocxError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(e.to_string())
    }
}

impl From<quick_xml::Error> for DocxError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e.to_string())
    }
}

impl From<std::io::Error> for DocxError {
    fn from(e: std::io::Error) -> Self {
        Self::Zip(e.to_string())
    }
}
