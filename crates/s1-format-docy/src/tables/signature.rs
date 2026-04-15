use crate::constants::{sig, DOCY_VERSION};
use crate::writer::DocyWriter;

/// Signature table: NO length prefix. Just raw property bytes.
/// Known-working format: [00][04][05000000] = 6 bytes
pub fn write(w: &mut DocyWriter) {
    // NO begin_length_block here — signature has no length prefix
    w.write_prop_long(sig::VERSION, DOCY_VERSION);
}
