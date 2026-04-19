use crate::constants::{sig, DOCY_VERSION};
use crate::writer::DocyWriter;

/// Signature table — sdkjs skips this but we write it for completeness.
/// The sdkjs writer uses Read2 format (WriteByte+lenType+WriteLong).
pub fn write(w: &mut DocyWriter) {
    w.write_prop_long(sig::VERSION, DOCY_VERSION);
}
