//! DOCX writer — main entry point.
//!
//! Writes a [`DocumentModel`] as a DOCX file (ZIP archive).

use std::io::{Cursor, Write};

use s1_model::DocumentModel;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::content_writer::write_document_xml;
use crate::error::DocxError;
use crate::metadata_writer::write_core_xml;
use crate::style_writer::write_styles_xml;

/// Write a [`DocumentModel`] as DOCX bytes.
///
/// The output is a valid ZIP archive containing the OOXML structure:
/// - `[Content_Types].xml`
/// - `_rels/.rels`
/// - `word/document.xml`
/// - `word/_rels/document.xml.rels`
/// - `word/styles.xml` (if styles exist)
/// - `docProps/core.xml` (if metadata exists)
pub fn write(doc: &DocumentModel) -> Result<Vec<u8>, DocxError> {
    let buf = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(buf));
    let options = SimpleFileOptions::default();

    let has_styles = !doc.styles().is_empty();
    let core_xml = write_core_xml(doc);
    let has_core = core_xml.is_some();

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types_xml(has_styles, has_core).as_bytes())?;

    // _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels_xml(has_core).as_bytes())?;

    // word/_rels/document.xml.rels
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(document_rels_xml(has_styles).as_bytes())?;

    // word/document.xml
    zip.start_file("word/document.xml", options)?;
    let doc_xml = write_document_xml(doc);
    zip.write_all(doc_xml.as_bytes())?;

    // word/styles.xml (optional)
    if has_styles {
        zip.start_file("word/styles.xml", options)?;
        let styles_xml = write_styles_xml(doc);
        zip.write_all(styles_xml.as_bytes())?;
    }

    // docProps/core.xml (optional)
    if let Some(ref core) = core_xml {
        zip.start_file("docProps/core.xml", options)?;
        zip.write_all(core.as_bytes())?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

/// Generate `[Content_Types].xml`.
fn content_types_xml(has_styles: bool, has_core: bool) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
    );

    if has_styles {
        xml.push_str(
            r#"
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#,
        );
    }

    if has_core {
        xml.push_str(
            r#"
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#,
        );
    }

    xml.push_str("\n</Types>");
    xml
}

/// Generate `_rels/.rels`.
fn rels_xml(has_core: bool) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>"#,
    );

    if has_core {
        xml.push_str(
            r#"
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>"#,
        );
    }

    xml.push_str("\n</Relationships>");
    xml
}

/// Generate `word/_rels/document.xml.rels`.
fn document_rels_xml(has_styles: bool) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );

    if has_styles {
        xml.push_str(
            r#"
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
        );
    }

    xml.push_str("\n</Relationships>");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::{AttributeMap, Node, NodeType, Style, StyleType};

    fn make_simple_doc(text: &str) -> DocumentModel {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

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
    fn write_and_read_roundtrip_text() {
        let doc = make_simple_doc("Hello World");
        let bytes = write(&doc).unwrap();

        let doc2 = crate::read(&bytes).unwrap();
        assert_eq!(doc2.to_plain_text(), "Hello World");
    }

    #[test]
    fn write_and_read_roundtrip_formatting() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let run_id = doc.next_id();
        let mut run = Node::new(run_id, NodeType::Run);
        run.attributes = AttributeMap::new().bold(true).font_size(24.0);
        doc.insert_node(para_id, 0, run).unwrap();

        let text_id = doc.next_id();
        doc.insert_node(run_id, 0, Node::text(text_id, "Bold Title"))
            .unwrap();

        let bytes = write(&doc).unwrap();
        let doc2 = crate::read(&bytes).unwrap();

        assert_eq!(doc2.to_plain_text(), "Bold Title");

        let body_id2 = doc2.body_id().unwrap();
        let body2 = doc2.node(body_id2).unwrap();
        let para2 = doc2.node(body2.children[0]).unwrap();
        let run2 = doc2.node(para2.children[0]).unwrap();

        use s1_model::AttributeKey;
        assert_eq!(run2.attributes.get_bool(&AttributeKey::Bold), Some(true));
        assert_eq!(run2.attributes.get_f64(&AttributeKey::FontSize), Some(24.0));
    }

    #[test]
    fn write_and_read_roundtrip_styles() {
        let mut doc = make_simple_doc("Styled");

        let mut style = Style::new("Heading1", "Heading 1", StyleType::Paragraph);
        style.attributes = AttributeMap::new().bold(true).font_size(24.0);
        doc.set_style(style);

        let bytes = write(&doc).unwrap();
        let doc2 = crate::read(&bytes).unwrap();

        assert_eq!(doc2.to_plain_text(), "Styled");
        let s = doc2.style_by_id("Heading1").unwrap();
        assert_eq!(s.name, "Heading 1");
        use s1_model::AttributeKey;
        assert_eq!(s.attributes.get_bool(&AttributeKey::Bold), Some(true));
    }

    #[test]
    fn write_and_read_roundtrip_metadata() {
        let mut doc = make_simple_doc("Content");
        doc.metadata_mut().title = Some("My Document".to_string());
        doc.metadata_mut().creator = Some("Test Author".to_string());

        let bytes = write(&doc).unwrap();
        let doc2 = crate::read(&bytes).unwrap();

        assert_eq!(doc2.metadata().title.as_deref(), Some("My Document"));
        assert_eq!(doc2.metadata().creator.as_deref(), Some("Test Author"));
    }

    #[test]
    fn write_produces_valid_zip() {
        let doc = make_simple_doc("Test");
        let bytes = write(&doc).unwrap();

        // Should be readable as a ZIP
        let cursor = Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(cursor).unwrap();

        let names: Vec<&str> = archive.file_names().collect();
        assert!(names.contains(&"[Content_Types].xml"));
        assert!(names.contains(&"_rels/.rels"));
        assert!(names.contains(&"word/document.xml"));
        assert!(names.contains(&"word/_rels/document.xml.rels"));
    }

    #[test]
    fn write_multiple_paragraphs_roundtrip() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        for (i, text) in ["First", "Second", "Third"].into_iter().enumerate() {
            let para_id = doc.next_id();
            doc.insert_node(body_id, i, Node::new(para_id, NodeType::Paragraph))
                .unwrap();

            let run_id = doc.next_id();
            doc.insert_node(para_id, 0, Node::new(run_id, NodeType::Run))
                .unwrap();

            let text_id = doc.next_id();
            doc.insert_node(run_id, 0, Node::text(text_id, text))
                .unwrap();
        }

        let bytes = write(&doc).unwrap();
        let doc2 = crate::read(&bytes).unwrap();
        assert_eq!(doc2.to_plain_text(), "First\nSecond\nThird");
    }
}
