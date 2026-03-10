//! DOCX reader — main entry point.
//!
//! Reads a DOCX file (ZIP archive) and produces a [`DocumentModel`].

use std::io::{Cursor, Read};

use s1_model::DocumentModel;
use zip::ZipArchive;

use crate::content_parser::parse_document_xml;
use crate::error::DocxError;
use crate::metadata_parser::parse_core_xml;
use crate::style_parser::parse_styles_xml;

/// Read a DOCX file from bytes and produce a [`DocumentModel`].
///
/// The DOCX format is a ZIP archive containing XML files.
/// This reader handles:
/// - `word/document.xml` — main document content (paragraphs, runs, text, formatting)
/// - `word/styles.xml` — style definitions
/// - `docProps/core.xml` — document metadata
pub fn read(input: &[u8]) -> Result<DocumentModel, DocxError> {
    let cursor = Cursor::new(input);
    let mut archive = ZipArchive::new(cursor)?;

    let mut doc = DocumentModel::new();

    // Parse styles first (needed for style references in document.xml)
    if let Ok(styles_xml) = read_zip_entry(&mut archive, "word/styles.xml") {
        parse_styles_xml(&styles_xml, &mut doc)?;
    }

    // Parse main document content
    let doc_xml = read_zip_entry(&mut archive, "word/document.xml")?;
    parse_document_xml(&doc_xml, &mut doc)?;

    // Parse metadata
    if let Ok(core_xml) = read_zip_entry(&mut archive, "docProps/core.xml") {
        parse_core_xml(&core_xml, &mut doc)?;
    }

    Ok(doc)
}

/// Read a file from the ZIP archive as a UTF-8 string.
fn read_zip_entry(archive: &mut ZipArchive<Cursor<&[u8]>>, path: &str) -> Result<String, DocxError> {
    let mut file = archive
        .by_name(path)
        .map_err(|_| DocxError::MissingFile(path.to_string()))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::AttributeKey;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    /// Build a minimal DOCX file as bytes.
    fn make_docx(doc_xml: &str, styles_xml: Option<&str>, core_xml: Option<&str>) -> Vec<u8> {
        let buf = Vec::new();
        let mut zip = ZipWriter::new(Cursor::new(buf));
        let options = SimpleFileOptions::default();

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        )
        .unwrap();

        // _rels/.rels
        zip.start_file("_rels/.rels", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
        )
        .unwrap();

        // word/document.xml
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(doc_xml.as_bytes()).unwrap();

        // Optional: word/styles.xml
        if let Some(styles) = styles_xml {
            zip.start_file("word/styles.xml", options).unwrap();
            zip.write_all(styles.as_bytes()).unwrap();
        }

        // Optional: docProps/core.xml
        if let Some(core) = core_xml {
            zip.start_file("docProps/core.xml", options).unwrap();
            zip.write_all(core.as_bytes()).unwrap();
        }

        zip.finish().unwrap().into_inner()
    }

    fn simple_doc_xml(body_content: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>{body_content}</w:body>
</w:document>"#
        )
    }

    #[test]
    fn read_minimal_docx() {
        let doc_xml = simple_doc_xml(r#"<w:p><w:r><w:t>Hello World</w:t></w:r></w:p>"#);
        let docx = make_docx(&doc_xml, None, None);

        let doc = read(&docx).unwrap();
        assert_eq!(doc.to_plain_text(), "Hello World");
    }

    #[test]
    fn read_multiple_paragraphs() {
        let doc_xml = simple_doc_xml(
            r#"<w:p><w:r><w:t>First paragraph</w:t></w:r></w:p>
            <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
            <w:p><w:r><w:t>Third paragraph</w:t></w:r></w:p>"#,
        );
        let docx = make_docx(&doc_xml, None, None);

        let doc = read(&docx).unwrap();
        assert_eq!(
            doc.to_plain_text(),
            "First paragraph\nSecond paragraph\nThird paragraph"
        );
    }

    #[test]
    fn read_with_formatting() {
        let doc_xml = simple_doc_xml(
            r#"<w:p>
            <w:r><w:rPr><w:b/><w:sz w:val="48"/></w:rPr><w:t>Bold Title</w:t></w:r>
            </w:p>"#,
        );
        let docx = make_docx(&doc_xml, None, None);

        let doc = read(&docx).unwrap();
        assert_eq!(doc.to_plain_text(), "Bold Title");

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();
        let run = doc.node(para.children[0]).unwrap();

        assert_eq!(run.attributes.get_bool(&AttributeKey::Bold), Some(true));
        assert_eq!(run.attributes.get_f64(&AttributeKey::FontSize), Some(24.0)); // 48 half-pts
    }

    #[test]
    fn read_with_styles() {
        let doc_xml = simple_doc_xml(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#,
        );
        let styles_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="Heading 1"/>
    <w:rPr><w:b/><w:sz w:val="48"/></w:rPr>
  </w:style>
</w:styles>"#;
        let docx = make_docx(&doc_xml, Some(styles_xml), None);

        let doc = read(&docx).unwrap();
        assert_eq!(doc.to_plain_text(), "Title");

        // Style should be loaded
        let style = doc.style_by_id("Heading1").unwrap();
        assert_eq!(style.name, "Heading 1");
        assert_eq!(
            style.attributes.get_bool(&AttributeKey::Bold),
            Some(true)
        );
    }

    #[test]
    fn read_with_metadata() {
        let doc_xml = simple_doc_xml(r#"<w:p><w:r><w:t>Content</w:t></w:r></w:p>"#);
        let core_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>My Document</dc:title>
  <dc:creator>Test Author</dc:creator>
</cp:coreProperties>"#;
        let docx = make_docx(&doc_xml, None, Some(core_xml));

        let doc = read(&docx).unwrap();
        assert_eq!(doc.metadata().title.as_deref(), Some("My Document"));
        assert_eq!(doc.metadata().creator.as_deref(), Some("Test Author"));
    }

    #[test]
    fn read_invalid_zip() {
        let result = read(b"not a zip file");
        assert!(result.is_err());
    }

    #[test]
    fn read_missing_document_xml() {
        // ZIP without word/document.xml
        let buf = Vec::new();
        let mut zip = ZipWriter::new(Cursor::new(buf));
        let options = SimpleFileOptions::default();
        zip.start_file("dummy.txt", options).unwrap();
        zip.write_all(b"dummy").unwrap();
        let bytes = zip.finish().unwrap().into_inner();

        let result = read(&bytes);
        assert!(result.is_err());
    }
}
