//! Parse `word/footnotes.xml` — footnote definitions.

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{AttributeKey, AttributeValue, DocumentModel, Node, NodeId, NodeType};
use std::collections::HashMap;

use crate::error::DocxError;
use crate::xml_util::get_attr;

/// A parsed footnote definition from `word/footnotes.xml`.
pub struct FootnoteDef {
    /// The footnote's XML id (w:id attribute).
    pub id: String,
    /// The NodeId of the FootnoteBody node in the document model.
    pub body_node_id: NodeId,
}

/// Parse `word/footnotes.xml` and create FootnoteBody nodes in the document model.
///
/// Footnote body nodes are stored as children of the Document root, similar to
/// comments. Returns a map from footnote XML id to FootnoteDef.
///
/// Footnotes with id="0" (separator) and id="-1" (continuation separator) are
/// skipped as they are special OOXML constructs, not user-authored content.
pub fn parse_footnotes_xml(
    xml: &str,
    doc: &mut DocumentModel,
) -> Result<HashMap<String, FootnoteDef>, DocxError> {
    let mut reader = Reader::from_str(xml);
    let mut footnotes = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"footnote" => {
                let id = get_attr(&e, b"id").unwrap_or_default();

                // Skip separator (id=0) and continuation separator (id=-1)
                if id == "0" || id == "-1" {
                    skip_element(&mut reader)?;
                    continue;
                }

                // Create FootnoteBody node as child of Document root
                let body_id = doc.next_id();
                let mut body = Node::new(body_id, NodeType::FootnoteBody);
                body.attributes.set(
                    AttributeKey::FootnoteNumber,
                    AttributeValue::String(id.clone()),
                );

                let root_id = doc.root_id();
                let root_children = doc.node(root_id).map(|n| n.children.len()).unwrap_or(0);
                doc.insert_node(root_id, root_children, body)
                    .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;

                // Parse footnote content (paragraphs inside the footnote)
                parse_footnote_content(&mut reader, doc, body_id)?;

                footnotes.insert(
                    id.clone(),
                    FootnoteDef {
                        id,
                        body_node_id: body_id,
                    },
                );
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    Ok(footnotes)
}

/// Parse the content inside a `<w:footnote>` element (paragraphs).
fn parse_footnote_content(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    body_id: NodeId,
) -> Result<(), DocxError> {
    let mut para_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"p" => {
                        let para_id = doc.next_id();
                        doc.insert_node(
                            body_id,
                            para_index,
                            Node::new(para_id, NodeType::Paragraph),
                        )
                        .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;
                        parse_footnote_paragraph(reader, doc, para_id)?;
                        para_index += 1;
                    }
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"footnote" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

/// Parse a paragraph inside a footnote (extract text runs).
fn parse_footnote_paragraph(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    para_id: NodeId,
) -> Result<(), DocxError> {
    let mut child_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"r" => {
                        parse_footnote_run(reader, doc, para_id, &mut child_index)?;
                    }
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"p" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

/// Parse a run inside a footnote paragraph (extract text content).
fn parse_footnote_run(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    para_id: NodeId,
    child_index: &mut usize,
) -> Result<(), DocxError> {
    let run_id = doc.next_id();
    doc.insert_node(para_id, *child_index, Node::new(run_id, NodeType::Run))
        .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;
    *child_index += 1;

    let mut text_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"t" => {
                        let text_content = reader
                            .read_text(e.to_end().name())
                            .map_err(|e| DocxError::Xml(format!("Error reading text: {e}")))?;
                        if !text_content.is_empty() {
                            let text_id = doc.next_id();
                            doc.insert_node(
                                run_id,
                                text_index,
                                Node::text(text_id, &*text_content),
                            )
                            .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;
                            text_index += 1;
                        }
                    }
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::Empty(_)) => {} // Skip footnoteRef etc.
            Ok(Event::End(e)) if e.local_name().as_ref() == b"r" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

/// Skip an element and all its descendants.
fn skip_element(reader: &mut Reader<&[u8]>) -> Result<(), DocxError> {
    let mut depth = 1u32;
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_footnote() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:id="1">
    <w:p><w:r><w:t>This is a footnote.</w:t></w:r></w:p>
  </w:footnote>
</w:footnotes>"#;

        let mut doc = DocumentModel::new();
        let footnotes = parse_footnotes_xml(xml, &mut doc).unwrap();

        assert_eq!(footnotes.len(), 1);
        let f = footnotes.get("1").unwrap();
        assert_eq!(f.id, "1");

        let body = doc.node(f.body_node_id).unwrap();
        assert_eq!(body.node_type, NodeType::FootnoteBody);
        assert_eq!(
            body.attributes.get_string(&AttributeKey::FootnoteNumber),
            Some("1")
        );
        // Has one paragraph child
        assert_eq!(body.children.len(), 1);
    }

    #[test]
    fn parse_multiple_footnotes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:id="1">
    <w:p><w:r><w:t>First footnote</w:t></w:r></w:p>
  </w:footnote>
  <w:footnote w:id="2">
    <w:p><w:r><w:t>Second footnote</w:t></w:r></w:p>
  </w:footnote>
</w:footnotes>"#;

        let mut doc = DocumentModel::new();
        let footnotes = parse_footnotes_xml(xml, &mut doc).unwrap();

        assert_eq!(footnotes.len(), 2);
        assert!(footnotes.contains_key("1"));
        assert!(footnotes.contains_key("2"));
    }

    #[test]
    fn parse_footnote_with_multiple_paragraphs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:id="1">
    <w:p><w:r><w:t>First paragraph</w:t></w:r></w:p>
    <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
  </w:footnote>
</w:footnotes>"#;

        let mut doc = DocumentModel::new();
        let footnotes = parse_footnotes_xml(xml, &mut doc).unwrap();

        let f = footnotes.get("1").unwrap();
        let body = doc.node(f.body_node_id).unwrap();
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn parse_skips_separator_footnotes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:id="-1" w:type="separator">
    <w:p><w:r><w:separator/></w:r></w:p>
  </w:footnote>
  <w:footnote w:id="0" w:type="continuationSeparator">
    <w:p><w:r><w:continuationSeparator/></w:r></w:p>
  </w:footnote>
  <w:footnote w:id="1">
    <w:p><w:r><w:t>Real footnote</w:t></w:r></w:p>
  </w:footnote>
</w:footnotes>"#;

        let mut doc = DocumentModel::new();
        let footnotes = parse_footnotes_xml(xml, &mut doc).unwrap();

        // Only the real footnote should be parsed
        assert_eq!(footnotes.len(), 1);
        assert!(footnotes.contains_key("1"));
    }

    #[test]
    fn parse_empty_footnotes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
</w:footnotes>"#;

        let mut doc = DocumentModel::new();
        let footnotes = parse_footnotes_xml(xml, &mut doc).unwrap();
        assert!(footnotes.is_empty());
    }
}
