//! Parse `word/document.xml` — the main document content.
//!
//! Handles paragraphs, runs, text, breaks, and tabs. Tables, images,
//! headers/footers, and lists are deferred to Phase 2.

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{AttributeMap, DocumentModel, Node, NodeId, NodeType};

use crate::error::DocxError;
use crate::property_parser::{parse_paragraph_properties, parse_run_properties};
use crate::xml_util::get_attr;

/// Parse `word/document.xml` into the document model.
pub fn parse_document_xml(xml: &str, doc: &mut DocumentModel) -> Result<(), DocxError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"body" => {
                parse_body(&mut reader, doc)?;
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    Ok(())
}

/// Parse `<w:body>` contents.
fn parse_body(reader: &mut Reader<&[u8]>, doc: &mut DocumentModel) -> Result<(), DocxError> {
    let body_id = doc
        .body_id()
        .ok_or_else(|| DocxError::InvalidStructure("No body node in model".into()))?;

    let mut child_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"p" => {
                        parse_paragraph(reader, doc, body_id, child_index)?;
                        child_index += 1;
                    }
                    // Tables, sections — skip for Phase 1
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"body" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    Ok(())
}

/// Parse `<w:p>` — a paragraph.
fn parse_paragraph(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    parent_id: NodeId,
    index: usize,
) -> Result<(), DocxError> {
    let para_id = doc.next_id();
    doc.insert_node(parent_id, index, Node::new(para_id, NodeType::Paragraph))
        .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;

    let mut child_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"pPr" => {
                        let attrs = parse_paragraph_properties(reader)?;
                        if let Some(node) = doc.node_mut(para_id) {
                            node.attributes = attrs;
                        }
                    }
                    b"r" => {
                        parse_run(reader, doc, para_id, &mut child_index)?;
                    }
                    // Hyperlinks contain runs — extract the runs
                    b"hyperlink" => {
                        parse_hyperlink_runs(reader, doc, para_id, &mut child_index)?;
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

/// Intermediate representation for run content before building nodes.
enum RunContent {
    Text(String),
    Break(NodeType),
    Tab,
}

/// Parse `<w:r>` — a run of text with formatting.
fn parse_run(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    para_id: NodeId,
    child_index: &mut usize,
) -> Result<(), DocxError> {
    let mut run_attrs = AttributeMap::new();
    let mut content: Vec<RunContent> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"rPr" => {
                        run_attrs = parse_run_properties(reader)?;
                    }
                    b"t" => {
                        let text = read_text_content(reader)?;
                        if !text.is_empty() {
                            content.push(RunContent::Text(text));
                        }
                    }
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"br" => {
                        let break_type = get_attr(&e, b"type");
                        let node_type = match break_type.as_deref() {
                            Some("page") => NodeType::PageBreak,
                            Some("column") => NodeType::ColumnBreak,
                            _ => NodeType::LineBreak,
                        };
                        content.push(RunContent::Break(node_type));
                    }
                    b"tab" => {
                        content.push(RunContent::Tab);
                    }
                    b"t" => {
                        // Self-closing <w:t/> — empty text, skip
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"r" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    // Build nodes from collected content.
    // Text items go into runs; breaks/tabs go directly into the paragraph.
    let mut texts: Vec<String> = Vec::new();

    for item in content {
        match item {
            RunContent::Text(text) => {
                texts.push(text);
            }
            RunContent::Break(node_type) => {
                flush_texts_to_run(&mut texts, &run_attrs, doc, para_id, child_index)?;
                let break_id = doc.next_id();
                doc.insert_node(
                    para_id,
                    *child_index,
                    Node::new(break_id, node_type),
                )
                .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;
                *child_index += 1;
            }
            RunContent::Tab => {
                flush_texts_to_run(&mut texts, &run_attrs, doc, para_id, child_index)?;
                let tab_id = doc.next_id();
                doc.insert_node(
                    para_id,
                    *child_index,
                    Node::new(tab_id, NodeType::Tab),
                )
                .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;
                *child_index += 1;
            }
        }
    }

    // Flush remaining text
    flush_texts_to_run(&mut texts, &run_attrs, doc, para_id, child_index)?;

    Ok(())
}

/// Create a Run node with accumulated text content.
fn flush_texts_to_run(
    texts: &mut Vec<String>,
    run_attrs: &AttributeMap,
    doc: &mut DocumentModel,
    para_id: NodeId,
    child_index: &mut usize,
) -> Result<(), DocxError> {
    if texts.is_empty() {
        return Ok(());
    }

    let combined: String = texts.drain(..).collect();
    if combined.is_empty() {
        return Ok(());
    }

    let run_id = doc.next_id();
    let mut run = Node::new(run_id, NodeType::Run);
    run.attributes = run_attrs.clone();
    doc.insert_node(para_id, *child_index, run)
        .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;
    *child_index += 1;

    let text_id = doc.next_id();
    doc.insert_node(run_id, 0, Node::text(text_id, combined))
        .map_err(|e| DocxError::InvalidStructure(format!("{e}")))?;

    Ok(())
}

/// Read text content inside `<w:t>...</w:t>`.
fn read_text_content(reader: &mut Reader<&[u8]>) -> Result<String, DocxError> {
    let mut text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Text(e)) => {
                text.push_str(&e.unescape().map_err(|e| DocxError::Xml(format!("{e}")))?);
            }
            Ok(Event::CData(e)) => {
                text.push_str(
                    std::str::from_utf8(&e).map_err(|e| DocxError::Xml(format!("{e}")))?,
                );
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"t" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    Ok(text)
}

/// Parse runs inside `<w:hyperlink>` — we extract the text but don't yet
/// handle the hyperlink relationship (Phase 2).
fn parse_hyperlink_runs(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    para_id: NodeId,
    child_index: &mut usize,
) -> Result<(), DocxError> {
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"r" => {
                        parse_run(reader, doc, para_id, child_index)?;
                    }
                    _ => {
                        skip_element(reader)?;
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"hyperlink" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

/// Skip an element and all its children.
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
    use s1_model::AttributeKey;

    fn wrap_doc(body_content: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>{body_content}</w:body>
</w:document>"#
        )
    }

    #[test]
    fn parse_single_paragraph() {
        let xml = wrap_doc(r#"<w:p><w:r><w:t>Hello World</w:t></w:r></w:p>"#);
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();
        assert_eq!(doc.to_plain_text(), "Hello World");
    }

    #[test]
    fn parse_multiple_paragraphs() {
        let xml = wrap_doc(
            r#"<w:p><w:r><w:t>First</w:t></w:r></w:p>
            <w:p><w:r><w:t>Second</w:t></w:r></w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();
        assert_eq!(doc.to_plain_text(), "First\nSecond");
    }

    #[test]
    fn parse_empty_paragraph() {
        let xml = wrap_doc(r#"<w:p></w:p>"#);
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1); // one empty paragraph
    }

    #[test]
    fn parse_bold_run() {
        let xml =
            wrap_doc(r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r></w:p>"#);
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();

        // Find the run node
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();
        let run = doc.node(para.children[0]).unwrap();

        assert_eq!(run.attributes.get_bool(&AttributeKey::Bold), Some(true));
        assert_eq!(doc.to_plain_text(), "Bold");
    }

    #[test]
    fn parse_multiple_runs() {
        let xml = wrap_doc(
            r#"<w:p>
            <w:r><w:rPr><w:b/></w:rPr><w:t>Hello </w:t></w:r>
            <w:r><w:rPr><w:i/></w:rPr><w:t>World</w:t></w:r>
            </w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();
        assert_eq!(doc.to_plain_text(), "Hello World");

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();
        assert_eq!(para.children.len(), 2); // two runs

        let run1 = doc.node(para.children[0]).unwrap();
        assert_eq!(run1.attributes.get_bool(&AttributeKey::Bold), Some(true));

        let run2 = doc.node(para.children[1]).unwrap();
        assert_eq!(run2.attributes.get_bool(&AttributeKey::Italic), Some(true));
    }

    #[test]
    fn parse_paragraph_alignment() {
        let xml = wrap_doc(
            r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Centered</w:t></w:r></w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();

        assert_eq!(
            para.attributes.get_alignment(&AttributeKey::Alignment),
            Some(s1_model::Alignment::Center)
        );
    }

    #[test]
    fn parse_line_break() {
        let xml = wrap_doc(
            r#"<w:p><w:r><w:t>Before</w:t><w:br/><w:t>After</w:t></w:r></w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();

        // Should produce: Paragraph > Run("Before"), LineBreak, Run("After")
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();

        assert_eq!(para.children.len(), 3);
        let child0 = doc.node(para.children[0]).unwrap();
        assert_eq!(child0.node_type, NodeType::Run);

        let child1 = doc.node(para.children[1]).unwrap();
        assert_eq!(child1.node_type, NodeType::LineBreak);

        let child2 = doc.node(para.children[2]).unwrap();
        assert_eq!(child2.node_type, NodeType::Run);
    }

    #[test]
    fn parse_page_break() {
        let xml = wrap_doc(
            r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();

        assert_eq!(para.children.len(), 1);
        let child = doc.node(para.children[0]).unwrap();
        assert_eq!(child.node_type, NodeType::PageBreak);
    }

    #[test]
    fn parse_tab() {
        let xml = wrap_doc(
            r#"<w:p><w:r><w:t>Col1</w:t><w:tab/><w:t>Col2</w:t></w:r></w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();

        assert_eq!(para.children.len(), 3); // Run, Tab, Run
        let child1 = doc.node(para.children[1]).unwrap();
        assert_eq!(child1.node_type, NodeType::Tab);
    }

    #[test]
    fn parse_paragraph_with_style_ref() {
        let xml = wrap_doc(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();

        assert_eq!(
            para.attributes.get_string(&AttributeKey::StyleId),
            Some("Heading1")
        );
    }

    #[test]
    fn parse_unknown_elements_ignored() {
        // Unknown elements should be silently skipped, not cause errors
        let xml = wrap_doc(
            r#"<w:p>
            <w:bookmarkStart w:id="0" w:name="_GoBack"/>
            <w:r><w:t>Text</w:t></w:r>
            <w:bookmarkEnd w:id="0"/>
            </w:p>"#,
        );
        let mut doc = DocumentModel::new();
        parse_document_xml(&xml, &mut doc).unwrap();
        assert_eq!(doc.to_plain_text(), "Text");
    }
}
