//! Error types for the RTF format crate.

/// Error type for RTF format operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum RtfError {
    /// The input does not start with a valid RTF header (`{\rtf1`).
    #[error("Invalid RTF: missing or malformed header")]
    InvalidHeader,

    /// Unexpected end of input while parsing.
    #[error("Unexpected end of input at byte offset {offset}")]
    UnexpectedEof {
        /// Byte offset where the input ended prematurely.
        offset: usize,
    },

    /// Unbalanced braces in the RTF document.
    #[error("Unbalanced braces: {message}")]
    UnbalancedBraces {
        /// Details about the brace mismatch.
        message: String,
    },

    /// A control word parameter could not be parsed as an integer.
    #[error("Invalid control word parameter for \\{word}: {value:?}")]
    InvalidParameter {
        /// The control word name.
        word: String,
        /// The raw parameter string that failed to parse.
        value: String,
    },

    /// The font table references an index that does not exist.
    #[error("Font index {index} not found in font table")]
    FontNotFound {
        /// The missing font index.
        index: usize,
    },

    /// The color table references an index that does not exist.
    #[error("Color index {index} not found in color table")]
    ColorNotFound {
        /// The missing color index.
        index: usize,
    },

    /// A general parse error with context.
    #[error("RTF parse error at byte offset {offset}: {message}")]
    ParseError {
        /// Byte offset where the error occurred.
        offset: usize,
        /// Description of what went wrong.
        message: String,
    },
}
