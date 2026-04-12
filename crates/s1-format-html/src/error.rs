//! Error types for the HTML format crate.

/// Errors produced by the HTML format crate.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum HtmlError {
    /// The input is not valid UTF-8.
    #[error("invalid UTF-8 in HTML input: {0}")]
    InvalidUtf8(String),

    /// A document model insertion error.
    #[error("model error: {0}")]
    Model(String),

    /// An error during HTML parsing.
    #[error("HTML parse error: {0}")]
    Parse(String),

    /// An error during HTML writing.
    #[error("HTML write error: {0}")]
    Write(String),
}
