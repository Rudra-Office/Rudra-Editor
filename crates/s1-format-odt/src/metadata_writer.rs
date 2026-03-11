//! Write ODF `meta.xml` from document metadata.

use s1_model::DocumentModel;

use crate::xml_util::escape_xml;

/// Generate `meta.xml` content, or `None` if metadata is all empty.
pub fn write_meta_xml(doc: &DocumentModel) -> Option<String> {
    let meta = doc.metadata();

    let has_data = meta.title.is_some()
        || meta.creator.is_some()
        || meta.subject.is_some()
        || meta.description.is_some()
        || !meta.keywords.is_empty()
        || meta.created.is_some()
        || meta.modified.is_some()
        || meta.revision.is_some()
        || meta.language.is_some();

    if !has_data {
        return None;
    }

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">
<office:meta>"#,
    );

    if let Some(ref title) = meta.title {
        xml.push_str(&format!("<dc:title>{}</dc:title>", escape_xml(title)));
    }
    if let Some(ref subject) = meta.subject {
        xml.push_str(&format!("<dc:subject>{}</dc:subject>", escape_xml(subject)));
    }
    if let Some(ref desc) = meta.description {
        xml.push_str(&format!(
            "<dc:description>{}</dc:description>",
            escape_xml(desc)
        ));
    }
    if let Some(ref creator) = meta.creator {
        xml.push_str(&format!(
            "<meta:initial-creator>{}</meta:initial-creator>",
            escape_xml(creator)
        ));
    }
    for kw in &meta.keywords {
        if !kw.is_empty() {
            xml.push_str(&format!("<meta:keyword>{}</meta:keyword>", escape_xml(kw)));
        }
    }
    if let Some(ref created) = meta.created {
        xml.push_str(&format!(
            "<meta:creation-date>{}</meta:creation-date>",
            escape_xml(created)
        ));
    }
    if let Some(ref modified) = meta.modified {
        xml.push_str(&format!("<dc:date>{}</dc:date>", escape_xml(modified)));
    }
    if let Some(rev) = meta.revision {
        xml.push_str(&format!("<meta:editing-cycles>{rev}</meta:editing-cycles>"));
    }
    if let Some(ref lang) = meta.language {
        xml.push_str(&format!("<dc:language>{}</dc:language>", escape_xml(lang)));
    }

    xml.push_str("</office:meta></office:document-meta>");
    Some(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_empty_metadata() {
        let doc = DocumentModel::new();
        assert!(write_meta_xml(&doc).is_none());
    }

    #[test]
    fn write_basic_metadata() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().title = Some("Test".to_string());
        doc.metadata_mut().creator = Some("Author".to_string());

        let xml = write_meta_xml(&doc).unwrap();
        assert!(xml.contains("<dc:title>Test</dc:title>"));
        assert!(xml.contains("<meta:initial-creator>Author</meta:initial-creator>"));
    }

    #[test]
    fn write_keywords() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().keywords = vec!["rust".to_string(), "document".to_string()];

        let xml = write_meta_xml(&doc).unwrap();
        assert!(xml.contains("<meta:keyword>rust</meta:keyword>"));
        assert!(xml.contains("<meta:keyword>document</meta:keyword>"));
    }
}
