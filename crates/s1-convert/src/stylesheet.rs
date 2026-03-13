//! STSH (Style Sheet) parsing for DOC binary format.
//!
//! The STSH is stored in the Table stream and maps style indices to style
//! names, types, and inheritance relationships. Paragraph formatting SPRMs
//! reference styles by index (e.g., the `istd` prefix in PAPx), and this
//! table resolves those indices to human-readable names and base style
//! chains.
//!
//! The STSH location is specified by `fcStshf` / `lcbStshf` in the FIB
//! (at FIB offsets 0x00A2 and 0x00A6 respectively).
//!
//! Structure:
//! - STSHI header: 2-byte `cbStshi` length prefix, then `cbStshi` bytes
//!   of style sheet information (skipped for now)
//! - Sequence of STD (Style Definition) entries until end of data:
//!   - 2 bytes: `cbStd` — size of this STD entry (0 = empty slot)
//!   - `cbStd` bytes of STD data containing style type, base style index,
//!     name, and other properties
//!
//! Reference: MS-DOC specification, Section 2.9.271 (STSH), 2.9.260 (STD)

use crate::error::ConvertError;

/// The type of a style in the DOC style sheet.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DocStyleType {
    /// Paragraph style (sgc = 1).
    Paragraph,
    /// Character (run) style (sgc = 2).
    Character,
    /// Table style (sgc = 3).
    Table,
    /// List/numbering style (sgc = 4).
    List,
    /// Unknown or unrecognized style type.
    Unknown(u8),
}

/// A single style entry parsed from the DOC style sheet (STSH).
///
/// Contains the style's index, human-readable name, type, and optional
/// base (parent) style index for inheritance.
#[derive(Debug, Clone, PartialEq)]
pub struct DocStyle {
    /// Zero-based index of this style in the style sheet.
    pub index: u16,
    /// The display name of the style (e.g., "Normal", "Heading 1").
    pub name: String,
    /// The type of style (paragraph, character, table, or list).
    pub style_type: DocStyleType,
    /// Index of the base (parent) style, or `None` if this style has no
    /// parent (0x0FFF means "no base style" in the spec).
    pub based_on: Option<u16>,
}

/// Parse the STSH (Style Sheet) from a region of the Table stream.
///
/// The `table_stream` is the full Table stream data, and `offset`/`length`
/// specify the sub-range containing the STSH (from the FIB's `fcStshf` and
/// `lcbStshf` fields).
///
/// Returns a list of [`DocStyle`] values. Empty style slots (with `cbStd == 0`)
/// are skipped and not included in the output.
///
/// # Errors
///
/// Returns `ConvertError::InvalidDoc` if:
/// - The offset/length range exceeds the table stream
/// - The STSHI header is truncated
/// - An STD entry extends past the end of the data
pub fn parse_stylesheet(
    table_stream: &[u8],
    offset: u32,
    length: u32,
) -> Result<Vec<DocStyle>, ConvertError> {
    if length == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let len = length as usize;

    if start.checked_add(len).is_none() || start + len > table_stream.len() {
        return Err(ConvertError::InvalidDoc(format!(
            "STSH range {}..{} exceeds table stream size {}",
            start,
            start + len,
            table_stream.len()
        )));
    }

    let data = &table_stream[start..start + len];

    if data.len() < 2 {
        return Err(ConvertError::InvalidDoc(format!(
            "STSH too short: {} bytes (need at least 2 for STSHI header size)",
            data.len()
        )));
    }

    // Read STSHI header size
    let cb_stshi = u16::from_le_bytes([data[0], data[1]]) as usize;
    let mut pos = 2;

    // Skip the STSHI data
    if pos + cb_stshi > data.len() {
        return Err(ConvertError::InvalidDoc(format!(
            "STSHI header truncated: need {} bytes at offset {}, but only {} remain",
            cb_stshi,
            pos,
            data.len() - pos
        )));
    }
    pos += cb_stshi;

    // Parse STD entries
    let mut styles = Vec::new();
    let mut style_index: u16 = 0;

    while pos + 2 <= data.len() {
        let cb_std = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        if cb_std == 0 {
            // Empty style slot — skip
            style_index += 1;
            continue;
        }

        if pos + cb_std > data.len() {
            return Err(ConvertError::InvalidDoc(format!(
                "STD entry {} truncated: need {} bytes at offset {}, but only {} remain",
                style_index,
                cb_std,
                pos,
                data.len() - pos
            )));
        }

        let std_data = &data[pos..pos + cb_std];
        pos += cb_std;

        if let Some(style) = parse_std_entry(std_data, style_index) {
            styles.push(style);
        }

        style_index += 1;
    }

    Ok(styles)
}

/// Parse a single STD (Style Definition) entry.
///
/// STD layout (first few fields):
/// - Bytes 0-1: packed word containing:
///   - Bits 0-11: `sti` (built-in style identifier)
///   - Bit 12: `fScratch` (temporary style flag)
///   - Bit 13: `fInvalHeight` (invalid height cache)
///   - Bit 14: `fHasUpe` (has UPE data)
///   - Bit 15: `fMassCopy` (mass copy flag)
/// - Bytes 2-3: packed word containing:
///   - Bits 0-3: `stk` (style kind: 1=paragraph, 2=character, 3=table, 4=list)
///   - Bits 4-15: `istdBase` (base style index, 0x0FFF = no base)
/// - Bytes 4-5: packed word containing:
///   - Bits 0-3: `cupx` (count of property exceptions)
///   - Bits 4-15: `istdNext` (next style index)
/// - Bytes 6-7: `bchUpe` (byte count of UPE data before the style name)
/// - Then comes the style name (if enough data remains):
///   - After `bchUpe` bytes of UPE data
///   - Style name as a counted string: 2-byte character count, then UTF-16LE chars
///
/// Returns `None` if the entry is too short to parse.
fn parse_std_entry(std_data: &[u8], style_index: u16) -> Option<DocStyle> {
    if std_data.len() < 8 {
        return None;
    }

    // Bytes 0-1: sti and flags
    let word0 = u16::from_le_bytes([std_data[0], std_data[1]]);
    let sti = word0 & 0x0FFF;

    // Bytes 2-3: stk and istdBase
    let word1 = u16::from_le_bytes([std_data[2], std_data[3]]);
    let stk = (word1 & 0x000F) as u8;
    let istd_base = (word1 >> 4) & 0x0FFF;

    let style_type = match stk {
        1 => DocStyleType::Paragraph,
        2 => DocStyleType::Character,
        3 => DocStyleType::Table,
        4 => DocStyleType::List,
        _ => DocStyleType::Unknown(stk),
    };

    let based_on = if istd_base == 0x0FFF {
        None
    } else {
        Some(istd_base)
    };

    // Try to extract the style name
    // Bytes 6-7: bchUpe (byte count of property exceptions data)
    let bch_upe = u16::from_le_bytes([std_data[6], std_data[7]]) as usize;

    // The style name follows after the 8-byte fixed header + bchUpe bytes of
    // UPE (property exceptions) data.
    let name_region_start = 8 + bch_upe;

    let name = if name_region_start + 2 <= std_data.len() {
        // Read the name as a counted string:
        // 2 bytes: character count (in UTF-16 code units)
        let char_count =
            u16::from_le_bytes([std_data[name_region_start], std_data[name_region_start + 1]])
                as usize;

        let name_bytes_start = name_region_start + 2;
        let name_bytes_needed = char_count * 2;

        if char_count > 0 && name_bytes_start + name_bytes_needed <= std_data.len() {
            let name_data = &std_data[name_bytes_start..name_bytes_start + name_bytes_needed];
            decode_utf16le_chars(name_data, char_count)
        } else {
            // Fall back to built-in name from sti
            builtin_style_name(sti)
        }
    } else {
        // Not enough data for name — use built-in name from sti
        builtin_style_name(sti)
    };

    Some(DocStyle {
        index: style_index,
        name,
        style_type,
        based_on,
    })
}

/// Decode a fixed number of UTF-16LE code units from a byte slice.
fn decode_utf16le_chars(data: &[u8], count: usize) -> String {
    let mut code_units = Vec::with_capacity(count);
    for i in 0..count {
        let offset = i * 2;
        if offset + 1 < data.len() {
            code_units.push(u16::from_le_bytes([data[offset], data[offset + 1]]));
        }
    }
    String::from_utf16_lossy(&code_units)
}

/// Map a built-in style identifier (sti) to its canonical English name.
///
/// These are the well-known built-in styles defined by the MS-DOC spec.
/// Only the most common ones are mapped; unknown sti values return a
/// placeholder name.
fn builtin_style_name(sti: u16) -> String {
    match sti {
        0 => "Normal".into(),
        1 => "Heading 1".into(),
        2 => "Heading 2".into(),
        3 => "Heading 3".into(),
        4 => "Heading 4".into(),
        5 => "Heading 5".into(),
        6 => "Heading 6".into(),
        7 => "Heading 7".into(),
        8 => "Heading 8".into(),
        9 => "Heading 9".into(),
        10 => "Index 1".into(),
        19 => "TOC 1".into(),
        20 => "TOC 2".into(),
        21 => "TOC 3".into(),
        65 => "Default Paragraph Font".into(),
        99 => "List Paragraph".into(),
        _ => format!("Style {sti}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal STSH binary blob from a list of style definitions.
    ///
    /// Each tuple is `(sti, stk, istd_base, name)`.
    fn build_stsh(styles: &[(u16, u8, Option<u16>, &str)]) -> Vec<u8> {
        let mut buf = Vec::new();

        // STSHI header: cbStshi = 2, then 2 bytes of dummy STSHI data
        let cb_stshi: u16 = 2;
        buf.extend_from_slice(&cb_stshi.to_le_bytes());
        buf.extend_from_slice(&[0x00, 0x00]); // dummy STSHI data

        for &(sti, stk, istd_base, name) in styles {
            // Build STD entry
            let mut std_data = Vec::new();

            // Bytes 0-1: sti (bits 0-11), rest 0
            let word0 = sti & 0x0FFF;
            std_data.extend_from_slice(&word0.to_le_bytes());

            // Bytes 2-3: stk (bits 0-3), istdBase (bits 4-15)
            let base = istd_base.unwrap_or(0x0FFF);
            let word1 = (stk as u16 & 0x0F) | ((base & 0x0FFF) << 4);
            std_data.extend_from_slice(&word1.to_le_bytes());

            // Bytes 4-5: cupx=0, istdNext=0
            std_data.extend_from_slice(&0u16.to_le_bytes());

            // Bytes 6-7: bchUpe = 0 (no property exceptions)
            std_data.extend_from_slice(&0u16.to_le_bytes());

            // Style name: 2-byte char count + UTF-16LE chars
            let utf16_chars: Vec<u16> = name.encode_utf16().collect();
            let char_count = utf16_chars.len() as u16;
            std_data.extend_from_slice(&char_count.to_le_bytes());
            for &unit in &utf16_chars {
                std_data.extend_from_slice(&unit.to_le_bytes());
            }

            // Write cbStd + std_data
            let cb_std = std_data.len() as u16;
            buf.extend_from_slice(&cb_std.to_le_bytes());
            buf.extend_from_slice(&std_data);
        }

        buf
    }

    #[test]
    fn parse_empty_stylesheet() {
        let result = parse_stylesheet(&[], 0, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_normal_style() {
        let stsh = build_stsh(&[(0, 1, None, "Normal")]);
        let result = parse_stylesheet(&stsh, 0, stsh.len() as u32).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 0);
        assert_eq!(result[0].name, "Normal");
        assert_eq!(result[0].style_type, DocStyleType::Paragraph);
        assert_eq!(result[0].based_on, None);
    }

    #[test]
    fn parse_heading_style_with_based_on() {
        let stsh = build_stsh(&[(0, 1, None, "Normal"), (1, 1, Some(0), "Heading 1")]);
        let result = parse_stylesheet(&stsh, 0, stsh.len() as u32).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].index, 0);
        assert_eq!(result[0].name, "Normal");
        assert_eq!(result[0].based_on, None);

        assert_eq!(result[1].index, 1);
        assert_eq!(result[1].name, "Heading 1");
        assert_eq!(result[1].style_type, DocStyleType::Paragraph);
        assert_eq!(result[1].based_on, Some(0));
    }

    #[test]
    fn parse_character_style() {
        let stsh = build_stsh(&[(65, 2, None, "Default Paragraph Font")]);
        let result = parse_stylesheet(&stsh, 0, stsh.len() as u32).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].style_type, DocStyleType::Character);
        assert_eq!(result[0].name, "Default Paragraph Font");
    }

    #[test]
    fn parse_stylesheet_with_empty_slot() {
        // Build a STSH with Normal, then an empty slot (cbStd=0), then Heading 1
        let mut buf = Vec::new();

        // STSHI header
        let cb_stshi: u16 = 2;
        buf.extend_from_slice(&cb_stshi.to_le_bytes());
        buf.extend_from_slice(&[0x00, 0x00]);

        // Style 0: Normal (use build_stsh helper logic inline)
        {
            let mut std_data = Vec::new();
            std_data.extend_from_slice(&0u16.to_le_bytes()); // sti=0
            let word1 = (1u16 & 0x0F) | (0x0FFFu16 << 4);
            std_data.extend_from_slice(&word1.to_le_bytes()); // stk=1, no base
            std_data.extend_from_slice(&0u16.to_le_bytes()); // cupx/istdNext
            std_data.extend_from_slice(&0u16.to_le_bytes()); // bchUpe=0
                                                             // name
            let name = "Normal";
            let utf16: Vec<u16> = name.encode_utf16().collect();
            std_data.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
            for &u in &utf16 {
                std_data.extend_from_slice(&u.to_le_bytes());
            }
            buf.extend_from_slice(&(std_data.len() as u16).to_le_bytes());
            buf.extend_from_slice(&std_data);
        }

        // Style 1: Empty slot
        buf.extend_from_slice(&0u16.to_le_bytes());

        // Style 2: Heading 1
        {
            let mut std_data = Vec::new();
            let sti: u16 = 1;
            std_data.extend_from_slice(&sti.to_le_bytes());
            let word1 = (1u16 & 0x0F) | (0u16 << 4); // basedOn = 0
            std_data.extend_from_slice(&word1.to_le_bytes());
            std_data.extend_from_slice(&0u16.to_le_bytes());
            std_data.extend_from_slice(&0u16.to_le_bytes());
            let name = "Heading 1";
            let utf16: Vec<u16> = name.encode_utf16().collect();
            std_data.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
            for &u in &utf16 {
                std_data.extend_from_slice(&u.to_le_bytes());
            }
            buf.extend_from_slice(&(std_data.len() as u16).to_le_bytes());
            buf.extend_from_slice(&std_data);
        }

        let result = parse_stylesheet(&buf, 0, buf.len() as u32).unwrap();
        // Should have 2 styles: Normal (index 0) and Heading 1 (index 2)
        // Empty slot at index 1 is skipped
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].index, 0);
        assert_eq!(result[0].name, "Normal");
        assert_eq!(result[1].index, 2);
        assert_eq!(result[1].name, "Heading 1");
        assert_eq!(result[1].based_on, Some(0));
    }

    #[test]
    fn parse_stylesheet_out_of_range() {
        let data = vec![0u8; 10];
        let result = parse_stylesheet(&data, 5, 20);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds table stream"), "got: {err}");
    }

    #[test]
    fn parse_stylesheet_truncated_header() {
        // Just 1 byte — not enough for cbStshi
        let data = [0x01];
        let result = parse_stylesheet(&data, 0, 1);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("too short"), "got: {err}");
    }

    #[test]
    fn builtin_style_name_coverage() {
        assert_eq!(builtin_style_name(0), "Normal");
        assert_eq!(builtin_style_name(1), "Heading 1");
        assert_eq!(builtin_style_name(9), "Heading 9");
        assert_eq!(builtin_style_name(65), "Default Paragraph Font");
        assert_eq!(builtin_style_name(999), "Style 999");
    }

    #[test]
    fn doc_style_type_variants() {
        // Exercise all DocStyleType variants
        assert_eq!(DocStyleType::Paragraph, DocStyleType::Paragraph);
        assert_eq!(DocStyleType::Character, DocStyleType::Character);
        assert_eq!(DocStyleType::Table, DocStyleType::Table);
        assert_eq!(DocStyleType::List, DocStyleType::List);
        assert_eq!(DocStyleType::Unknown(5), DocStyleType::Unknown(5));
        assert_ne!(DocStyleType::Paragraph, DocStyleType::Character);
    }
}
