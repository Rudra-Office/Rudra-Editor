//! Plain text reader/writer for s1engine.
//!
//! Reads plain text files with automatic encoding detection (UTF-8, UTF-16,
//! Latin-1 fallback) and writes documents as UTF-8 plain text.
//!
//! # Reading
//! ```ignore
//! let result = s1_format_txt::read(bytes)?;
//! let doc = result.document;
//! let encoding = result.encoding;
//! ```
//!
//! # Writing
//! ```ignore
//! let text = s1_format_txt::write_string(&doc);
//! let bytes = s1_format_txt::write(&doc);
//! ```

pub mod error;
pub mod reader;
pub mod writer;

// Re-export primary types at crate root.
pub use error::TxtError;
pub use reader::{read, DetectedEncoding, ReadResult};
pub use writer::{write, write_string};
