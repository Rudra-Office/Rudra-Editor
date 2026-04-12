//! RTF reader/writer for s1engine.
//!
//! Reads RTF 1.x into a [`DocumentModel`] and writes back to RTF format.

mod error;
mod reader;
mod writer;

pub use error::RtfError;
pub use reader::read_rtf;
pub use writer::write_rtf;
