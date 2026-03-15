//! Parse `word/header*.xml` and `word/footer*.xml` into the document model.
//!
//! Headers and footers are stored as children of the Document root node,
//! with `NodeType::Header` or `NodeType::Footer`.

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{DocumentModel, Node, NodeId, NodeType, NumberingDefinitions};

use crate::content_parser;
use crate::error::DocxError;

/// Parse a header XML file and return the NodeId of the created Header node.
///
/// The Header node is added as a child of the Document root.
pub fn parse_header_xml(
    xml: &str,
    doc: &mut DocumentModel,
    rels: &HashMap<String, String>,
    media: &HashMap<String, Vec<u8>>,
    numbering: &NumberingDefinitions,
) -> Result<NodeId, DocxError> {
    parse_hf_xml(xml, doc, rels, media, numbering, NodeType::Header, b"hdr")
}

/// Parse a footer XML file and return the NodeId of the created Footer node.
///
/// The Footer node is added as a child of the Document root.
pub fn parse_footer_xml(
    xml: &str,
    doc: &mut DocumentModel,
    rels: &HashMap<String, String>,
    media: &HashMap<String, Vec<u8>>,
    numbering: &NumberingDefinitions,
) -> Result<NodeId, DocxError> {
    parse_hf_xml(xml, doc, rels, media, numbering, NodeType::Footer, b"ftr")
}

/// Common parser for both headers and footers.
fn parse_hf_xml(
    xml: &str,
    doc: &mut DocumentModel,
    rels: &HashMap<String, String>,
    media: &HashMap<String, Vec<u8>>,
    numbering: &NumberingDefinitions,
    node_type: NodeType,
    root_tag: &[u8],
) -> Result<NodeId, DocxError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    // Find the root element (w:hdr or w:ftr)
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == root_tag => {
                break;
            }
            Ok(Event::Eof) => {
                return Err(DocxError::InvalidStructure(
                    "Missing root element in header/footer XML".to_string(),
                ));
            }
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    // Create the Header/Footer node as child of Document root
    let hf_id = doc.next_id();
    let root_id = doc.root_id();
    let root_children = doc.node(root_id).map(|n| n.children.len()).unwrap_or(0);
    doc.insert_node(root_id, root_children, Node::new(hf_id, node_type))
        .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;

    // Parse block-level content inside the header/footer
    content_parser::parse_block_content(&mut reader, doc, hf_id, rels, media, numbering, root_tag)?;

    Ok(hf_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::{AttributeKey, AttributeValue, FieldType, NumberingDefinitions};

    #[test]
    fn parse_header_with_paragraph() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:p><w:r><w:t>Header text</w:t></w:r></w:p>
</w:hdr>"#;

        let mut doc = DocumentModel::new();
        let rels = HashMap::new();
        let media = HashMap::new();
        let numbering = NumberingDefinitions::default();

        let hdr_id = parse_header_xml(xml, &mut doc, &rels, &media, &numbering).unwrap();

        let hdr = doc.node(hdr_id).unwrap();
        assert_eq!(hdr.node_type, NodeType::Header);
        assert_eq!(hdr.children.len(), 1); // one paragraph

        let para = doc.node(hdr.children[0]).unwrap();
        assert_eq!(para.node_type, NodeType::Paragraph);
    }

    #[test]
    fn parse_footer_with_paragraph() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:p><w:r><w:t>Footer text</w:t></w:r></w:p>
</w:ftr>"#;

        let mut doc = DocumentModel::new();
        let rels = HashMap::new();
        let media = HashMap::new();
        let numbering = NumberingDefinitions::default();

        let ftr_id = parse_footer_xml(xml, &mut doc, &rels, &media, &numbering).unwrap();

        let ftr = doc.node(ftr_id).unwrap();
        assert_eq!(ftr.node_type, NodeType::Footer);
        assert_eq!(ftr.children.len(), 1);
    }

    #[test]
    fn parse_header_with_multiple_paragraphs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:p><w:r><w:t>Line 1</w:t></w:r></w:p>
    <w:p><w:r><w:t>Line 2</w:t></w:r></w:p>
</w:hdr>"#;

        let mut doc = DocumentModel::new();
        let rels = HashMap::new();
        let media = HashMap::new();
        let numbering = NumberingDefinitions::default();

        let hdr_id = parse_header_xml(xml, &mut doc, &rels, &media, &numbering).unwrap();

        let hdr = doc.node(hdr_id).unwrap();
        assert_eq!(hdr.children.len(), 2);
    }

    #[test]
    fn parse_footer_with_complex_field_page_number() {
        // Footer with non-self-closing fldChar elements spanning multiple runs
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:p>
        <w:r><w:fldChar w:fldCharType="begin"></w:fldChar></w:r>
        <w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r>
        <w:r><w:fldChar w:fldCharType="separate"></w:fldChar></w:r>
        <w:r><w:t>1</w:t></w:r>
        <w:r><w:fldChar w:fldCharType="end"></w:fldChar></w:r>
    </w:p>
</w:ftr>"#;

        let mut doc = DocumentModel::new();
        let rels = HashMap::new();
        let media = HashMap::new();
        let numbering = NumberingDefinitions::default();

        let ftr_id = parse_footer_xml(xml, &mut doc, &rels, &media, &numbering).unwrap();

        let ftr = doc.node(ftr_id).unwrap();
        assert_eq!(ftr.node_type, NodeType::Footer);
        assert_eq!(ftr.children.len(), 1);

        let para = doc.node(ftr.children[0]).unwrap();
        assert_eq!(para.node_type, NodeType::Paragraph);

        // Should have a Field node for PAGE
        assert!(
            !para.children.is_empty(),
            "Footer paragraph should have a field node"
        );
        let field = doc.node(para.children[0]).unwrap();
        assert_eq!(field.node_type, NodeType::Field);
        assert_eq!(
            field.attributes.get(&AttributeKey::FieldType),
            Some(&AttributeValue::FieldType(FieldType::PageNumber))
        );
    }
}
