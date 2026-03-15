//! Parse `word/endnotes.xml` — endnote definitions.

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{AttributeKey, AttributeValue, DocumentModel, Node, NodeId, NodeType};
use std::collections::HashMap;

use crate::error::DocxError;
use crate::xml_util::get_attr;

/// A parsed endnote definition from `word/endnotes.xml`.
pub struct EndnoteDef {
    /// The endnote's XML id (w:id attribute).
    pub id: String,
    /// The NodeId of the EndnoteBody node in the document model.
    pub body_node_id: NodeId,
}

/// Parse `word/endnotes.xml` and create EndnoteBody nodes in the document model.
///
/// Endnote body nodes are stored as children of the Document root, similar to
/// comments. Returns a map from endnote XML id to EndnoteDef.
///
/// Endnotes with id="0" (separator) and id="-1" (continuation separator) are
/// skipped as they are special OOXML constructs, not user-authored content.
pub fn parse_endnotes_xml(
    xml: &str,
    doc: &mut DocumentModel,
) -> Result<HashMap<String, EndnoteDef>, DocxError> {
    let mut reader = Reader::from_str(xml);
    let mut endnotes = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"endnote" => {
                let id = get_attr(&e, b"id").unwrap_or_default();

                // Skip separator (id=0) and continuation separator (id=-1)
                if id == "0" || id == "-1" {
                    skip_element(&mut reader)?;
                    continue;
                }

                // Create EndnoteBody node as child of Document root
                let body_id = doc.next_id();
                let mut body = Node::new(body_id, NodeType::EndnoteBody);
                body.attributes.set(
                    AttributeKey::EndnoteNumber,
                    AttributeValue::String(id.clone()),
                );

                let root_id = doc.root_id();
                let root_children = doc.node(root_id).map(|n| n.children.len()).unwrap_or(0);
                doc.insert_node(root_id, root_children, body)
                    .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;

                // Parse endnote content (paragraphs inside the endnote)
                parse_endnote_content(&mut reader, doc, body_id)?;

                endnotes.insert(
                    id.clone(),
                    EndnoteDef {
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

    Ok(endnotes)
}

/// Parse the content inside a `<w:endnote>` element (paragraphs).
fn parse_endnote_content(
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
                        parse_endnote_paragraph(reader, doc, para_id)?;
                        para_index += 1;
                    }
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"endnote" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

/// Parse a paragraph inside an endnote (extract text runs).
fn parse_endnote_paragraph(
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
                        parse_endnote_run(reader, doc, para_id, &mut child_index)?;
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

/// Parse a run inside an endnote paragraph (extract text content).
fn parse_endnote_run(
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
            Ok(Event::Empty(_)) => {} // Skip endnoteRef etc.
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
    fn parse_single_endnote() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:endnote w:id="1">
    <w:p><w:r><w:t>This is an endnote.</w:t></w:r></w:p>
  </w:endnote>
</w:endnotes>"#;

        let mut doc = DocumentModel::new();
        let endnotes = parse_endnotes_xml(xml, &mut doc).unwrap();

        assert_eq!(endnotes.len(), 1);
        let e = endnotes.get("1").unwrap();
        assert_eq!(e.id, "1");

        let body = doc.node(e.body_node_id).unwrap();
        assert_eq!(body.node_type, NodeType::EndnoteBody);
        assert_eq!(
            body.attributes.get_string(&AttributeKey::EndnoteNumber),
            Some("1")
        );
        // Has one paragraph child
        assert_eq!(body.children.len(), 1);
    }

    #[test]
    fn parse_multiple_endnotes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:endnote w:id="1">
    <w:p><w:r><w:t>First endnote</w:t></w:r></w:p>
  </w:endnote>
  <w:endnote w:id="2">
    <w:p><w:r><w:t>Second endnote</w:t></w:r></w:p>
  </w:endnote>
</w:endnotes>"#;

        let mut doc = DocumentModel::new();
        let endnotes = parse_endnotes_xml(xml, &mut doc).unwrap();

        assert_eq!(endnotes.len(), 2);
        assert!(endnotes.contains_key("1"));
        assert!(endnotes.contains_key("2"));
    }

    #[test]
    fn parse_endnote_with_multiple_paragraphs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:endnote w:id="1">
    <w:p><w:r><w:t>First paragraph</w:t></w:r></w:p>
    <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
  </w:endnote>
</w:endnotes>"#;

        let mut doc = DocumentModel::new();
        let endnotes = parse_endnotes_xml(xml, &mut doc).unwrap();

        let e = endnotes.get("1").unwrap();
        let body = doc.node(e.body_node_id).unwrap();
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn parse_skips_separator_endnotes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:endnote w:id="-1" w:type="separator">
    <w:p><w:r><w:separator/></w:r></w:p>
  </w:endnote>
  <w:endnote w:id="0" w:type="continuationSeparator">
    <w:p><w:r><w:continuationSeparator/></w:r></w:p>
  </w:endnote>
  <w:endnote w:id="1">
    <w:p><w:r><w:t>Real endnote</w:t></w:r></w:p>
  </w:endnote>
</w:endnotes>"#;

        let mut doc = DocumentModel::new();
        let endnotes = parse_endnotes_xml(xml, &mut doc).unwrap();

        // Only the real endnote should be parsed
        assert_eq!(endnotes.len(), 1);
        assert!(endnotes.contains_key("1"));
    }

    #[test]
    fn parse_empty_endnotes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
</w:endnotes>"#;

        let mut doc = DocumentModel::new();
        let endnotes = parse_endnotes_xml(xml, &mut doc).unwrap();
        assert!(endnotes.is_empty());
    }
}
