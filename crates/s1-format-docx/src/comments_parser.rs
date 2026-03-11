//! Parse `word/comments.xml` — comment definitions.

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{AttributeKey, AttributeValue, DocumentModel, Node, NodeId, NodeType};
use std::collections::HashMap;

use crate::error::DocxError;
use crate::xml_util::get_attr;

/// A parsed comment definition from `word/comments.xml`.
pub struct CommentDef {
    /// The comment's XML id (w:id attribute).
    pub id: String,
    /// The NodeId of the CommentBody node in the document model.
    pub body_node_id: NodeId,
}

/// Parse `word/comments.xml` and create CommentBody nodes in the document model.
///
/// Comment body nodes are stored as children of the Document root, similar to
/// headers and footers. Returns a map from comment XML id → CommentDef.
pub fn parse_comments_xml(
    xml: &str,
    doc: &mut DocumentModel,
) -> Result<HashMap<String, CommentDef>, DocxError> {
    let mut reader = Reader::from_str(xml);
    let mut comments = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"comment" => {
                let id = get_attr(&e, b"id").unwrap_or_default();
                let author = get_attr(&e, b"author");
                let date = get_attr(&e, b"date");

                // Create CommentBody node as child of Document root
                let body_id = doc.next_id();
                let mut body = Node::new(body_id, NodeType::CommentBody);
                body.attributes
                    .set(AttributeKey::CommentId, AttributeValue::String(id.clone()));
                if let Some(ref a) = author {
                    body.attributes.set(
                        AttributeKey::CommentAuthor,
                        AttributeValue::String(a.clone()),
                    );
                }
                if let Some(ref d) = date {
                    body.attributes
                        .set(AttributeKey::CommentDate, AttributeValue::String(d.clone()));
                }

                let root_id = doc.root_id();
                let root_children = doc.node(root_id).map(|n| n.children.len()).unwrap_or(0);
                doc.insert_node(root_id, root_children, body)
                    .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;

                // Parse comment content (paragraphs inside the comment)
                parse_comment_content(&mut reader, doc, body_id)?;

                comments.insert(
                    id.clone(),
                    CommentDef {
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

    Ok(comments)
}

/// Parse the content inside a `<w:comment>` element (paragraphs).
fn parse_comment_content(
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
                        parse_comment_paragraph(reader, doc, para_id)?;
                        para_index += 1;
                    }
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"comment" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

/// Parse a paragraph inside a comment (extract text runs).
fn parse_comment_paragraph(
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
                        parse_comment_run(reader, doc, para_id, &mut child_index)?;
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

/// Parse a run inside a comment paragraph (extract text content).
fn parse_comment_run(
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
            Ok(Event::Empty(_)) => {} // Skip annotationRef etc.
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
    fn parse_single_comment() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:comment w:id="0" w:author="Alice" w:date="2024-03-12T10:30:00Z">
    <w:p><w:r><w:t>Great point!</w:t></w:r></w:p>
  </w:comment>
</w:comments>"#;

        let mut doc = DocumentModel::new();
        let comments = parse_comments_xml(xml, &mut doc).unwrap();

        assert_eq!(comments.len(), 1);
        let c = comments.get("0").unwrap();
        assert_eq!(c.id, "0");

        let body = doc.node(c.body_node_id).unwrap();
        assert_eq!(body.node_type, NodeType::CommentBody);
        assert_eq!(
            body.attributes.get_string(&AttributeKey::CommentId),
            Some("0")
        );
        assert_eq!(
            body.attributes.get_string(&AttributeKey::CommentAuthor),
            Some("Alice")
        );
        assert_eq!(
            body.attributes.get_string(&AttributeKey::CommentDate),
            Some("2024-03-12T10:30:00Z")
        );
        // Has one paragraph child
        assert_eq!(body.children.len(), 1);
    }

    #[test]
    fn parse_multiple_comments() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:comment w:id="0" w:author="Alice" w:date="2024-01-01T00:00:00Z">
    <w:p><w:r><w:t>Comment one</w:t></w:r></w:p>
  </w:comment>
  <w:comment w:id="1" w:author="Bob">
    <w:p><w:r><w:t>Comment two</w:t></w:r></w:p>
  </w:comment>
</w:comments>"#;

        let mut doc = DocumentModel::new();
        let comments = parse_comments_xml(xml, &mut doc).unwrap();

        assert_eq!(comments.len(), 2);
        assert!(comments.contains_key("0"));
        assert!(comments.contains_key("1"));

        let c1 = doc.node(comments["1"].body_node_id).unwrap();
        assert_eq!(
            c1.attributes.get_string(&AttributeKey::CommentAuthor),
            Some("Bob")
        );
        // No date on comment 1
        assert!(c1
            .attributes
            .get_string(&AttributeKey::CommentDate)
            .is_none());
    }

    #[test]
    fn parse_comment_with_multiple_paragraphs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:comment w:id="0" w:author="Alice">
    <w:p><w:r><w:t>First paragraph</w:t></w:r></w:p>
    <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
  </w:comment>
</w:comments>"#;

        let mut doc = DocumentModel::new();
        let comments = parse_comments_xml(xml, &mut doc).unwrap();

        let c = comments.get("0").unwrap();
        let body = doc.node(c.body_node_id).unwrap();
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn parse_empty_comments() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
</w:comments>"#;

        let mut doc = DocumentModel::new();
        let comments = parse_comments_xml(xml, &mut doc).unwrap();
        assert!(comments.is_empty());
    }
}
