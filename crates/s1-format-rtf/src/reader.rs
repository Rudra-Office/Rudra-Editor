//! RTF → DocumentModel reader.
#![allow(unused_assignments)]

use s1_model::attributes::{AttributeKey, AttributeValue};
use s1_model::{DocumentModel, Node, NodeType};

use crate::RtfError;

/// Read RTF bytes into a [`DocumentModel`].
pub fn read_rtf(input: &[u8]) -> Result<DocumentModel, RtfError> {
    let text = String::from_utf8_lossy(input);
    if !text.starts_with("{\\rtf") {
        return Err(RtfError::InvalidHeader);
    }

    let mut doc = DocumentModel::new();
    let body_id = doc.body_id().ok_or_else(|| RtfError::ParseError {
        offset: 0,
        message: "no body in document model".into(),
    })?;

    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut font_size: Option<f64> = None;
    let mut current_text = String::new();
    let mut para_id: Option<s1_model::NodeId> = None;

    // Start first paragraph
    let first_para_id = doc.next_id();
    let first_para = Node::new(first_para_id, NodeType::Paragraph);
    let _ = doc.insert_node(body_id, 0, first_para);
    para_id = Some(first_para_id);

    let bytes = text.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    let mut skip_group = 0i32; // depth at which we started skipping

    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if skip_group > 0 && depth < skip_group {
                    skip_group = 0;
                }
                i += 1;
            }
            b'\\' if skip_group == 0 => {
                // Flush accumulated text
                if !current_text.is_empty() {
                    flush_text(
                        &mut doc,
                        &mut current_text,
                        para_id,
                        bold,
                        italic,
                        underline,
                        font_size,
                    );
                }

                i += 1;
                // Read control word
                let word_start = i;
                while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let word = &text[word_start..i];

                // Read optional numeric parameter
                let num_start = i;
                if i < bytes.len() && (bytes[i] == b'-' || bytes[i].is_ascii_digit()) {
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                let param: Option<i64> = if i > num_start {
                    text[num_start..i].parse().ok()
                } else {
                    None
                };

                // Skip trailing space after control word
                if i < bytes.len() && bytes[i] == b' ' {
                    i += 1;
                }

                match word {
                    "par" | "line" => {
                        // New paragraph
                        let pid = doc.next_id();
                        let p = Node::new(pid, NodeType::Paragraph);
                        let _ = doc.insert_node(
                            body_id,
                            doc.node(body_id).map(|n| n.children.len()).unwrap_or(0),
                            p,
                        );
                        para_id = Some(pid);
                    }
                    "pard" => {
                        bold = false;
                        italic = false;
                        underline = false;
                        font_size = None;
                    }
                    "b" => bold = param.unwrap_or(1) != 0,
                    "i" => italic = param.unwrap_or(1) != 0,
                    "ul" => underline = true,
                    "ulnone" => underline = false,
                    "fs" => font_size = param.map(|n| n as f64 / 2.0),
                    "fonttbl" | "colortbl" | "stylesheet" | "info" | "header" | "footer"
                    | "headerf" | "footerf" => {
                        // Skip these groups entirely
                        skip_group = depth;
                    }
                    "'" => {
                        // Hex character escape: \'XX
                        if i + 1 < bytes.len() {
                            let hex: String = text[i..i + 2.min(bytes.len() - i)].to_string();
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                current_text.push(byte as char);
                            }
                            i += 2;
                        }
                    }
                    _ => {} // Skip unknown control words
                }
            }
            b'\n' | b'\r' => {
                i += 1; // Skip whitespace
            }
            _ if skip_group > 0 => {
                i += 1; // Skip content in skipped groups
            }
            _ => {
                current_text.push(bytes[i] as char);
                i += 1;
            }
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        flush_text(
            &mut doc,
            &mut current_text,
            para_id,
            bold,
            italic,
            underline,
            font_size,
        );
    }

    Ok(doc)
}

fn flush_text(
    doc: &mut DocumentModel,
    text: &mut String,
    para_id: Option<s1_model::NodeId>,
    bold: bool,
    italic: bool,
    underline: bool,
    font_size: Option<f64>,
) {
    let pid = match para_id {
        Some(id) => id,
        None => return,
    };

    let run_id = doc.next_id();
    let mut run = Node::new(run_id, NodeType::Run);
    if bold {
        run.attributes
            .set(AttributeKey::Bold, AttributeValue::Bool(true));
    }
    if italic {
        run.attributes
            .set(AttributeKey::Italic, AttributeValue::Bool(true));
    }
    if underline {
        run.attributes.set(
            AttributeKey::Underline,
            AttributeValue::UnderlineStyle(s1_model::UnderlineStyle::Single),
        );
    }
    if let Some(sz) = font_size {
        run.attributes
            .set(AttributeKey::FontSize, AttributeValue::Float(sz));
    }

    let text_id = doc.next_id();
    let mut text_node = Node::new(text_id, NodeType::Text);
    text_node.text_content = Some(std::mem::take(text));

    let _ = doc.insert_node(
        pid,
        doc.node(pid).map(|n| n.children.len()).unwrap_or(0),
        run,
    );
    let _ = doc.insert_node(run_id, 0, text_node);
}
