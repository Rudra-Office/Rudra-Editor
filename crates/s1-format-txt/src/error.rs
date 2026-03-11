//! Error types for the plain text format crate.

use std::fmt;

/// Error type for TXT format operations.
#[derive(Debug, Clone, PartialEq)]
pub enum TxtError {
    /// The input bytes could not be decoded as valid text.
    DecodingError { encoding: String, message: String },
}

impl fmt::Display for TxtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DecodingError { encoding, message } => {
                write!(f, "Failed to decode as {encoding}: {message}")
            }
        }
    }
}

impl std::error::Error for TxtError {}
