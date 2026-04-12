//! HTML → DocumentModel reader.
#![allow(unused_variables)]

use s1_model::attributes::{AttributeKey, AttributeValue};
use s1_model::{DocumentModel, Node, NodeType};

use crate::HtmlError;

/// Read HTML bytes into a [`DocumentModel`].
pub fn read_html(input: &[u8]) -> Result<DocumentModel, HtmlError> {
    let text = String::from_utf8_lossy(input);
    let mut doc = DocumentModel::new();
    let body_id = doc
        .body_id()
        .ok_or_else(|| HtmlError::Parse("no body".into()))?;

    // Simple tag-based parser — walks HTML text and extracts structure
    let mut pos = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut current_para_id: Option<s1_model::NodeId> = None;
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut strikethrough = false;
    let mut link_url: Option<String> = None;

    while pos < chars.len() {
        if chars[pos] == '<' {
            // Parse tag
            let tag_start = pos + 1;
            let mut tag_end = tag_start;
            while tag_end < chars.len() && chars[tag_end] != '>' {
                tag_end += 1;
            }
            let tag_content: String = chars[tag_start..tag_end].iter().collect();
            let tag_lower = tag_content.to_lowercase();
            let tag_name = tag_lower.split_whitespace().next().unwrap_or("");

            match tag_name {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level: u8 = tag_name[1..].parse().unwrap_or(1);
                    let para_id = doc.next_id();
                    let mut para = Node::new(para_id, NodeType::Paragraph);
                    para.attributes.set(
                        AttributeKey::StyleId,
                        AttributeValue::String(format!("Heading{}", level)),
                    );
                    let _ = doc.insert_node(
                        body_id,
                        doc.node(body_id).map(|n| n.children.len()).unwrap_or(0),
                        para,
                    );
                    current_para_id = Some(para_id);
                }
                "p" | "div" => {
                    let para_id = doc.next_id();
                    let para = Node::new(para_id, NodeType::Paragraph);
                    let _ = doc.insert_node(
                        body_id,
                        doc.node(body_id).map(|n| n.children.len()).unwrap_or(0),
                        para,
                    );
                    current_para_id = Some(para_id);
                }
                "b" | "strong" => bold = true,
                "/b" | "/strong" => bold = false,
                "i" | "em" => italic = true,
                "/i" | "/em" => italic = false,
                "u" => underline = true,
                "/u" => underline = false,
                "s" | "del" | "strike" => strikethrough = true,
                "/s" | "/del" | "/strike" => strikethrough = false,
                "br" | "br/" => {
                    if let Some(pid) = current_para_id {
                        let br_id = doc.next_id();
                        let br = Node::new(br_id, NodeType::LineBreak);
                        let _ = doc.insert_node(
                            pid,
                            doc.node(pid).map(|n| n.children.len()).unwrap_or(0),
                            br,
                        );
                    }
                }
                "/h1" | "/h2" | "/h3" | "/h4" | "/h5" | "/h6" | "/p" | "/div" => {
                    current_para_id = None;
                }
                _ if tag_name.starts_with("a ") || tag_name == "a" => {
                    // Extract href
                    if let Some(href_pos) = tag_lower.find("href=\"") {
                        let start = href_pos + 6;
                        let end = tag_lower[start..]
                            .find('"')
                            .map(|p| start + p)
                            .unwrap_or(tag_lower.len());
                        let href: String = chars[tag_start..tag_end].iter().collect();
                        if let Some(hp) = href.to_lowercase().find("href=\"") {
                            let hs = hp + 6;
                            let he = href[hs..].find('"').map(|p| hs + p).unwrap_or(href.len());
                            link_url = Some(href[hs..he].to_string());
                        }
                    }
                }
                "/a" => link_url = None,
                _ => {}
            }
            pos = tag_end + 1;
        } else {
            // Text content — collect until next tag
            let text_start = pos;
            while pos < chars.len() && chars[pos] != '<' {
                pos += 1;
            }
            let text_content: String = chars[text_start..pos].iter().collect();
            let text_content = decode_entities(&text_content);

            if !text_content.trim().is_empty() {
                // Ensure we have a paragraph
                let para_id = if let Some(pid) = current_para_id {
                    pid
                } else {
                    let pid = doc.next_id();
                    let para = Node::new(pid, NodeType::Paragraph);
                    let _ = doc.insert_node(
                        body_id,
                        doc.node(body_id).map(|n| n.children.len()).unwrap_or(0),
                        para,
                    );
                    current_para_id = Some(pid);
                    pid
                };

                // Create run with formatting
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
                if strikethrough {
                    run.attributes
                        .set(AttributeKey::Strikethrough, AttributeValue::Bool(true));
                }
                if let Some(ref url) = link_url {
                    run.attributes.set(
                        AttributeKey::HyperlinkUrl,
                        AttributeValue::String(url.clone()),
                    );
                }

                let text_id = doc.next_id();
                let mut text_node = Node::new(text_id, NodeType::Text);
                text_node.text_content = Some(text_content);

                let _ = doc.insert_node(
                    para_id,
                    doc.node(para_id).map(|n| n.children.len()).unwrap_or(0),
                    run,
                );
                let _ = doc.insert_node(run_id, 0, text_node);
            }
        }
    }

    Ok(doc)
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", "\u{00A0}")
}
