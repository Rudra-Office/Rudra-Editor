//! Parse `docProps/core.xml` — Dublin Core metadata.

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::DocumentModel;

use crate::error::DocxError;

/// Parse `docProps/core.xml` and set metadata on the document model.
pub fn parse_core_xml(xml: &str, doc: &mut DocumentModel) -> Result<(), DocxError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut current_tag: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                current_tag = Some(String::from_utf8_lossy(&name).to_string());
            }
            Ok(Event::Text(e)) => {
                if let Some(ref tag) = current_tag {
                    let text = e
                        .unescape()
                        .map_err(|e| DocxError::Xml(format!("{e}")))?
                        .to_string();

                    if !text.is_empty() {
                        let meta = doc.metadata_mut();
                        match tag.as_str() {
                            "title" => meta.title = Some(text),
                            "subject" => meta.subject = Some(text),
                            "creator" => meta.creator = Some(text),
                            "description" => meta.description = Some(text),
                            "keywords" => {
                                meta.keywords = text
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                            "revision" => {
                                meta.revision = text.parse().ok();
                            }
                            "created" => meta.created = Some(text),
                            "modified" => meta.modified = Some(text),
                            "language" => meta.language = Some(text),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
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
    fn parse_basic_metadata() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/"
                   xmlns:dcterms="http://purl.org/dc/terms/">
  <dc:title>Test Document</dc:title>
  <dc:creator>John Doe</dc:creator>
  <dc:subject>Testing</dc:subject>
  <dc:description>A test document</dc:description>
</cp:coreProperties>"#;

        let mut doc = DocumentModel::new();
        parse_core_xml(xml, &mut doc).unwrap();

        let meta = doc.metadata();
        assert_eq!(meta.title.as_deref(), Some("Test Document"));
        assert_eq!(meta.creator.as_deref(), Some("John Doe"));
        assert_eq!(meta.subject.as_deref(), Some("Testing"));
        assert_eq!(meta.description.as_deref(), Some("A test document"));
    }

    #[test]
    fn parse_keywords() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties">
  <cp:keywords>test, document, sample</cp:keywords>
</cp:coreProperties>"#;

        let mut doc = DocumentModel::new();
        parse_core_xml(xml, &mut doc).unwrap();

        let meta = doc.metadata();
        assert_eq!(meta.keywords, vec!["test", "document", "sample"]);
    }

    #[test]
    fn parse_revision() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties">
  <cp:revision>5</cp:revision>
</cp:coreProperties>"#;

        let mut doc = DocumentModel::new();
        parse_core_xml(xml, &mut doc).unwrap();

        assert_eq!(doc.metadata().revision, Some(5));
    }

    #[test]
    fn parse_empty_metadata() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties">
</cp:coreProperties>"#;

        let mut doc = DocumentModel::new();
        parse_core_xml(xml, &mut doc).unwrap();

        let meta = doc.metadata();
        assert!(meta.title.is_none());
        assert!(meta.creator.is_none());
    }
}
