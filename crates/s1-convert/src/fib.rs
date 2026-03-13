//! FileInformationBlock (FIB) parsing for DOC binary format.
//!
//! The FIB is the first structure in the WordDocument stream of a DOC file.
//! It contains version information, feature flags, and offsets to other
//! structures needed to read the document.
//!
//! This implements parsing for Word 97-2003 format (nFib >= 0x00C1).
//!
//! Reference: MS-DOC specification, Section 2.5.1 - Fib

use crate::error::ConvertError;

/// Parsed FileInformationBlock from a DOC WordDocument stream.
///
/// Contains the essential fields needed to locate and read the piece table
/// (Clx structure), character formatting bin table (PlcfBteChpx), style sheet
/// (STSH), and font table (SttbfFfn) from the Table stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fib {
    /// Magic number — must be 0xA5EC for a valid DOC file.
    pub w_ident: u16,
    /// Version number (e.g., 0x00C1 for Word 97).
    pub n_fib: u16,
    /// Which table stream to use: 0 = "0Table", 1 = "1Table".
    pub which_table: u8,
    /// Offset of the Clx structure in the Table stream.
    pub fc_clx: u32,
    /// Length of the Clx structure in bytes.
    pub lcb_clx: u32,
    /// Count of characters in the main document text body.
    pub ccp_text: u32,
    /// Offset of PlcfBteChpx (character formatting bin table) in the Table stream.
    pub fc_plcf_bte_chpx: u32,
    /// Length of PlcfBteChpx in bytes.
    pub lcb_plcf_bte_chpx: u32,
    /// Offset of PlcfBtePapx (paragraph formatting bin table) in the Table stream.
    pub fc_plcf_bte_papx: u32,
    /// Length of PlcfBtePapx in bytes.
    pub lcb_plcf_bte_papx: u32,
    /// Offset of the style sheet (STSH) in the Table stream.
    pub fc_stshf: u32,
    /// Length of the style sheet (STSH) in bytes.
    pub lcb_stshf: u32,
    /// Offset of the font table (SttbfFfn) in the Table stream.
    pub fc_sttbf_ffn: u32,
    /// Length of the font table (SttbfFfn) in bytes.
    pub lcb_sttbf_ffn: u32,
}

/// DOC file magic number (wIdent).
const DOC_MAGIC: u16 = 0xA5EC;

/// Minimum FIB size we need to parse (through fibRgFcLcb with fcClx/lcbClx).
/// fcClx is at 0x01A2 and lcbClx at 0x01A6, both 4 bytes, so we need at least 0x01AA.
const MIN_FIB_SIZE: usize = 0x01AA;

/// Byte offset of ccpText in the FIB (fibRgLw area).
const OFFSET_CCP_TEXT: usize = 0x004C;

/// Byte offset of fcPlcfBteChpx in the FIB (fibRgFcLcb97 area).
const OFFSET_FC_PLCF_BTE_CHPX: usize = 0x0162;

/// Byte offset of lcbPlcfBteChpx in the FIB (fibRgFcLcb97 area).
const OFFSET_LCB_PLCF_BTE_CHPX: usize = 0x0166;

/// Byte offset of fcPlcfBtePapx in the FIB (fibRgFcLcb97 area).
const OFFSET_FC_PLCF_BTE_PAPX: usize = 0x0102;

/// Byte offset of lcbPlcfBtePapx in the FIB (fibRgFcLcb97 area).
const OFFSET_LCB_PLCF_BTE_PAPX: usize = 0x0106;

/// Byte offset of fcStshf in the FIB (fibRgFcLcb97 area).
const OFFSET_FC_STSHF: usize = 0x00A2;

/// Byte offset of lcbStshf in the FIB (fibRgFcLcb97 area).
const OFFSET_LCB_STSHF: usize = 0x00A6;

/// Byte offset of fcSttbfFfn in the FIB (fibRgFcLcb97 area).
const OFFSET_FC_STTBF_FFN: usize = 0x0112;

/// Byte offset of lcbSttbfFfn in the FIB (fibRgFcLcb97 area).
const OFFSET_LCB_STTBF_FFN: usize = 0x0116;

/// Byte offset of fcClx in the FIB (fibRgFcLcb area, Word 97+).
const OFFSET_FC_CLX: usize = 0x01A2;

/// Byte offset of lcbClx in the FIB (fibRgFcLcb area, Word 97+).
const OFFSET_LCB_CLX: usize = 0x01A6;

impl Fib {
    /// Parse a FileInformationBlock from the WordDocument stream data.
    ///
    /// # Errors
    ///
    /// Returns `ConvertError::InvalidDoc` if:
    /// - The data is too short to contain a valid FIB
    /// - The magic number (wIdent) is not 0xA5EC
    /// - The version (nFib) is unsupported (< 0x00C1)
    pub fn parse(data: &[u8]) -> Result<Self, ConvertError> {
        if data.len() < MIN_FIB_SIZE {
            return Err(ConvertError::InvalidDoc(format!(
                "WordDocument stream too short for FIB: {} bytes (need at least {})",
                data.len(),
                MIN_FIB_SIZE
            )));
        }

        let w_ident = u16::from_le_bytes([data[0], data[1]]);
        if w_ident != DOC_MAGIC {
            return Err(ConvertError::InvalidDoc(format!(
                "invalid FIB magic: expected 0x{DOC_MAGIC:04X}, got 0x{w_ident:04X}"
            )));
        }

        let n_fib = u16::from_le_bytes([data[2], data[3]]);
        if n_fib < 0x00C1 {
            return Err(ConvertError::InvalidDoc(format!(
                "unsupported DOC version: nFib=0x{n_fib:04X} (need >= 0x00C1 for Word 97+)"
            )));
        }

        // Byte 11 (0x0B), bit 1 (value 0x02) = fWhichTblStm
        let which_table = if data[0x0B] & 0x02 != 0 { 1 } else { 0 };

        let ccp_text = u32::from_le_bytes([
            data[OFFSET_CCP_TEXT],
            data[OFFSET_CCP_TEXT + 1],
            data[OFFSET_CCP_TEXT + 2],
            data[OFFSET_CCP_TEXT + 3],
        ]);

        let fc_clx = u32::from_le_bytes([
            data[OFFSET_FC_CLX],
            data[OFFSET_FC_CLX + 1],
            data[OFFSET_FC_CLX + 2],
            data[OFFSET_FC_CLX + 3],
        ]);

        let lcb_clx = u32::from_le_bytes([
            data[OFFSET_LCB_CLX],
            data[OFFSET_LCB_CLX + 1],
            data[OFFSET_LCB_CLX + 2],
            data[OFFSET_LCB_CLX + 3],
        ]);

        let fc_plcf_bte_chpx = u32::from_le_bytes([
            data[OFFSET_FC_PLCF_BTE_CHPX],
            data[OFFSET_FC_PLCF_BTE_CHPX + 1],
            data[OFFSET_FC_PLCF_BTE_CHPX + 2],
            data[OFFSET_FC_PLCF_BTE_CHPX + 3],
        ]);

        let lcb_plcf_bte_chpx = u32::from_le_bytes([
            data[OFFSET_LCB_PLCF_BTE_CHPX],
            data[OFFSET_LCB_PLCF_BTE_CHPX + 1],
            data[OFFSET_LCB_PLCF_BTE_CHPX + 2],
            data[OFFSET_LCB_PLCF_BTE_CHPX + 3],
        ]);

        let fc_plcf_bte_papx = u32::from_le_bytes([
            data[OFFSET_FC_PLCF_BTE_PAPX],
            data[OFFSET_FC_PLCF_BTE_PAPX + 1],
            data[OFFSET_FC_PLCF_BTE_PAPX + 2],
            data[OFFSET_FC_PLCF_BTE_PAPX + 3],
        ]);

        let lcb_plcf_bte_papx = u32::from_le_bytes([
            data[OFFSET_LCB_PLCF_BTE_PAPX],
            data[OFFSET_LCB_PLCF_BTE_PAPX + 1],
            data[OFFSET_LCB_PLCF_BTE_PAPX + 2],
            data[OFFSET_LCB_PLCF_BTE_PAPX + 3],
        ]);

        let fc_stshf = u32::from_le_bytes([
            data[OFFSET_FC_STSHF],
            data[OFFSET_FC_STSHF + 1],
            data[OFFSET_FC_STSHF + 2],
            data[OFFSET_FC_STSHF + 3],
        ]);

        let lcb_stshf = u32::from_le_bytes([
            data[OFFSET_LCB_STSHF],
            data[OFFSET_LCB_STSHF + 1],
            data[OFFSET_LCB_STSHF + 2],
            data[OFFSET_LCB_STSHF + 3],
        ]);

        let fc_sttbf_ffn = u32::from_le_bytes([
            data[OFFSET_FC_STTBF_FFN],
            data[OFFSET_FC_STTBF_FFN + 1],
            data[OFFSET_FC_STTBF_FFN + 2],
            data[OFFSET_FC_STTBF_FFN + 3],
        ]);

        let lcb_sttbf_ffn = u32::from_le_bytes([
            data[OFFSET_LCB_STTBF_FFN],
            data[OFFSET_LCB_STTBF_FFN + 1],
            data[OFFSET_LCB_STTBF_FFN + 2],
            data[OFFSET_LCB_STTBF_FFN + 3],
        ]);

        Ok(Fib {
            w_ident,
            n_fib,
            which_table,
            fc_clx,
            lcb_clx,
            ccp_text,
            fc_plcf_bte_chpx,
            lcb_plcf_bte_chpx,
            fc_plcf_bte_papx,
            lcb_plcf_bte_papx,
            fc_stshf,
            lcb_stshf,
            fc_sttbf_ffn,
            lcb_sttbf_ffn,
        })
    }

    /// Returns the name of the table stream to use.
    ///
    /// Based on the `fWhichTblStm` flag in the FIB, this is either
    /// "1Table" or "0Table".
    pub fn table_stream_name(&self) -> &str {
        if self.which_table == 1 {
            "1Table"
        } else {
            "0Table"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal valid FIB buffer with the required fields set.
    fn make_fib_buffer(
        w_ident: u16,
        n_fib: u16,
        which_table: bool,
        fc_clx: u32,
        lcb_clx: u32,
        ccp_text: u32,
    ) -> Vec<u8> {
        make_fib_buffer_full(w_ident, n_fib, which_table, fc_clx, lcb_clx, ccp_text, 0, 0)
    }

    /// Create a FIB buffer with all fields including CHPx bin table offsets.
    fn make_fib_buffer_full(
        w_ident: u16,
        n_fib: u16,
        which_table: bool,
        fc_clx: u32,
        lcb_clx: u32,
        ccp_text: u32,
        fc_plcf_bte_chpx: u32,
        lcb_plcf_bte_chpx: u32,
    ) -> Vec<u8> {
        let mut buf = vec![0u8; MIN_FIB_SIZE];

        // wIdent at bytes 0-1
        buf[0..2].copy_from_slice(&w_ident.to_le_bytes());
        // nFib at bytes 2-3
        buf[2..4].copy_from_slice(&n_fib.to_le_bytes());
        // fWhichTblStm at byte 11, bit 1
        if which_table {
            buf[0x0B] |= 0x02;
        }
        // ccpText at 0x004C
        buf[OFFSET_CCP_TEXT..OFFSET_CCP_TEXT + 4].copy_from_slice(&ccp_text.to_le_bytes());
        // fcPlcfBteChpx at 0x0162
        buf[OFFSET_FC_PLCF_BTE_CHPX..OFFSET_FC_PLCF_BTE_CHPX + 4]
            .copy_from_slice(&fc_plcf_bte_chpx.to_le_bytes());
        // lcbPlcfBteChpx at 0x0166
        buf[OFFSET_LCB_PLCF_BTE_CHPX..OFFSET_LCB_PLCF_BTE_CHPX + 4]
            .copy_from_slice(&lcb_plcf_bte_chpx.to_le_bytes());
        // fcClx at 0x01A2
        buf[OFFSET_FC_CLX..OFFSET_FC_CLX + 4].copy_from_slice(&fc_clx.to_le_bytes());
        // lcbClx at 0x01A6
        buf[OFFSET_LCB_CLX..OFFSET_LCB_CLX + 4].copy_from_slice(&lcb_clx.to_le_bytes());

        buf
    }

    #[test]
    fn fib_parse_magic() {
        let buf = make_fib_buffer(DOC_MAGIC, 0x00C1, true, 100, 50, 42);
        let fib = Fib::parse(&buf).unwrap();
        assert_eq!(fib.w_ident, DOC_MAGIC);
        assert_eq!(fib.n_fib, 0x00C1);
        assert_eq!(fib.ccp_text, 42);
    }

    #[test]
    fn fib_parse_invalid_magic() {
        let buf = make_fib_buffer(0x1234, 0x00C1, true, 100, 50, 0);
        let result = Fib::parse(&buf);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid FIB magic"), "got: {err}");
    }

    #[test]
    fn fib_parse_too_short() {
        let buf = vec![0u8; 32]; // Way too short
        let result = Fib::parse(&buf);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("too short"), "got: {err}");
    }

    #[test]
    fn fib_parse_unsupported_version() {
        let buf = make_fib_buffer(DOC_MAGIC, 0x0060, true, 100, 50, 0);
        let result = Fib::parse(&buf);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unsupported DOC version"), "got: {err}");
    }

    #[test]
    fn fib_table_stream_name_1table() {
        let buf = make_fib_buffer(DOC_MAGIC, 0x00C1, true, 100, 50, 0);
        let fib = Fib::parse(&buf).unwrap();
        assert_eq!(fib.which_table, 1);
        assert_eq!(fib.table_stream_name(), "1Table");
    }

    #[test]
    fn fib_table_stream_name_0table() {
        let buf = make_fib_buffer(DOC_MAGIC, 0x00C1, false, 100, 50, 0);
        let fib = Fib::parse(&buf).unwrap();
        assert_eq!(fib.which_table, 0);
        assert_eq!(fib.table_stream_name(), "0Table");
    }

    #[test]
    fn fib_parse_clx_offsets() {
        let buf = make_fib_buffer(DOC_MAGIC, 0x00C1, true, 0x1234, 0x5678, 999);
        let fib = Fib::parse(&buf).unwrap();
        assert_eq!(fib.fc_clx, 0x1234);
        assert_eq!(fib.lcb_clx, 0x5678);
        assert_eq!(fib.ccp_text, 999);
    }

    #[test]
    fn fib_parse_chpx_offsets() {
        let buf =
            make_fib_buffer_full(DOC_MAGIC, 0x00C1, true, 0x1234, 0x5678, 999, 0xABCD, 0x0100);
        let fib = Fib::parse(&buf).unwrap();
        assert_eq!(fib.fc_plcf_bte_chpx, 0xABCD);
        assert_eq!(fib.lcb_plcf_bte_chpx, 0x0100);
    }
}
