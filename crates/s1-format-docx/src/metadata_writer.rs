//! Generate `docProps/core.xml` from a DocumentModel.

use s1_model::DocumentModel;

use crate::xml_writer::escape_xml;

/// Generate `docProps/core.xml` content.
///
/// Returns `None` if metadata is completely empty.
pub fn write_core_xml(doc: &DocumentModel) -> Option<String> {
    let meta = doc.metadata();

    // Check if there's any metadata worth writing
    let has_content = meta.title.is_some()
        || meta.subject.is_some()
        || meta.creator.is_some()
        || meta.description.is_some()
        || !meta.keywords.is_empty()
        || meta.revision.is_some()
        || meta.created.is_some()
        || meta.modified.is_some()
        || meta.language.is_some();

    if !has_content {
        return None;
    }

    let mut xml = String::new();

    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push('\n');
    xml.push_str(
        r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
    );

    if let Some(ref title) = meta.title {
        xml.push_str(&format!("<dc:title>{}</dc:title>", escape_xml(title)));
    }

    if let Some(ref subject) = meta.subject {
        xml.push_str(&format!("<dc:subject>{}</dc:subject>", escape_xml(subject)));
    }

    if let Some(ref creator) = meta.creator {
        xml.push_str(&format!("<dc:creator>{}</dc:creator>", escape_xml(creator)));
    }

    if let Some(ref desc) = meta.description {
        xml.push_str(&format!(
            "<dc:description>{}</dc:description>",
            escape_xml(desc)
        ));
    }

    if !meta.keywords.is_empty() {
        let joined = meta.keywords.join(", ");
        xml.push_str(&format!(
            "<cp:keywords>{}</cp:keywords>",
            escape_xml(&joined)
        ));
    }

    if let Some(rev) = meta.revision {
        xml.push_str(&format!("<cp:revision>{rev}</cp:revision>"));
    }

    if let Some(ref created) = meta.created {
        xml.push_str(&format!(
            r#"<dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>"#,
            escape_xml(created)
        ));
    }

    if let Some(ref modified) = meta.modified {
        xml.push_str(&format!(
            r#"<dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>"#,
            escape_xml(modified)
        ));
    }

    if let Some(ref lang) = meta.language {
        xml.push_str(&format!("<dc:language>{}</dc:language>", escape_xml(lang)));
    }

    xml.push_str("</cp:coreProperties>");

    Some(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_empty_metadata() {
        let doc = DocumentModel::new();
        assert!(write_core_xml(&doc).is_none());
    }

    #[test]
    fn write_basic_metadata() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().title = Some("My Document".to_string());
        doc.metadata_mut().creator = Some("Test Author".to_string());

        let xml = write_core_xml(&doc).unwrap();
        assert!(xml.contains("<dc:title>My Document</dc:title>"));
        assert!(xml.contains("<dc:creator>Test Author</dc:creator>"));
    }

    #[test]
    fn write_keywords() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().keywords = vec![
            "test".to_string(),
            "document".to_string(),
            "sample".to_string(),
        ];

        let xml = write_core_xml(&doc).unwrap();
        assert!(xml.contains("<cp:keywords>test, document, sample</cp:keywords>"));
    }

    #[test]
    fn write_revision_and_dates() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().revision = Some(5);
        doc.metadata_mut().created = Some("2024-01-15T10:00:00Z".to_string());
        doc.metadata_mut().modified = Some("2024-01-16T12:00:00Z".to_string());

        let xml = write_core_xml(&doc).unwrap();
        assert!(xml.contains("<cp:revision>5</cp:revision>"));
        assert!(xml.contains("2024-01-15T10:00:00Z"));
        assert!(xml.contains("2024-01-16T12:00:00Z"));
    }

    #[test]
    fn write_escapes_metadata() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().title = Some("A & B".to_string());

        let xml = write_core_xml(&doc).unwrap();
        assert!(xml.contains("<dc:title>A &amp; B</dc:title>"));
    }
}
