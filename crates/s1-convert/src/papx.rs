//! PAPx (Paragraph Properties) parsing for DOC binary format.
//!
//! Parses paragraph formatting SPRMs from PAPx property arrays in DOC files.
//! The PAPx data is located via the PlcfBtePapx bin table referenced from the
//! FIB, but this module focuses on parsing the SPRM byte sequences into
//! structured paragraph properties.
//!
//! Reference: MS-DOC specification, Sections 2.6.1 (Prl), 2.6.2 (Sprm),
//! 2.9.182 (PAPx)

/// Paragraph properties extracted from PAPx SPRM data.
///
/// Each field is `Option` because a given PAPx may only contain a subset
/// of formatting overrides. Absent fields inherit from the paragraph's style.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParaProperties {
    /// Justification: 0 = left, 1 = center, 2 = right, 3 = both/justify.
    pub justification: Option<u8>,
    /// Left indent in twips (1/1440 inch). Can be negative for hanging indent.
    pub indent_left_twips: Option<i16>,
    /// Right indent in twips.
    pub indent_right_twips: Option<i16>,
    /// First-line indent in twips. Negative values produce a hanging indent.
    pub indent_first_twips: Option<i16>,
    /// Space before paragraph in twips.
    pub space_before_twips: Option<u16>,
    /// Space after paragraph in twips.
    pub space_after_twips: Option<u16>,
    /// Line spacing information.
    pub line_spacing: Option<LineSpacingInfo>,
    /// Keep all lines of this paragraph on one page.
    pub keep_lines: Option<bool>,
    /// Keep this paragraph on the same page as the next paragraph.
    pub keep_with_next: Option<bool>,
    /// Insert a page break before this paragraph.
    pub page_break_before: Option<bool>,
    /// List nesting level (0-8).
    pub list_level: Option<u8>,
    /// List format override index (ilfo). References the list formatting table.
    pub list_id: Option<u16>,
    /// Style index (istd) from the PAPx prefix, indicating the base paragraph style.
    pub style_index: Option<u16>,
}

/// Line spacing information from sprmPDyaLine.
///
/// The SPRM encodes both the spacing value and a flag indicating whether
/// the value is a multiple of single-spacing or an absolute/minimum value.
#[derive(Debug, Clone, PartialEq)]
pub struct LineSpacingInfo {
    /// Spacing value in twips (if not multiple) or 240ths of single-line spacing.
    /// Positive values mean "at least" spacing, negative values mean "exact" spacing
    /// (when `is_multiple` is false).
    pub value: i16,
    /// If true, `value` is a multiple of single-line spacing (240 = single, 480 = double).
    /// If false, `value` is an absolute measurement in twips.
    pub is_multiple: bool,
}

// --- Paragraph SPRM opcodes ---

/// sprmPJc — paragraph justification (1 byte operand).
const SPRM_PJC: u16 = 0x2403;
/// sprmPFKeep — keep lines together (1 byte toggle).
const SPRM_PF_KEEP: u16 = 0x2405;
/// sprmPFKeepFollow — keep with next paragraph (1 byte toggle).
const SPRM_PF_KEEP_FOLLOW: u16 = 0x2406;
/// sprmPFPageBreakBefore — page break before paragraph (1 byte toggle).
const SPRM_PF_PAGE_BREAK_BEFORE: u16 = 0x2407;
/// sprmPIlvl — list level (1 byte operand).
const SPRM_PILVL: u16 = 0x260A;
/// sprmPIlfo — list format override index (2 byte operand).
const SPRM_PILFO: u16 = 0x460B;
/// sprmPDxaRight — right indent in twips (2 byte signed operand).
const SPRM_PDXA_RIGHT: u16 = 0x840E;
/// sprmPDxaLeft — left indent in twips (2 byte signed operand).
const SPRM_PDXA_LEFT: u16 = 0x840F;
/// sprmPDxaLeft1 — first-line indent in twips (2 byte signed operand).
const SPRM_PDXA_LEFT1: u16 = 0x8411;
/// sprmPDyaLine — line spacing: low 16 bits = value, high 16 bits = fMultLinespace (4 byte operand).
const SPRM_PDYA_LINE: u16 = 0x6412;
/// sprmPDyaBefore — space before paragraph in twips (2 byte operand).
const SPRM_PDYA_BEFORE: u16 = 0xA413;
/// sprmPDyaAfter — space after paragraph in twips (2 byte operand).
const SPRM_PDYA_AFTER: u16 = 0xA414;

/// Parse paragraph properties from a sequence of SPRM bytes.
///
/// The `data` slice should contain concatenated SPRM entries (opcode + operand).
/// Unknown SPRMs are silently skipped using the standard operand size logic.
///
/// # Examples
///
/// ```
/// use s1_convert::papx::parse_para_properties_from_sprms;
///
/// // sprmPJc (0x2403) with operand 1 (center)
/// let sprms = [0x03, 0x24, 0x01];
/// let props = parse_para_properties_from_sprms(&sprms);
/// assert_eq!(props.justification, Some(1));
/// ```
pub fn parse_para_properties_from_sprms(data: &[u8]) -> ParaProperties {
    let mut props = ParaProperties::default();
    let mut offset = 0;

    while offset + 2 <= data.len() {
        let opcode = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        let operand_size = sprm_operand_size(opcode, &data[offset..]);
        if offset + operand_size > data.len() {
            // Truncated SPRM — stop parsing
            break;
        }

        let operand = &data[offset..offset + operand_size];
        apply_sprm_to_para(&mut props, opcode, operand);
        offset += operand_size;
    }

    props
}

/// Apply a single paragraph SPRM to a `ParaProperties` struct.
///
/// If the `opcode` is not a recognized paragraph SPRM, this is a no-op.
/// The `operand` slice must contain exactly the operand bytes for this SPRM.
pub fn apply_sprm_to_para(props: &mut ParaProperties, opcode: u16, operand: &[u8]) {
    match opcode {
        SPRM_PJC => {
            if let Some(&val) = operand.first() {
                props.justification = Some(val);
            }
        }
        SPRM_PF_KEEP => {
            if let Some(&val) = operand.first() {
                props.keep_lines = Some(val != 0);
            }
        }
        SPRM_PF_KEEP_FOLLOW => {
            if let Some(&val) = operand.first() {
                props.keep_with_next = Some(val != 0);
            }
        }
        SPRM_PF_PAGE_BREAK_BEFORE => {
            if let Some(&val) = operand.first() {
                props.page_break_before = Some(val != 0);
            }
        }
        SPRM_PILVL => {
            if let Some(&val) = operand.first() {
                props.list_level = Some(val);
            }
        }
        SPRM_PILFO => {
            if operand.len() >= 2 {
                props.list_id = Some(u16::from_le_bytes([operand[0], operand[1]]));
            }
        }
        SPRM_PDXA_RIGHT => {
            if operand.len() >= 2 {
                props.indent_right_twips = Some(i16::from_le_bytes([operand[0], operand[1]]));
            }
        }
        SPRM_PDXA_LEFT => {
            if operand.len() >= 2 {
                props.indent_left_twips = Some(i16::from_le_bytes([operand[0], operand[1]]));
            }
        }
        SPRM_PDXA_LEFT1 => {
            if operand.len() >= 2 {
                props.indent_first_twips = Some(i16::from_le_bytes([operand[0], operand[1]]));
            }
        }
        SPRM_PDYA_LINE => {
            if operand.len() >= 4 {
                let dya_line = i16::from_le_bytes([operand[0], operand[1]]);
                let f_mult = u16::from_le_bytes([operand[2], operand[3]]);
                props.line_spacing = Some(LineSpacingInfo {
                    value: dya_line,
                    is_multiple: f_mult != 0,
                });
            }
        }
        SPRM_PDYA_BEFORE => {
            if operand.len() >= 2 {
                props.space_before_twips = Some(u16::from_le_bytes([operand[0], operand[1]]));
            }
        }
        SPRM_PDYA_AFTER => {
            if operand.len() >= 2 {
                props.space_after_twips = Some(u16::from_le_bytes([operand[0], operand[1]]));
            }
        }
        _ => {
            // Unknown SPRM — silently skip
        }
    }
}

/// Determine the operand size for a given SPRM opcode.
///
/// Uses bits 14-13 of the opcode to determine the operand size type:
/// - 0b00 → 1 byte (toggle/byte value)
/// - 0b01 → 1 byte
/// - 0b10 → 2 bytes (word value)
/// - 0b11 → 4 bytes or variable length
///
/// For type 0b11, bit 9 distinguishes:
/// - bit 9 clear → 4 bytes (fixed)
/// - bit 9 set → variable: first byte of operand is length
///
/// Special handling for sprmPDyaLine (0x6412) which uses the 0b11 prefix
/// and has a 4-byte operand.
fn sprm_operand_size(opcode: u16, remaining: &[u8]) -> usize {
    let spra = (opcode >> 13) & 0x07;
    match spra {
        0 | 1 => 1, // 1-byte operand
        2 => 2,     // 2-byte operand
        3 => 4,     // 4-byte operand
        4 => 2,     // 2-byte operand
        5 => 2,     // 2-byte operand
        6 => {
            // Variable length: first byte is the operand size
            if remaining.is_empty() {
                0
            } else {
                let cb = remaining[0] as usize;
                1 + cb // include the length byte itself
            }
        }
        7 => 3, // 3-byte operand
        _ => 0, // shouldn't happen with 3 bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build SPRM bytes from opcode and operand.
    fn make_sprm(opcode: u16, operand: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&opcode.to_le_bytes());
        buf.extend_from_slice(operand);
        buf
    }

    #[test]
    fn parse_para_justification_left() {
        // sprmPJc = 0x2403, operand = 0 (left)
        let data = make_sprm(SPRM_PJC, &[0x00]);
        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.justification, Some(0));
    }

    #[test]
    fn parse_para_justification_center() {
        // sprmPJc = 0x2403, operand = 1 (center)
        let data = make_sprm(SPRM_PJC, &[0x01]);
        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.justification, Some(1));
    }

    #[test]
    fn parse_para_spacing() {
        // sprmPDyaBefore = 0xA413, operand = 120 twips (6pt)
        // sprmPDyaAfter = 0xA414, operand = 240 twips (12pt)
        let mut data = make_sprm(SPRM_PDYA_BEFORE, &120u16.to_le_bytes());
        data.extend_from_slice(&make_sprm(SPRM_PDYA_AFTER, &240u16.to_le_bytes()));

        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.space_before_twips, Some(120));
        assert_eq!(props.space_after_twips, Some(240));
    }

    #[test]
    fn parse_para_indent() {
        // sprmPDxaLeft = 0x840F, operand = 720 twips (0.5 inch)
        // sprmPDxaRight = 0x840E, operand = 360 twips (0.25 inch)
        // sprmPDxaLeft1 = 0x8411, operand = -360 twips (hanging indent)
        let mut data = make_sprm(SPRM_PDXA_LEFT, &720i16.to_le_bytes());
        data.extend_from_slice(&make_sprm(SPRM_PDXA_RIGHT, &360i16.to_le_bytes()));
        data.extend_from_slice(&make_sprm(SPRM_PDXA_LEFT1, &(-360i16).to_le_bytes()));

        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.indent_left_twips, Some(720));
        assert_eq!(props.indent_right_twips, Some(360));
        assert_eq!(props.indent_first_twips, Some(-360));
    }

    #[test]
    fn parse_para_line_spacing() {
        // sprmPDyaLine = 0x6412, operand = 4 bytes: value (240) + fMult (1)
        // 240 with fMult=1 means single spacing
        let mut operand = Vec::new();
        operand.extend_from_slice(&240i16.to_le_bytes()); // dyaLine
        operand.extend_from_slice(&1u16.to_le_bytes()); // fMultLinespace
        let data = make_sprm(SPRM_PDYA_LINE, &operand);

        let props = parse_para_properties_from_sprms(&data);
        let ls = props.line_spacing.unwrap();
        assert_eq!(ls.value, 240);
        assert!(ls.is_multiple);
    }

    #[test]
    fn parse_para_line_spacing_exact() {
        // Exact spacing: -360 twips, fMult=0
        let mut operand = Vec::new();
        operand.extend_from_slice(&(-360i16).to_le_bytes()); // dyaLine (negative = exact)
        operand.extend_from_slice(&0u16.to_le_bytes()); // fMultLinespace = 0
        let data = make_sprm(SPRM_PDYA_LINE, &operand);

        let props = parse_para_properties_from_sprms(&data);
        let ls = props.line_spacing.unwrap();
        assert_eq!(ls.value, -360);
        assert!(!ls.is_multiple);
    }

    #[test]
    fn parse_para_keep_lines() {
        // sprmPFKeep = 0x2405, operand = 1 (true)
        let data = make_sprm(SPRM_PF_KEEP, &[0x01]);
        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.keep_lines, Some(true));

        // Also test false
        let data_off = make_sprm(SPRM_PF_KEEP, &[0x00]);
        let props_off = parse_para_properties_from_sprms(&data_off);
        assert_eq!(props_off.keep_lines, Some(false));
    }

    #[test]
    fn parse_para_page_break_before() {
        // sprmPFPageBreakBefore = 0x2407, operand = 1 (true)
        let data = make_sprm(SPRM_PF_PAGE_BREAK_BEFORE, &[0x01]);
        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.page_break_before, Some(true));
    }

    #[test]
    fn parse_para_list_info() {
        // sprmPIlvl = 0x260A, operand = 2 (level 2)
        // sprmPIlfo = 0x460B, operand = 5 (list id 5)
        let mut data = make_sprm(SPRM_PILVL, &[0x02]);
        data.extend_from_slice(&make_sprm(SPRM_PILFO, &5u16.to_le_bytes()));

        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.list_level, Some(2));
        assert_eq!(props.list_id, Some(5));
    }

    #[test]
    fn parse_para_multiple_sprms_combined() {
        // Test a realistic combination: center-aligned, space before 120, keep with next
        let mut data = make_sprm(SPRM_PJC, &[0x01]); // center
        data.extend_from_slice(&make_sprm(SPRM_PDYA_BEFORE, &120u16.to_le_bytes()));
        data.extend_from_slice(&make_sprm(SPRM_PF_KEEP_FOLLOW, &[0x01]));

        let props = parse_para_properties_from_sprms(&data);
        assert_eq!(props.justification, Some(1));
        assert_eq!(props.space_before_twips, Some(120));
        assert_eq!(props.keep_with_next, Some(true));
        // Fields not set should remain None
        assert_eq!(props.indent_left_twips, None);
        assert_eq!(props.page_break_before, None);
    }

    #[test]
    fn parse_para_empty_sprms() {
        let props = parse_para_properties_from_sprms(&[]);
        assert_eq!(props, ParaProperties::default());
    }

    #[test]
    fn parse_para_truncated_sprm_is_safe() {
        // Just an opcode with no operand bytes — should not panic
        let data = [0x03, 0x24]; // sprmPJc opcode only, missing operand
        let props = parse_para_properties_from_sprms(&data);
        // Truncated — justification should not be set
        assert_eq!(props.justification, None);
    }

    #[test]
    fn apply_sprm_unknown_opcode() {
        // An unknown SPRM opcode should be silently ignored
        let mut props = ParaProperties::default();
        apply_sprm_to_para(&mut props, 0xFFFF, &[0x01, 0x02]);
        assert_eq!(props, ParaProperties::default());
    }

    #[test]
    fn parse_para_justify_all_values() {
        for (val, expected) in [(0u8, 0u8), (1, 1), (2, 2), (3, 3)] {
            let data = make_sprm(SPRM_PJC, &[val]);
            let props = parse_para_properties_from_sprms(&data);
            assert_eq!(props.justification, Some(expected));
        }
    }
}
