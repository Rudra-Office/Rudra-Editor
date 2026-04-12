//! DocumentModel → RTF writer.

use s1_model::attributes::AttributeKey;
use s1_model::{DocumentModel, NodeType};

use crate::RtfError;

/// Write a [`DocumentModel`] to RTF bytes.
pub fn write_rtf(doc: &DocumentModel) -> Result<Vec<u8>, RtfError> {
    let mut rtf = String::with_capacity(4096);

    // RTF header
    rtf.push_str("{\\rtf1\\ansi\\deff0\n");
    rtf.push_str("{\\fonttbl{\\f0 Arial;}}\n");

    let body_id = match doc.body_id() {
        Some(id) => id,
        None => {
            rtf.push('}');
            return Ok(rtf.into_bytes());
        }
    };

    if let Some(body) = doc.node(body_id) {
        for (i, &child_id) in body.children.iter().enumerate() {
            if i > 0 {
                rtf.push_str("\\par\n");
            }
            write_node(doc, child_id, &mut rtf);
        }
    }

    rtf.push_str("\n}");
    Ok(rtf.into_bytes())
}

fn write_node(doc: &DocumentModel, node_id: s1_model::NodeId, rtf: &mut String) {
    let node = match doc.node(node_id) {
        Some(n) => n,
        None => return,
    };

    match node.node_type {
        NodeType::Paragraph => {
            rtf.push_str("\\pard ");
            // Check for heading style
            if let Some(style) = node.attributes.get_string(&AttributeKey::StyleId) {
                if style.starts_with("Heading") {
                    let level = style
                        .trim_start_matches("Heading")
                        .parse::<u8>()
                        .unwrap_or(1);
                    let size = match level {
                        1 => 48, // 24pt * 2
                        2 => 36, // 18pt * 2
                        3 => 28, // 14pt * 2
                        _ => 24,
                    };
                    rtf.push_str(&format!("\\b\\fs{} ", size));
                }
            }
            for &child_id in &node.children {
                write_node(doc, child_id, rtf);
            }
        }
        NodeType::Run => {
            let bold = node
                .attributes
                .get_bool(&AttributeKey::Bold)
                .unwrap_or(false);
            let italic = node
                .attributes
                .get_bool(&AttributeKey::Italic)
                .unwrap_or(false);
            let underline = node.attributes.get(&AttributeKey::Underline).is_some();
            let font_size = node.attributes.get_f64(&AttributeKey::FontSize);

            let mut prefix = String::new();
            let mut suffix = String::new();

            if bold {
                prefix.push_str("\\b ");
                suffix.push_str("\\b0 ");
            }
            if italic {
                prefix.push_str("\\i ");
                suffix.push_str("\\i0 ");
            }
            if underline {
                prefix.push_str("\\ul ");
                suffix.push_str("\\ulnone ");
            }
            if let Some(sz) = font_size {
                prefix.push_str(&format!("\\fs{} ", (sz * 2.0) as i64));
            }

            if !prefix.is_empty() {
                rtf.push('{');
                rtf.push_str(&prefix);
            }

            for &child_id in &node.children {
                write_node(doc, child_id, rtf);
            }

            if !suffix.is_empty() {
                rtf.push_str(&suffix);
                rtf.push('}');
            }
        }
        NodeType::Text => {
            if let Some(ref tc) = node.text_content {
                for ch in tc.chars() {
                    match ch {
                        '\\' => rtf.push_str("\\\\"),
                        '{' => rtf.push_str("\\{"),
                        '}' => rtf.push_str("\\}"),
                        c if c as u32 > 127 => rtf.push_str(&format!("\\u{}?", c as i32)),
                        c => rtf.push(c),
                    }
                }
            }
        }
        NodeType::LineBreak => rtf.push_str("\\line "),
        _ => {
            for &child_id in &node.children {
                write_node(doc, child_id, rtf);
            }
        }
    }
}
