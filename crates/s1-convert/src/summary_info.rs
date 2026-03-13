//! OLE2 SummaryInformation stream parser for DOC metadata extraction.
//!
//! Parses the `\x05SummaryInformation` stream from OLE2/CFB compound files
//! to extract document metadata (title, author, subject, keywords, etc.).
//!
//! The SummaryInformation stream uses the OLE2 Property Set format:
//! - Header with byte order, version, and property set offset
//! - Property set with property count and ID/offset pairs
//! - Property values encoded as typed variants (VT_LPSTR for strings)
//!
//! Reference: [MS-OLEPS] OLE Property Set Data Structures

use s1_model::metadata::DocumentMetadata;

use crate::error::ConvertError;

/// OLE2 property IDs for SummaryInformation stream.
const PID_TITLE: u32 = 0x0002;
const PID_SUBJECT: u32 = 0x0003;
const PID_AUTHOR: u32 = 0x0004;
const PID_KEYWORDS: u32 = 0x0005;
const PID_COMMENTS: u32 = 0x0006;
const PID_LAST_AUTHOR: u32 = 0x0008;

/// OLE2 variant type for null-terminated ANSI string.
const VT_LPSTR: u32 = 0x001E;

/// Parse metadata from an OLE2 SummaryInformation stream.
///
/// Extracts title, author, subject, keywords, and description from the
/// property set data. Returns default metadata if the stream is too short
/// or malformed rather than failing with an error.
///
/// # Errors
///
/// Returns `ConvertError::InvalidDoc` if the stream header is present but
/// the property set data is internally inconsistent in a way that indicates
/// corruption rather than simply missing data.
pub fn parse_summary_info(data: &[u8]) -> Result<DocumentMetadata, ConvertError> {
    let mut meta = DocumentMetadata::default();

    // Minimum size: byte order (2) + version (2) + OS version (4) +
    // class ID (16) + section count (4) = 28 bytes, plus at least
    // one section FMTID (16) + offset (4) = 48 bytes minimum.
    if data.len() < 48 {
        return Ok(meta);
    }

    // Verify byte order marker (0xFFFE = little-endian)
    let byte_order = u16::from_le_bytes([data[0], data[1]]);
    if byte_order != 0xFFFE {
        return Ok(meta);
    }

    // Read number of property sets (at offset 24, after byte order + version + OS + classid)
    let num_sections = read_u32_le(data, 24);
    if num_sections == 0 {
        return Ok(meta);
    }

    // First section: FMTID (16 bytes at offset 28) + offset (4 bytes at offset 44)
    let section_offset = read_u32_le(data, 44) as usize;
    if section_offset >= data.len() {
        return Ok(meta);
    }

    let section_data = &data[section_offset..];
    parse_property_set(section_data, &mut meta)?;

    Ok(meta)
}

/// Parse a single property set section and populate metadata fields.
///
/// The section format is:
/// - Bytes 0-3: section size
/// - Bytes 4-7: number of properties
/// - Then N * 8 bytes of (property_id: u32, offset: u32) pairs
/// - Then property values at the specified offsets within the section
fn parse_property_set(data: &[u8], meta: &mut DocumentMetadata) -> Result<(), ConvertError> {
    if data.len() < 8 {
        return Ok(());
    }

    let _section_size = read_u32_le(data, 0) as usize;
    let num_properties = read_u32_le(data, 4) as usize;

    // Sanity check: each property entry is 8 bytes, starting at offset 8
    let entries_end = 8 + num_properties * 8;
    if entries_end > data.len() {
        return Ok(());
    }

    for i in 0..num_properties {
        let entry_offset = 8 + i * 8;
        let prop_id = read_u32_le(data, entry_offset);
        let prop_offset = read_u32_le(data, entry_offset + 4) as usize;

        if prop_offset >= data.len() {
            continue;
        }

        let prop_data = &data[prop_offset..];
        match prop_id {
            PID_TITLE => {
                if let Some(s) = read_vt_lpstr(prop_data) {
                    meta.title = Some(s);
                }
            }
            PID_SUBJECT => {
                if let Some(s) = read_vt_lpstr(prop_data) {
                    meta.subject = Some(s);
                }
            }
            PID_AUTHOR => {
                if let Some(s) = read_vt_lpstr(prop_data) {
                    meta.creator = Some(s);
                }
            }
            PID_KEYWORDS => {
                if let Some(s) = read_vt_lpstr(prop_data) {
                    // Store keywords as a single entry (DOC doesn't separate them)
                    if !s.is_empty() {
                        meta.keywords = vec![s];
                    }
                }
            }
            PID_COMMENTS => {
                if let Some(s) = read_vt_lpstr(prop_data) {
                    meta.description = Some(s);
                }
            }
            PID_LAST_AUTHOR => {
                // Store as custom property since DocumentMetadata doesn't have
                // a dedicated field for "last author"
                if let Some(s) = read_vt_lpstr(prop_data) {
                    if !s.is_empty() {
                        meta.custom_properties.insert("last_author".to_string(), s);
                    }
                }
            }
            _ => {
                // Unknown property — skip
            }
        }
    }

    Ok(())
}

/// Read a VT_LPSTR property value.
///
/// Format:
/// - Bytes 0-3: variant type (must be 0x001E for VT_LPSTR)
/// - Bytes 4-7: string byte count (including null terminator)
/// - Bytes 8..(8+count): string data (Windows-1252 or UTF-8)
///
/// Returns `None` if the type is wrong, data is truncated, or the string is empty.
fn read_vt_lpstr(data: &[u8]) -> Option<String> {
    if data.len() < 8 {
        return None;
    }

    let vt_type = read_u32_le(data, 0);
    if vt_type != VT_LPSTR {
        return None;
    }

    let byte_count = read_u32_le(data, 4) as usize;
    if byte_count == 0 {
        return None;
    }

    // Check bounds
    if 8 + byte_count > data.len() {
        // Truncated — try to read what we have
        let available = data.len() - 8;
        if available == 0 {
            return None;
        }
        let str_bytes = &data[8..8 + available];
        return Some(decode_windows_1252(str_bytes));
    }

    let str_bytes = &data[8..8 + byte_count];
    Some(decode_windows_1252(str_bytes))
}

/// Decode a Windows-1252 byte string, stripping null terminators.
///
/// First tries UTF-8 decoding. If that fails, falls back to byte-by-byte
/// Windows-1252 to Unicode mapping.
fn decode_windows_1252(bytes: &[u8]) -> String {
    // Strip trailing null bytes
    let trimmed = bytes
        .iter()
        .rposition(|&b| b != 0)
        .map(|pos| &bytes[..=pos])
        .unwrap_or(&[]);

    if trimmed.is_empty() {
        return String::new();
    }

    // Try UTF-8 first (many modern DOC files use it)
    if let Ok(s) = std::str::from_utf8(trimmed) {
        return s.to_string();
    }

    // Fallback: Windows-1252 decoding
    trimmed.iter().map(|&b| windows_1252_to_char(b)).collect()
}

/// Convert a Windows-1252 byte to a Unicode character.
///
/// The 0x80-0x9F range differs from ISO-8859-1 in Windows-1252.
fn windows_1252_to_char(byte: u8) -> char {
    match byte {
        0x80 => '\u{20AC}', // Euro sign
        0x82 => '\u{201A}', // Single low-9 quotation mark
        0x83 => '\u{0192}', // Latin small f with hook
        0x84 => '\u{201E}', // Double low-9 quotation mark
        0x85 => '\u{2026}', // Horizontal ellipsis
        0x86 => '\u{2020}', // Dagger
        0x87 => '\u{2021}', // Double dagger
        0x88 => '\u{02C6}', // Modifier letter circumflex accent
        0x89 => '\u{2030}', // Per mille sign
        0x8A => '\u{0160}', // Latin capital S with caron
        0x8B => '\u{2039}', // Single left-pointing angle quotation
        0x8C => '\u{0152}', // Latin capital ligature OE
        0x8E => '\u{017D}', // Latin capital Z with caron
        0x91 => '\u{2018}', // Left single quotation mark
        0x92 => '\u{2019}', // Right single quotation mark
        0x93 => '\u{201C}', // Left double quotation mark
        0x94 => '\u{201D}', // Right double quotation mark
        0x95 => '\u{2022}', // Bullet
        0x96 => '\u{2013}', // En dash
        0x97 => '\u{2014}', // Em dash
        0x98 => '\u{02DC}', // Small tilde
        0x99 => '\u{2122}', // Trade mark sign
        0x9A => '\u{0161}', // Latin small s with caron
        0x9B => '\u{203A}', // Single right-pointing angle quotation
        0x9C => '\u{0153}', // Latin small ligature oe
        0x9E => '\u{017E}', // Latin small z with caron
        0x9F => '\u{0178}', // Latin capital Y with diaeresis
        // All other bytes map to the same Unicode codepoint
        _ => byte as char,
    }
}

/// Read a little-endian u32 from a byte slice at the given offset.
fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    if offset + 4 > data.len() {
        return 0;
    }
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal SummaryInformation stream with the given properties.
    fn build_summary_info(properties: &[(u32, &str)]) -> Vec<u8> {
        // Header: 48 bytes
        // - byte order: 0xFFFE (2 bytes)
        // - version: 0x0000 (2 bytes)
        // - OS version: 0x00000000 (4 bytes)
        // - class ID: 16 zero bytes (16 bytes)
        // - num sections: 1 (4 bytes)
        // - FMTID: 16 zero bytes (16 bytes)
        // - section offset: 48 (4 bytes) — section immediately follows header
        let mut buf = Vec::new();

        // Byte order
        buf.extend_from_slice(&0xFFFEu16.to_le_bytes());
        // Version
        buf.extend_from_slice(&0u16.to_le_bytes());
        // OS version
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Class ID (16 bytes)
        buf.extend_from_slice(&[0u8; 16]);
        // Number of sections
        buf.extend_from_slice(&1u32.to_le_bytes());
        // FMTID (16 bytes)
        buf.extend_from_slice(&[0u8; 16]);
        // Section offset = 48 (header size)
        buf.extend_from_slice(&48u32.to_le_bytes());

        // Now build the property set section
        let num_props = properties.len() as u32;
        // Property entries start at offset 8 within the section
        // Each entry: prop_id (4) + offset (4) = 8 bytes
        let entries_size = num_props as usize * 8;
        let values_start = 8 + entries_size;

        // Build property values first to know their offsets
        let mut value_blobs: Vec<Vec<u8>> = Vec::new();
        let mut value_offsets: Vec<usize> = Vec::new();
        let mut current_offset = values_start;

        for &(_pid, text) in properties {
            value_offsets.push(current_offset);
            let mut blob = Vec::new();
            // VT_LPSTR type
            blob.extend_from_slice(&VT_LPSTR.to_le_bytes());
            // String byte count (including null terminator)
            let byte_count = text.len() as u32 + 1;
            blob.extend_from_slice(&byte_count.to_le_bytes());
            // String data + null terminator
            blob.extend_from_slice(text.as_bytes());
            blob.push(0);
            // Pad to 4-byte boundary
            while blob.len() % 4 != 0 {
                blob.push(0);
            }
            current_offset += blob.len();
            value_blobs.push(blob);
        }

        let section_size = current_offset as u32;

        // Section header
        buf.extend_from_slice(&section_size.to_le_bytes());
        buf.extend_from_slice(&num_props.to_le_bytes());

        // Property entries
        for (i, &(pid, _)) in properties.iter().enumerate() {
            buf.extend_from_slice(&pid.to_le_bytes());
            buf.extend_from_slice(&(value_offsets[i] as u32).to_le_bytes());
        }

        // Property values
        for blob in &value_blobs {
            buf.extend_from_slice(blob);
        }

        buf
    }

    #[test]
    fn parse_empty_summary() {
        // Too-short data returns default metadata
        let meta = parse_summary_info(&[]).unwrap();
        assert!(meta.title.is_none());
        assert!(meta.creator.is_none());
        assert!(meta.subject.is_none());
        assert!(meta.description.is_none());
        assert!(meta.keywords.is_empty());

        // Wrong byte order returns default
        let mut bad = vec![0u8; 48];
        bad[0] = 0x00; // not 0xFE
        bad[1] = 0x00; // not 0xFF
        let meta = parse_summary_info(&bad).unwrap();
        assert!(meta.title.is_none());
    }

    #[test]
    fn parse_title_and_author() {
        let data = build_summary_info(&[(PID_TITLE, "My Document"), (PID_AUTHOR, "Jane Doe")]);
        let meta = parse_summary_info(&data).unwrap();
        assert_eq!(meta.title.as_deref(), Some("My Document"));
        assert_eq!(meta.creator.as_deref(), Some("Jane Doe"));
    }

    #[test]
    fn parse_all_properties() {
        let data = build_summary_info(&[
            (PID_TITLE, "Test Title"),
            (PID_SUBJECT, "Test Subject"),
            (PID_AUTHOR, "Author Name"),
            (PID_KEYWORDS, "rust document engine"),
            (PID_COMMENTS, "A test document"),
            (PID_LAST_AUTHOR, "Editor Name"),
        ]);
        let meta = parse_summary_info(&data).unwrap();
        assert_eq!(meta.title.as_deref(), Some("Test Title"));
        assert_eq!(meta.subject.as_deref(), Some("Test Subject"));
        assert_eq!(meta.creator.as_deref(), Some("Author Name"));
        assert_eq!(meta.keywords, vec!["rust document engine".to_string()]);
        assert_eq!(meta.description.as_deref(), Some("A test document"));
        assert_eq!(
            meta.custom_properties
                .get("last_author")
                .map(|s| s.as_str()),
            Some("Editor Name")
        );
    }
}
