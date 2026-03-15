//! Generate `word/footnotes.xml` from FootnoteBody nodes in the document model.

use s1_model::{AttributeKey, DocumentModel, NodeId, NodeType};

use crate::xml_writer::escape_xml;

/// Collect all FootnoteBody node IDs from the document root's children.
pub fn collect_footnote_bodies(doc: &DocumentModel) -> Vec<NodeId> {
    let root_id = doc.root_id();
    let root = match doc.node(root_id) {
        Some(n) => n,
        None => return Vec::new(),
    };

    root.children
        .iter()
        .filter(|&&id| {
            doc.node(id)
                .is_some_and(|n| n.node_type == NodeType::FootnoteBody)
        })
        .copied()
        .collect()
}

/// Generate `word/footnotes.xml` from FootnoteBody nodes.
///
/// Returns `None` if there are no footnotes.
/// Always includes separator (id=0) and continuation separator (id=1) footnotes
/// as required by the OOXML specification.
pub fn write_footnotes_xml(doc: &DocumentModel) -> Option<String> {
    let bodies = collect_footnote_bodies(doc);
    if bodies.is_empty() {
        return None;
    }

    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push('\n');
    xml.push_str(
        r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
    );

    // Standard separator footnote (id=0)
    xml.push_str(
        r#"<w:footnote w:type="separator" w:id="0"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>"#,
    );

    // Standard continuation separator footnote (id=1)
    xml.push_str(
        r#"<w:footnote w:type="continuationSeparator" w:id="1"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>"#,
    );

    for &body_id in &bodies {
        let body = match doc.node(body_id) {
            Some(n) => n,
            None => continue,
        };

        let footnote_id = body
            .attributes
            .get_string(&AttributeKey::FootnoteNumber)
            .unwrap_or("0");

        xml.push_str(&format!(
            r#"<w:footnote w:id="{}">"#,
            escape_xml(footnote_id)
        ));

        // Write paragraphs inside the footnote
        for &child_id in &body.children {
            let child = match doc.node(child_id) {
                Some(n) => n,
                None => continue,
            };
            if child.node_type == NodeType::Paragraph {
                write_footnote_paragraph(doc, child_id, &mut xml);
            }
        }

        xml.push_str("</w:footnote>");
    }

    xml.push_str("</w:footnotes>");
    Some(xml)
}

/// Write a paragraph inside a footnote.
fn write_footnote_paragraph(doc: &DocumentModel, para_id: NodeId, xml: &mut String) {
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
            write_footnote_run(doc, child_id, xml);
        }
    }

    xml.push_str("</w:p>");
}

/// Write a run inside a footnote paragraph.
fn write_footnote_run(doc: &DocumentModel, run_id: NodeId, xml: &mut String) {
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

    fn make_footnote_doc(id: &str, text: &str) -> DocumentModel {
        let mut doc = DocumentModel::new();
        let root_id = doc.root_id();
        let root_children = doc.node(root_id).unwrap().children.len();

        let body_id = doc.next_id();
        let mut body = Node::new(body_id, NodeType::FootnoteBody);
        body.attributes.set(
            AttributeKey::FootnoteNumber,
            AttributeValue::String(id.into()),
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
    fn write_single_footnote() {
        let doc = make_footnote_doc("2", "A footnote.");
        let xml = write_footnotes_xml(&doc).unwrap();

        assert!(xml.contains(r#"<w:footnote w:id="2">"#));
        assert!(xml.contains("A footnote."));
        assert!(xml.contains("</w:footnote>"));
        // Should include separator footnotes
        assert!(xml.contains(r#"<w:footnote w:type="separator" w:id="0">"#));
        assert!(xml.contains(r#"<w:footnote w:type="continuationSeparator" w:id="1">"#));
    }

    #[test]
    fn write_no_footnotes_returns_none() {
        let doc = DocumentModel::new();
        assert!(write_footnotes_xml(&doc).is_none());
    }

    #[test]
    fn collect_bodies_ignores_non_footnotes() {
        let doc = DocumentModel::new();
        // Doc has Body as root child; should not be collected
        let bodies = collect_footnote_bodies(&doc);
        assert!(bodies.is_empty());
    }

    #[test]
    fn write_footnote_escapes_xml() {
        let doc = make_footnote_doc("3", "A & B < C");
        let xml = write_footnotes_xml(&doc).unwrap();

        assert!(xml.contains("A &amp; B &lt; C"));
    }
}
