//! SttbfFfn (Font Table) parsing for DOC binary format.
//!
//! The SttbfFfn is stored in the Table stream and maps font indices to font
//! names. Character formatting SPRMs reference fonts by index (e.g.,
//! `sprmCRgFtc0`), and this table resolves those indices to actual font names.
//!
//! The table location is specified by `fcSttbfFfn` / `lcbSttbfFfn` in the FIB
//! (at FIB offsets 0x0112 and 0x0116 respectively).
//!
//! Structure (Word 97+ / nFib >= 0x00C1):
//! - 2 bytes: `cData` — count of entries (or 0xFFFF for extended format)
//! - 2 bytes: `cbExtra` — extra data bytes per entry (usually 0)
//! - Then `cData` entries, each:
//!   - 1 byte: `cbFfnM1` — size of FFN data minus 1
//!   - `cbFfnM1 + 1` bytes of FFN data containing flags, panose, and the
//!     font name as a null-terminated UTF-16LE string
//!
//! Reference: MS-DOC specification, Section 2.9.268 (SttbfFfn), 2.9.76 (Ffn)

use crate::error::ConvertError;

/// A single entry from the DOC font table.
///
/// Contains the font name and whether it is a TrueType font.
#[derive(Debug, Clone, PartialEq)]
pub struct FontEntry {
    /// The font family name (e.g., "Times New Roman", "Arial").
    pub name: String,
    /// Whether this is a TrueType font (`fTrueType` flag, bit 2 of FFN byte 0).
    pub is_truetype: bool,
}

/// Parse the SttbfFfn (font table) from a region of the Table stream.
///
/// The `table_stream` is the full Table stream data, and `offset`/`length`
/// specify the sub-range containing the SttbfFfn (from the FIB's
/// `fcSttbfFfn` and `lcbSttbfFfn` fields).
///
/// Returns a list of [`FontEntry`] values indexed by their position in the
/// table. Font index 0 corresponds to `result[0]`, etc.
///
/// # Errors
///
/// Returns `ConvertError::InvalidDoc` if:
/// - The offset/length range exceeds the table stream
/// - The SttbfFfn header is truncated
/// - An FFN entry extends past the end of the data
pub fn parse_font_table(
    table_stream: &[u8],
    offset: u32,
    length: u32,
) -> Result<Vec<FontEntry>, ConvertError> {
    if length == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let len = length as usize;

    if start.checked_add(len).is_none() || start + len > table_stream.len() {
        return Err(ConvertError::InvalidDoc(format!(
            "SttbfFfn range {}..{} exceeds table stream size {}",
            start,
            start + len,
            table_stream.len()
        )));
    }

    let data = &table_stream[start..start + len];

    if data.len() < 4 {
        return Err(ConvertError::InvalidDoc(format!(
            "SttbfFfn too short: {} bytes (need at least 4 for header)",
            data.len()
        )));
    }

    // Read cData (count of entries)
    let mut pos: usize = 0;
    let raw_count = u16::from_le_bytes([data[pos], data[pos + 1]]);
    pos += 2;

    let count = if raw_count == 0xFFFF {
        // Extended Sttb: next 4 bytes are the actual count
        if data.len() < 8 {
            return Err(ConvertError::InvalidDoc(
                "SttbfFfn extended header truncated".into(),
            ));
        }
        let ext_count =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;
        // Skip cbExtra (2 bytes)
        if pos + 2 > data.len() {
            return Err(ConvertError::InvalidDoc(
                "SttbfFfn extended header cbExtra truncated".into(),
            ));
        }
        pos += 2;
        ext_count as usize
    } else {
        let count = raw_count as usize;
        // Skip cbExtra (2 bytes)
        if pos + 2 > data.len() {
            return Err(ConvertError::InvalidDoc(
                "SttbfFfn header cbExtra truncated".into(),
            ));
        }
        pos += 2;
        count
    };

    let mut fonts = Vec::with_capacity(count);

    for i in 0..count {
        if pos >= data.len() {
            // No more data — stop parsing (best-effort)
            break;
        }

        // Read cbFfnM1 (1 byte): size of FFN data minus 1
        let cb_ffn_m1 = data[pos] as usize;
        pos += 1;

        let ffn_size = cb_ffn_m1 + 1;

        if pos + ffn_size > data.len() {
            return Err(ConvertError::InvalidDoc(format!(
                "FFN entry {} truncated: need {} bytes at offset {}, but only {} remain",
                i,
                ffn_size,
                pos,
                data.len() - pos
            )));
        }

        let ffn_data = &data[pos..pos + ffn_size];
        pos += ffn_size;

        let entry = parse_ffn_entry(ffn_data);
        fonts.push(entry);
    }

    Ok(fonts)
}

/// Parse a single FFN (Font Family Name) entry from its raw data bytes.
///
/// FFN structure (Word 97+ Unicode):
/// - Byte 0: flags (bits 0-1 = prq/pitch, bit 2 = fTrueType, bit 3 = fEmbedded)
/// - Bytes 1-2: wWeight (font weight)
/// - Byte 3: chs (character set)
/// - Byte 4: ixchSzAlt (index to alternate font name, 0 if none)
/// - Bytes 5-14: panose (10 bytes)
/// - Bytes 15-24: fs (10 bytes, font signature)
/// - Byte 25+: xszFfn — font name as null-terminated UTF-16LE string
///
/// For robustness, if the expected offset for the name is out of range, we
/// scan the entry for a plausible UTF-16LE string.
fn parse_ffn_entry(ffn_data: &[u8]) -> FontEntry {
    // Extract fTrueType flag from byte 0, bit 2
    let is_truetype = if ffn_data.is_empty() {
        false
    } else {
        ffn_data[0] & 0x04 != 0
    };

    // The font name in Word 97+ starts at byte offset 40 in the FFN if all
    // preceding fields are present. However, the spec notes the name area
    // starts right after the fixed fields. The fixed fields are:
    //   - 1 byte (flags) + 2 bytes (wWeight) + 1 byte (chs) + 1 byte (ixchSzAlt)
    //     + 10 bytes (panose) + 10 bytes (fs) = 25 bytes
    // So the name starts at byte 25 for the "full" FFN structure.
    //
    // However, older documents may have shorter fixed-field regions. We try
    // offset 25 first, then fall back to scanning.
    const NAME_OFFSET: usize = 25;

    let name = if ffn_data.len() > NAME_OFFSET + 2 {
        extract_utf16le_string(&ffn_data[NAME_OFFSET..])
    } else {
        // Try scanning from the beginning for a UTF-16LE string
        scan_for_utf16le_name(ffn_data)
    };

    let name = name.unwrap_or_default();

    FontEntry { name, is_truetype }
}

/// Extract a null-terminated UTF-16LE string from a byte slice.
///
/// Reads pairs of bytes as little-endian u16 code units until a null
/// terminator (0x0000) is found or the data runs out.
fn extract_utf16le_string(data: &[u8]) -> Option<String> {
    let mut code_units = Vec::new();
    let mut i = 0;

    while i + 1 < data.len() {
        let unit = u16::from_le_bytes([data[i], data[i + 1]]);
        if unit == 0 {
            break;
        }
        code_units.push(unit);
        i += 2;
    }

    if code_units.is_empty() {
        return None;
    }

    String::from_utf16(&code_units).ok()
}

/// Scan FFN data for a plausible UTF-16LE font name.
///
/// Looks for a sequence of UTF-16LE code units in the printable ASCII range
/// that is at least 2 characters long. This is a fallback for entries where
/// the fixed-field layout does not match the expected format.
fn scan_for_utf16le_name(data: &[u8]) -> Option<String> {
    // Skip the first byte (flags) and look for printable UTF-16LE sequences
    let search_start = 1.min(data.len());
    let mut best_name: Option<String> = None;
    let mut best_len = 0;

    let mut i = search_start;
    while i + 1 < data.len() {
        let unit = u16::from_le_bytes([data[i], data[i + 1]]);

        // Check if this looks like the start of a font name
        // (printable characters: space through tilde, plus common extended)
        if is_plausible_font_char(unit) {
            let mut code_units = Vec::new();
            let mut j = i;
            while j + 1 < data.len() {
                let u = u16::from_le_bytes([data[j], data[j + 1]]);
                if u == 0 || !is_plausible_font_char(u) {
                    break;
                }
                code_units.push(u);
                j += 2;
            }

            if code_units.len() >= 2 && code_units.len() > best_len {
                if let Ok(s) = String::from_utf16(&code_units) {
                    best_len = code_units.len();
                    best_name = Some(s);
                }
            }

            i = j + 2;
        } else {
            i += 2;
        }
    }

    best_name
}

/// Check if a UTF-16 code unit is a plausible font name character.
///
/// Font names typically consist of ASCII letters, digits, spaces, hyphens,
/// and occasionally extended Latin characters.
fn is_plausible_font_char(unit: u16) -> bool {
    // ASCII printable range (space through tilde)
    (0x0020..=0x007E).contains(&unit)
    // Common extended Latin (accented characters)
    || (0x00C0..=0x024F).contains(&unit)
    // CJK compatibility (font names can be in CJK)
    || (0x3000..=0x9FFF).contains(&unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal SttbfFfn binary blob from a list of font names.
    ///
    /// Uses the standard format: 2-byte count, 2-byte cbExtra=0, then entries.
    fn build_sttbf_ffn(fonts: &[(&str, bool)]) -> Vec<u8> {
        let mut buf = Vec::new();

        // cData (count)
        let count = fonts.len() as u16;
        buf.extend_from_slice(&count.to_le_bytes());

        // cbExtra = 0
        buf.extend_from_slice(&0u16.to_le_bytes());

        for &(name, is_truetype) in fonts {
            // Build the FFN entry
            let mut ffn = Vec::new();

            // Byte 0: flags (bit 2 = fTrueType)
            let flags: u8 = if is_truetype { 0x04 } else { 0x00 };
            ffn.push(flags);

            // Bytes 1-2: wWeight (400 = normal)
            ffn.extend_from_slice(&400u16.to_le_bytes());

            // Byte 3: chs (charset, 0 = ANSI)
            ffn.push(0x00);

            // Byte 4: ixchSzAlt (0 = no alternate)
            ffn.push(0x00);

            // Bytes 5-14: panose (10 zero bytes)
            ffn.extend_from_slice(&[0u8; 10]);

            // Bytes 15-24: fs (font signature, 10 zero bytes)
            ffn.extend_from_slice(&[0u8; 10]);

            // Byte 25+: font name as null-terminated UTF-16LE
            for ch in name.encode_utf16() {
                ffn.extend_from_slice(&ch.to_le_bytes());
            }
            // Null terminator
            ffn.extend_from_slice(&0u16.to_le_bytes());

            // cbFfnM1 = ffn.len() - 1
            let cb_ffn_m1 = (ffn.len() - 1) as u8;
            buf.push(cb_ffn_m1);
            buf.extend_from_slice(&ffn);
        }

        buf
    }

    #[test]
    fn parse_empty_font_table() {
        // Zero-length data
        let result = parse_font_table(&[], 0, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_single_font_entry() {
        let sttbf = build_sttbf_ffn(&[("Arial", true)]);
        let result = parse_font_table(&sttbf, 0, sttbf.len() as u32).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Arial");
        assert!(result[0].is_truetype);
    }

    #[test]
    fn parse_multiple_fonts() {
        let sttbf = build_sttbf_ffn(&[
            ("Times New Roman", true),
            ("Courier New", true),
            ("Symbol", false),
        ]);
        let result = parse_font_table(&sttbf, 0, sttbf.len() as u32).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "Times New Roman");
        assert!(result[0].is_truetype);
        assert_eq!(result[1].name, "Courier New");
        assert!(result[1].is_truetype);
        assert_eq!(result[2].name, "Symbol");
        assert!(!result[2].is_truetype);
    }

    #[test]
    fn parse_font_table_with_offset() {
        // Put some padding before the actual data
        let sttbf = build_sttbf_ffn(&[("Calibri", true)]);
        let mut padded = vec![0xAA; 64];
        padded.extend_from_slice(&sttbf);

        let result = parse_font_table(&padded, 64, sttbf.len() as u32).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Calibri");
    }

    #[test]
    fn parse_font_table_out_of_range() {
        let data = vec![0u8; 10];
        let result = parse_font_table(&data, 5, 20);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds table stream"), "got: {err}");
    }

    #[test]
    fn parse_font_table_truncated_header() {
        // Only 2 bytes — missing cbExtra
        let data = [0x01, 0x00];
        let result = parse_font_table(&data, 0, 2);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("too short"), "got: {err}");
    }

    #[test]
    fn extract_utf16le_string_basic() {
        // "AB" as UTF-16LE + null terminator
        let data = [0x41, 0x00, 0x42, 0x00, 0x00, 0x00];
        let s = extract_utf16le_string(&data).unwrap();
        assert_eq!(s, "AB");
    }

    #[test]
    fn extract_utf16le_string_empty() {
        // Immediate null terminator
        let data = [0x00, 0x00];
        assert!(extract_utf16le_string(&data).is_none());
    }
}
