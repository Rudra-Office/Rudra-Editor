//! Error types for the ODT format crate.

use std::fmt;

/// Error type for ODT format operations.
#[derive(Debug)]
pub enum OdtError {
    /// Error reading the ZIP archive.
    Zip(String),
    /// Error parsing XML content.
    Xml(String),
    /// A required file is missing from the ODT archive.
    MissingFile(String),
    /// The document structure is invalid.
    InvalidStructure(String),
}

impl fmt::Display for OdtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip(msg) => write!(f, "ODT ZIP error: {msg}"),
            Self::Xml(msg) => write!(f, "ODT XML error: {msg}"),
            Self::MissingFile(path) => write!(f, "Missing file in ODT: {path}"),
            Self::InvalidStructure(msg) => write!(f, "Invalid ODT structure: {msg}"),
        }
    }
}

impl std::error::Error for OdtError {}

impl From<zip::result::ZipError> for OdtError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(e.to_string())
    }
}

impl From<quick_xml::Error> for OdtError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e.to_string())
    }
}

impl From<std::io::Error> for OdtError {
    fn from(e: std::io::Error) -> Self {
        Self::Zip(e.to_string())
    }
}
