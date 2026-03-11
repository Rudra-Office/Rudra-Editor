//! Generate `word/comments.xml` from CommentBody nodes in the document model.

use s1_model::{AttributeKey, DocumentModel, NodeId, NodeType};

use crate::xml_writer::escape_xml;

/// Collect all CommentBody node IDs from the document root's children.
pub fn collect_comment_bodies(doc: &DocumentModel) -> Vec<NodeId> {
    let root_id = doc.root_id();
    let root = match doc.node(root_id) {
        Some(n) => n,
        None => return Vec::new(),
    };

    root.children
        .iter()
        .filter(|&&id| {
            doc.node(id)
                .is_some_and(|n| n.node_type == NodeType::CommentBody)
        })
        .copied()
        .collect()
}

/// Generate `word/comments.xml` from CommentBody nodes.
///
/// Returns `None` if there are no comments.
pub fn write_comments_xml(doc: &DocumentModel) -> Option<String> {
    let bodies = collect_comment_bodies(doc);
    if bodies.is_empty() {
        return None;
    }

    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push('\n');
    xml.push_str(
        r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
    );

    for &body_id in &bodies {
        let body = match doc.node(body_id) {
            Some(n) => n,
            None => continue,
        };

        let comment_id = body
            .attributes
            .get_string(&AttributeKey::CommentId)
            .unwrap_or("0");
        let author = body
            .attributes
            .get_string(&AttributeKey::CommentAuthor)
            .unwrap_or("Unknown");
        let date = body.attributes.get_string(&AttributeKey::CommentDate);

        xml.push_str(&format!(
            r#"<w:comment w:id="{}" w:author="{}""#,
            escape_xml(comment_id),
            escape_xml(author)
        ));
        if let Some(d) = date {
            xml.push_str(&format!(r#" w:date="{}""#, escape_xml(d)));
        }
        xml.push('>');

        // Write paragraphs inside the comment
        for &child_id in &body.children {
            let child = match doc.node(child_id) {
                Some(n) => n,
                None => continue,
            };
            if child.node_type == NodeType::Paragraph {
                write_comment_paragraph(doc, child_id, &mut xml);
            }
        }

        xml.push_str("</w:comment>");
    }

    xml.push_str("</w:comments>");
    Some(xml)
}

/// Write a paragraph inside a comment.
fn write_comment_paragraph(doc: &DocumentModel, para_id: NodeId, xml: &mut String) {
    let para = match doc.node(para_id) {
        Some(n) => n,
        None => return,
    };

    xml.push_str("<w:p>");

    for &child_id in &para.children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };
        if child.node_type == NodeType::Run {
            write_comment_run(doc, child_id, xml);
        }
    }

    xml.push_str("</w:p>");
}

/// Write a run inside a comment paragraph.
fn write_comment_run(doc: &DocumentModel, run_id: NodeId, xml: &mut String) {
    let run = match doc.node(run_id) {
        Some(n) => n,
        None => return,
    };

    xml.push_str("<w:r>");

    for &child_id in &run.children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };
        if child.node_type == NodeType::Text {
            if let Some(text) = &child.text_content {
                xml.push_str(r#"<w:t xml:space="preserve">"#);
                xml.push_str(&escape_xml(text));
                xml.push_str("</w:t>");
            }
        }
    }

    xml.push_str("</w:r>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::{AttributeValue, Node};

    fn make_comment_doc(id: &str, author: &str, text: &str) -> DocumentModel {
        let mut doc = DocumentModel::new();
        let root_id = doc.root_id();
        let root_children = doc.node(root_id).unwrap().children.len();

        let body_id = doc.next_id();
        let mut body = Node::new(body_id, NodeType::CommentBody);
        body.attributes
            .set(AttributeKey::CommentId, AttributeValue::String(id.into()));
        body.attributes.set(
            AttributeKey::CommentAuthor,
            AttributeValue::String(author.into()),
        );
        doc.insert_node(root_id, root_children, body).unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let run_id = doc.next_id();
        doc.insert_node(para_id, 0, Node::new(run_id, NodeType::Run))
            .unwrap();

        let text_id = doc.next_id();
        doc.insert_node(run_id, 0, Node::text(text_id, text))
            .unwrap();

        doc
    }

    #[test]
    fn write_single_comment() {
        let doc = make_comment_doc("0", "Alice", "Nice work!");
        let xml = write_comments_xml(&doc).unwrap();

        assert!(xml.contains(r#"<w:comment w:id="0" w:author="Alice">"#));
        assert!(xml.contains("Nice work!"));
        assert!(xml.contains("</w:comment>"));
    }

    #[test]
    fn write_no_comments_returns_none() {
        let doc = DocumentModel::new();
        assert!(write_comments_xml(&doc).is_none());
    }

    #[test]
    fn collect_bodies_ignores_non_comments() {
        let doc = DocumentModel::new();
        // Doc has Body as root child; should not be collected
        let bodies = collect_comment_bodies(&doc);
        assert!(bodies.is_empty());
    }

    #[test]
    fn write_comment_with_date() {
        let mut doc = DocumentModel::new();
        let root_id = doc.root_id();
        let root_children = doc.node(root_id).unwrap().children.len();

        let body_id = doc.next_id();
        let mut body = Node::new(body_id, NodeType::CommentBody);
        body.attributes
            .set(AttributeKey::CommentId, AttributeValue::String("1".into()));
        body.attributes.set(
            AttributeKey::CommentAuthor,
            AttributeValue::String("Bob".into()),
        );
        body.attributes.set(
            AttributeKey::CommentDate,
            AttributeValue::String("2024-06-15T12:00:00Z".into()),
        );
        doc.insert_node(root_id, root_children, body).unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let xml = write_comments_xml(&doc).unwrap();
        assert!(xml.contains(r#"w:date="2024-06-15T12:00:00Z""#));
    }
}
