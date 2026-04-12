//! DocumentModel → HTML writer.

use s1_model::attributes::AttributeKey;
use s1_model::{DocumentModel, NodeType};

use crate::HtmlError;

/// Write a [`DocumentModel`] to HTML bytes.
pub fn write_html(doc: &DocumentModel) -> Result<Vec<u8>, HtmlError> {
    let mut html = String::with_capacity(4096);
    html.push_str("<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\"><style>");
    html.push_str("body{font-family:Arial,sans-serif;line-height:1.5;max-width:800px;margin:0 auto;padding:20px}");
    html.push_str("h1{font-size:24pt}h2{font-size:18pt}h3{font-size:14pt}h4{font-size:12pt}");
    html.push_str("table{border-collapse:collapse;margin:12px 0;width:100%}td,th{border:1px solid #ccc;padding:6px 10px}");
    html.push_str("</style></head><body>\n");

    let body_id = match doc.body_id() {
        Some(id) => id,
        None => {
            html.push_str("</body></html>");
            return Ok(html.into_bytes());
        }
    };

    if let Some(body) = doc.node(body_id) {
        for &child_id in &body.children {
            write_node(doc, child_id, &mut html);
        }
    }

    html.push_str("</body></html>\n");
    Ok(html.into_bytes())
}

fn write_node(doc: &DocumentModel, node_id: s1_model::NodeId, html: &mut String) {
    let node = match doc.node(node_id) {
        Some(n) => n,
        None => return,
    };

    match node.node_type {
        NodeType::Paragraph => {
            let style_id = node
                .attributes
                .get_string(&AttributeKey::StyleId)
                .unwrap_or("");
            let tag = if style_id.starts_with("Heading") {
                let level = style_id
                    .trim_start_matches("Heading")
                    .parse::<u8>()
                    .unwrap_or(1)
                    .min(6);
                format!("h{}", level)
            } else {
                "p".to_string()
            };
            html.push_str(&format!("<{}>", tag));
            for &child_id in &node.children {
                write_node(doc, child_id, html);
            }
            html.push_str(&format!("</{}>", tag));
            html.push('\n');
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
            let strikethrough = node
                .attributes
                .get_bool(&AttributeKey::Strikethrough)
                .unwrap_or(false);
            let url = node.attributes.get_string(&AttributeKey::HyperlinkUrl);

            if let Some(url) = url {
                html.push_str(&format!("<a href=\"{}\">", escape_html(url)));
            }
            if bold {
                html.push_str("<b>");
            }
            if italic {
                html.push_str("<i>");
            }
            if underline {
                html.push_str("<u>");
            }
            if strikethrough {
                html.push_str("<s>");
            }

            for &child_id in &node.children {
                write_node(doc, child_id, html);
            }

            if strikethrough {
                html.push_str("</s>");
            }
            if underline {
                html.push_str("</u>");
            }
            if italic {
                html.push_str("</i>");
            }
            if bold {
                html.push_str("</b>");
            }
            if url.is_some() {
                html.push_str("</a>");
            }
        }
        NodeType::Text => {
            if let Some(ref tc) = node.text_content {
                html.push_str(&escape_html(tc));
            }
        }
        NodeType::LineBreak => html.push_str("<br>"),
        NodeType::Table => {
            html.push_str("<table>");
            for &child_id in &node.children {
                write_node(doc, child_id, html);
            }
            html.push_str("</table>\n");
        }
        NodeType::TableRow => {
            html.push_str("<tr>");
            for &child_id in &node.children {
                write_node(doc, child_id, html);
            }
            html.push_str("</tr>");
        }
        NodeType::TableCell => {
            html.push_str("<td>");
            for &child_id in &node.children {
                write_node(doc, child_id, html);
            }
            html.push_str("</td>");
        }
        _ => {
            for &child_id in &node.children {
                write_node(doc, child_id, html);
            }
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
