//! Generate `word/endnotes.xml` from EndnoteBody nodes in the document model.

use s1_model::{AttributeKey, DocumentModel, NodeId, NodeType};

use crate::xml_writer::escape_xml;

/// Collect all EndnoteBody node IDs from the document root's children.
pub fn collect_endnote_bodies(doc: &DocumentModel) -> Vec<NodeId> {
    let root_id = doc.root_id();
    let root = match doc.node(root_id) {
        Some(n) => n,
        None => return Vec::new(),
    };

    root.children
        .iter()
        .filter(|&&id| {
            doc.node(id)
                .is_some_and(|n| n.node_type == NodeType::EndnoteBody)
        })
        .copied()
        .collect()
}

/// Generate `word/endnotes.xml` from EndnoteBody nodes.
///
/// Returns `None` if there are no endnotes.
/// Always includes separator (id=0) and continuation separator (id=1) endnotes
/// as required by the OOXML specification.
pub fn write_endnotes_xml(doc: &DocumentModel) -> Option<String> {
    let bodies = collect_endnote_bodies(doc);
    if bodies.is_empty() {
        return None;
    }

    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push('\n');
    xml.push_str(
        r#"<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
    );

    // Standard separator endnote (id=0)
    xml.push_str(
        r#"<w:endnote w:type="separator" w:id="0"><w:p><w:r><w:separator/></w:r></w:p></w:endnote>"#,
    );

    // Standard continuation separator endnote (id=1)
    xml.push_str(
        r#"<w:endnote w:type="continuationSeparator" w:id="1"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:endnote>"#,
    );

    for &body_id in &bodies {
        let body = match doc.node(body_id) {
            Some(n) => n,
            None => continue,
        };

        let endnote_id = body
            .attributes
            .get_string(&AttributeKey::EndnoteNumber)
            .unwrap_or("0");

        xml.push_str(&format!(r#"<w:endnote w:id="{}">"#, escape_xml(endnote_id)));

        // Write paragraphs inside the endnote
        for &child_id in &body.children {
            let child = match doc.node(child_id) {
                Some(n) => n,
                None => continue,
            };
            if child.node_type == NodeType::Paragraph {
                write_endnote_paragraph(doc, child_id, &mut xml);
            }
        }

        xml.push_str("</w:endnote>");
    }

    xml.push_str("</w:endnotes>");
    Some(xml)
}

/// Write a paragraph inside an endnote.
fn write_endnote_paragraph(doc: &DocumentModel, para_id: NodeId, xml: &mut String) {
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
            write_endnote_run(doc, child_id, xml);
        }
    }

    xml.push_str("</w:p>");
}

/// Write a run inside an endnote paragraph.
fn write_endnote_run(doc: &DocumentModel, run_id: NodeId, xml: &mut String) {
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

    fn make_endnote_doc(id: &str, text: &str) -> DocumentModel {
        let mut doc = DocumentModel::new();
        let root_id = doc.root_id();
        let root_children = doc.node(root_id).unwrap().children.len();

        let body_id = doc.next_id();
        let mut body = Node::new(body_id, NodeType::EndnoteBody);
        body.attributes.set(
            AttributeKey::EndnoteNumber,
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
    fn write_single_endnote() {
        let doc = make_endnote_doc("2", "An endnote.");
        let xml = write_endnotes_xml(&doc).unwrap();

        assert!(xml.contains(r#"<w:endnote w:id="2">"#));
        assert!(xml.contains("An endnote."));
        assert!(xml.contains("</w:endnote>"));
        // Should include separator endnotes
        assert!(xml.contains(r#"<w:endnote w:type="separator" w:id="0">"#));
        assert!(xml.contains(r#"<w:endnote w:type="continuationSeparator" w:id="1">"#));
    }

    #[test]
    fn write_no_endnotes_returns_none() {
        let doc = DocumentModel::new();
        assert!(write_endnotes_xml(&doc).is_none());
    }

    #[test]
    fn collect_bodies_ignores_non_endnotes() {
        let doc = DocumentModel::new();
        // Doc has Body as root child; should not be collected
        let bodies = collect_endnote_bodies(&doc);
        assert!(bodies.is_empty());
    }

    #[test]
    fn write_endnote_escapes_xml() {
        let doc = make_endnote_doc("3", "A & B < C");
        let xml = write_endnotes_xml(&doc).unwrap();

        assert!(xml.contains("A &amp; B &lt; C"));
    }
}
