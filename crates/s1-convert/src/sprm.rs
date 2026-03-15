//! SPRM (Single Property Modifier) parsing for DOC binary format.
//!
//! SPRMs are opcode + operand pairs that modify default character or paragraph
//! properties. They are stored inside CHPx and PAPx structures in the DOC
//! binary format.
//!
//! Each SPRM has a 16-bit opcode where:
//! - Bits 0-8: ispmd (property identifier)
//! - Bit 9: fSpec (special handling flag)
//! - Bits 10-12: sgc (operand type group: 1=paragraph, 2=character, etc.)
//! - Bits 13-15: spra (operand size type)
//!
//! Reference: MS-DOC specification, Section 2.6.1 - Sprm

/// A parsed Single Property Modifier.
#[derive(Debug, Clone, PartialEq)]
pub struct Sprm {
    /// The 16-bit SPRM opcode.
    pub opcode: u16,
    /// The operand bytes (variable length depending on opcode).
    pub operand: Vec<u8>,
}

// ─── Character formatting SPRM opcodes ──────────────────────────────────────

/// Bold toggle (1 byte: 0=off, 1=on, 128=toggle).
pub const SPRM_CF_BOLD: u16 = 0x0835;

/// Italic toggle (1 byte: 0=off, 1=on, 128=toggle).
pub const SPRM_CF_ITALIC: u16 = 0x0836;

/// Strikethrough toggle (1 byte: 0=off, 1=on).
pub const SPRM_CF_STRIKE: u16 = 0x0837;

/// All caps toggle (1 byte: 0=off, 1=on).
pub const SPRM_CF_CAPS: u16 = 0x0838;

/// Underline type (1 byte).
pub const SPRM_C_KUL: u16 = 0x2A3E;

/// Color index (1 byte, maps to standard DOC color table).
pub const SPRM_C_ICO: u16 = 0x2A42;

/// Font size in half-points (2 bytes).
pub const SPRM_C_HPS: u16 = 0x4A43;

/// Superscript/subscript (1 byte: 0=normal, 1=super, 2=sub).
pub const SPRM_C_SUPER_SUB: u16 = 0x2A48;

/// Font index for ASCII text (2 bytes, index into font table).
pub const SPRM_C_RG_FTC0: u16 = 0x4A4F;

/// Determine the operand size in bytes for a given SPRM opcode.
///
/// The size is determined by the `spra` field (bits 13-15) of the opcode:
/// - 0, 1: 1 byte (toggle or 1-byte value)
/// - 2: 2 bytes
/// - 3: 4 bytes
/// - 4, 5: 2 bytes
/// - 6: variable length (first byte of operand is the length)
/// - 7: 3 bytes
///
/// For variable-length SPRMs (spra=6), this returns `0` to signal that the
/// caller must read the first byte of the operand to determine the actual size.
pub fn sprm_operand_size(opcode: u16) -> usize {
    match (opcode >> 13) & 0x07 {
        0 | 1 => 1,
        2 => 2,
        3 => 4,
        4 | 5 => 2,
        6 => 0, // variable: first byte is length
        7 => 3,
        _ => 1,
    }
}

/// Parse a sequence of SPRMs from a byte slice.
///
/// Reads consecutive SPRM entries (2-byte opcode + variable operand) until
/// the data is exhausted. Malformed trailing bytes are silently ignored.
pub fn parse_sprms(data: &[u8]) -> Vec<Sprm> {
    let mut result = Vec::new();
    let mut offset = 0;

    while offset + 2 <= data.len() {
        let opcode = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        let size = sprm_operand_size(opcode);

        if size == 0 {
            // Variable length: first byte is the length
            if offset >= data.len() {
                break;
            }
            let var_len = data[offset] as usize;
            offset += 1;
            if offset + var_len > data.len() {
                break;
            }
            let operand = data[offset..offset + var_len].to_vec();
            offset += var_len;
            result.push(Sprm { opcode, operand });
        } else {
            if offset + size > data.len() {
                break;
            }
            let operand = data[offset..offset + size].to_vec();
            offset += size;
            result.push(Sprm { opcode, operand });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_sprm_bold() {
        // sprmCFBold = 0x0835, operand = 1 (bold on)
        let data = [0x35, 0x08, 0x01];
        let sprms = parse_sprms(&data);
        assert_eq!(sprms.len(), 1);
        assert_eq!(sprms[0].opcode, SPRM_CF_BOLD);
        assert_eq!(sprms[0].operand, vec![0x01]);
    }

    #[test]
    fn parse_multiple_sprms() {
        // sprmCFBold (0x0835, 1 byte operand) + sprmCHps (0x4A43, 2 byte operand)
        let mut data = Vec::new();
        // Bold on
        data.extend_from_slice(&0x0835u16.to_le_bytes());
        data.push(0x01);
        // Font size = 24 half-points (12pt)
        data.extend_from_slice(&0x4A43u16.to_le_bytes());
        data.extend_from_slice(&24u16.to_le_bytes());

        let sprms = parse_sprms(&data);
        assert_eq!(sprms.len(), 2);
        assert_eq!(sprms[0].opcode, SPRM_CF_BOLD);
        assert_eq!(sprms[0].operand, vec![0x01]);
        assert_eq!(sprms[1].opcode, SPRM_C_HPS);
        assert_eq!(sprms[1].operand, vec![24, 0]);
    }

    #[test]
    fn sprm_operand_sizes() {
        // spra = 0 (bits 13-15 = 000) => 1 byte
        assert_eq!(sprm_operand_size(SPRM_CF_BOLD), 1); // 0x0835 => spra = 0
        assert_eq!(sprm_operand_size(SPRM_CF_ITALIC), 1); // 0x0836 => spra = 0

        // spra = 1 (bits 13-15 = 001) => 1 byte
        assert_eq!(sprm_operand_size(SPRM_C_KUL), 1); // 0x2A3E => spra = 1
        assert_eq!(sprm_operand_size(SPRM_C_ICO), 1); // 0x2A42 => spra = 1
        assert_eq!(sprm_operand_size(SPRM_C_SUPER_SUB), 1); // 0x2A48 => spra = 1

        // spra = 2 (bits 13-15 = 010) => 2 bytes
        assert_eq!(sprm_operand_size(SPRM_C_HPS), 2); // 0x4A43 => spra = 2
        assert_eq!(sprm_operand_size(SPRM_C_RG_FTC0), 2); // 0x4A4F => spra = 2
    }
}
