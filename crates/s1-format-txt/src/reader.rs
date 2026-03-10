//! Plain text reader with encoding detection.
//!
//! Supports UTF-8 (with or without BOM), UTF-16 LE/BE (via BOM), and
//! falls back to Latin-1 (ISO 8859-1) if UTF-8 decoding fails.
//!
//! Line endings: `\r\n`, `\r`, and `\n` are all handled.
//! Each line becomes a Paragraph → Run → Text node.
//! Empty lines become empty paragraphs (no Run/Text children).

use encoding_rs::{UTF_16BE, UTF_16LE, UTF_8};
use s1_model::{DocumentModel, Node, NodeType};

use crate::error::TxtError;

/// The encoding that was detected during reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedEncoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    Latin1,
}

/// Result of reading a text file.
pub struct ReadResult {
    /// The parsed document model.
    pub document: DocumentModel,
    /// The encoding that was detected.
    pub encoding: DetectedEncoding,
}

/// Read plain text bytes into a [`DocumentModel`].
///
/// Performs encoding detection in this order:
/// 1. UTF-8 BOM (`EF BB BF`)
/// 2. UTF-16 LE BOM (`FF FE`)
/// 3. UTF-16 BE BOM (`FE FF`)
/// 4. Valid UTF-8 (no BOM)
/// 5. Latin-1 fallback (ISO 8859-1 — never fails, every byte is valid)
pub fn read(input: &[u8]) -> Result<ReadResult, TxtError> {
    let (text, encoding) = decode(input)?;
    let doc = text_to_document(&text);
    Ok(ReadResult {
        document: doc,
        encoding,
    })
}

/// Detect encoding and decode bytes to a string.
fn decode(input: &[u8]) -> Result<(String, DetectedEncoding), TxtError> {
    // Empty input
    if input.is_empty() {
        return Ok((String::new(), DetectedEncoding::Utf8));
    }

    // Check for BOM
    if input.len() >= 3 && input[0] == 0xEF && input[1] == 0xBB && input[2] == 0xBF {
        // UTF-8 BOM
        let (text, _, had_errors) = UTF_8.decode(&input[3..]);
        if had_errors {
            return Err(TxtError::DecodingError {
                encoding: "UTF-8 (BOM)".into(),
                message: "Invalid UTF-8 sequence after BOM".into(),
            });
        }
        return Ok((text.into_owned(), DetectedEncoding::Utf8Bom));
    }

    if input.len() >= 2 && input[0] == 0xFF && input[1] == 0xFE {
        // UTF-16 LE BOM
        let (text, _, had_errors) = UTF_16LE.decode(input);
        if had_errors {
            return Err(TxtError::DecodingError {
                encoding: "UTF-16 LE".into(),
                message: "Invalid UTF-16 LE sequence".into(),
            });
        }
        return Ok((text.into_owned(), DetectedEncoding::Utf16Le));
    }

    if input.len() >= 2 && input[0] == 0xFE && input[1] == 0xFF {
        // UTF-16 BE BOM
        let (text, _, had_errors) = UTF_16BE.decode(input);
        if had_errors {
            return Err(TxtError::DecodingError {
                encoding: "UTF-16 BE".into(),
                message: "Invalid UTF-16 BE sequence".into(),
            });
        }
        return Ok((text.into_owned(), DetectedEncoding::Utf16Be));
    }

    // No BOM — try UTF-8
    match std::str::from_utf8(input) {
        Ok(text) => Ok((text.to_string(), DetectedEncoding::Utf8)),
        Err(_) => {
            // Fall back to Latin-1 (ISO 8859-1) — every byte maps to a valid Unicode code point
            let text: String = input.iter().map(|&b| b as char).collect();
            Ok((text, DetectedEncoding::Latin1))
        }
    }
}

/// Convert decoded text into a document model.
fn text_to_document(text: &str) -> DocumentModel {
    let mut doc = DocumentModel::new();
    let body_id = doc.body_id().unwrap();

    // Normalize line endings: \r\n → \n, then \r → \n
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");

    let lines: Vec<&str> = normalized.split('\n').collect();

    for (i, line) in lines.iter().enumerate() {
        let para_id = doc.next_id();
        doc.insert_node(body_id, i, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        // Only add Run + Text for non-empty lines
        if !line.is_empty() {
            let run_id = doc.next_id();
            doc.insert_node(para_id, 0, Node::new(run_id, NodeType::Run))
                .unwrap();

            let text_id = doc.next_id();
            doc.insert_node(run_id, 0, Node::text(text_id, *line))
                .unwrap();
        }
    }

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_empty() {
        let result = read(b"").unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Utf8);
        assert_eq!(result.document.to_plain_text(), "");
    }

    #[test]
    fn read_single_line() {
        let result = read(b"Hello World").unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Utf8);
        assert_eq!(result.document.to_plain_text(), "Hello World");
    }

    #[test]
    fn read_multiple_lines() {
        let result = read(b"Line 1\nLine 2\nLine 3").unwrap();
        assert_eq!(result.document.to_plain_text(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn read_blank_lines() {
        let result = read(b"Line 1\n\nLine 3").unwrap();
        let text = crate::write_string(&result.document);
        assert_eq!(text, "Line 1\n\nLine 3");
    }

    #[test]
    fn read_crlf() {
        let result = read(b"Line 1\r\nLine 2\r\nLine 3").unwrap();
        assert_eq!(result.document.to_plain_text(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn read_cr_only() {
        let result = read(b"Line 1\rLine 2").unwrap();
        assert_eq!(result.document.to_plain_text(), "Line 1\nLine 2");
    }

    #[test]
    fn read_utf8_bom() {
        let mut input = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        input.extend_from_slice(b"Hello BOM");
        let result = read(&input).unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Utf8Bom);
        assert_eq!(result.document.to_plain_text(), "Hello BOM");
    }

    #[test]
    fn read_utf16_le_bom() {
        // "Hi" in UTF-16 LE with BOM
        let input: Vec<u8> = vec![
            0xFF, 0xFE, // BOM
            b'H', 0x00, b'i', 0x00,
        ];
        let result = read(&input).unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Utf16Le);
        assert_eq!(result.document.to_plain_text(), "Hi");
    }

    #[test]
    fn read_utf16_be_bom() {
        // "Hi" in UTF-16 BE with BOM
        let input: Vec<u8> = vec![
            0xFE, 0xFF, // BOM
            0x00, b'H', 0x00, b'i',
        ];
        let result = read(&input).unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Utf16Be);
        assert_eq!(result.document.to_plain_text(), "Hi");
    }

    #[test]
    fn read_latin1_fallback() {
        // 0xE9 = 'é' in Latin-1, but invalid as standalone UTF-8
        let input: Vec<u8> = vec![b'c', b'a', b'f', 0xE9];
        let result = read(&input).unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Latin1);
        assert_eq!(result.document.to_plain_text(), "café");
    }

    #[test]
    fn read_utf8_multibyte() {
        let input = "こんにちは".as_bytes(); // Japanese
        let result = read(input).unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Utf8);
        assert_eq!(result.document.to_plain_text(), "こんにちは");
    }

    #[test]
    fn read_preserves_structure() {
        let result = read(b"Hello\nWorld").unwrap();
        let doc = &result.document;
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();

        // Two paragraphs under body
        assert_eq!(body.children.len(), 2);

        // First paragraph has Run > Text "Hello"
        let para1 = doc.node(body.children[0]).unwrap();
        assert_eq!(para1.node_type, NodeType::Paragraph);
        assert_eq!(para1.children.len(), 1);

        let run1 = doc.node(para1.children[0]).unwrap();
        assert_eq!(run1.node_type, NodeType::Run);

        let text1 = doc.node(run1.children[0]).unwrap();
        assert_eq!(text1.text_content.as_deref(), Some("Hello"));
    }

    #[test]
    fn read_empty_lines_are_empty_paragraphs() {
        let result = read(b"A\n\nB").unwrap();
        let doc = &result.document;
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();

        assert_eq!(body.children.len(), 3);

        // Middle paragraph should have no children (empty)
        let empty_para = doc.node(body.children[1]).unwrap();
        assert_eq!(empty_para.node_type, NodeType::Paragraph);
        assert!(empty_para.children.is_empty());
    }

    #[test]
    fn read_trailing_newline() {
        let result = read(b"Hello\n").unwrap();
        let doc = &result.document;
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();

        // "Hello\n" splits into ["Hello", ""] → 2 paragraphs
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn roundtrip_simple() {
        let input = "Hello World\nSecond line\n\nFourth line";
        let result = read(input.as_bytes()).unwrap();
        let output = crate::write_string(&result.document);
        assert_eq!(output, input);
    }

    #[test]
    fn roundtrip_unicode() {
        let input = "こんにちは\ncafé\nüñíçödé";
        let result = read(input.as_bytes()).unwrap();
        let output = crate::write_string(&result.document);
        assert_eq!(output, input);
    }

    #[test]
    fn roundtrip_empty() {
        let input = "";
        let result = read(input.as_bytes()).unwrap();
        let output = crate::write_string(&result.document);
        assert_eq!(output, input);
    }
}
