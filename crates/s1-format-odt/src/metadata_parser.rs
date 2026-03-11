//! Parse ODF `meta.xml` into document metadata.

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::DocumentModel;

use crate::error::OdtError;

/// Parse `meta.xml` and populate `doc.metadata_mut()`.
pub fn parse_meta_xml(xml: &str, doc: &mut DocumentModel) -> Result<(), OdtError> {
    let mut reader = Reader::from_str(xml);
    let mut current_tag: Option<String> = None;
    let mut buf_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                let name = std::str::from_utf8(local.as_ref())
                    .unwrap_or("")
                    .to_string();
                current_tag = Some(name);
                buf_text.clear();
            }
            Ok(Event::Text(e)) => {
                if current_tag.is_some() {
                    if let Ok(t) = e.unescape() {
                        buf_text.push_str(&t);
                    }
                }
            }
            Ok(Event::End(_)) => {
                if let Some(ref tag) = current_tag {
                    let val = buf_text.trim().to_string();
                    if !val.is_empty() {
                        let meta = doc.metadata_mut();
                        match tag.as_str() {
                            "title" => meta.title = Some(val),
                            "subject" => meta.subject = Some(val),
                            "description" => meta.description = Some(val),
                            "initial-creator" | "creator" => {
                                if meta.creator.is_none() {
                                    meta.creator = Some(val);
                                }
                            }
                            "keyword" => {
                                meta.keywords.push(val);
                            }
                            "creation-date" => meta.created = Some(val),
                            "date" => meta.modified = Some(val),
                            "editing-cycles" => {
                                meta.revision = val.parse::<u32>().ok();
                            }
                            "language" => meta.language = Some(val),
                            _ => {}
                        }
                    }
                }
                current_tag = None;
                buf_text.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
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
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                      xmlns:dc="http://purl.org/dc/elements/1.1/"
                      xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">
  <office:meta>
    <dc:title>My Document</dc:title>
    <meta:initial-creator>Test Author</meta:initial-creator>
    <dc:subject>Test Subject</dc:subject>
  </office:meta>
</office:document-meta>"#;

        let mut doc = DocumentModel::new();
        parse_meta_xml(xml, &mut doc).unwrap();
        assert_eq!(doc.metadata().title.as_deref(), Some("My Document"));
        assert_eq!(doc.metadata().creator.as_deref(), Some("Test Author"));
        assert_eq!(doc.metadata().subject.as_deref(), Some("Test Subject"));
    }

    #[test]
    fn parse_keywords() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                      xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">
  <office:meta>
    <meta:keyword>rust</meta:keyword>
    <meta:keyword>document</meta:keyword>
  </office:meta>
</office:document-meta>"#;

        let mut doc = DocumentModel::new();
        parse_meta_xml(xml, &mut doc).unwrap();
        assert_eq!(
            doc.metadata().keywords,
            vec!["rust".to_string(), "document".to_string()]
        );
    }

    #[test]
    fn parse_revision_and_dates() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                      xmlns:dc="http://purl.org/dc/elements/1.1/"
                      xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">
  <office:meta>
    <meta:creation-date>2026-03-11T00:00:00Z</meta:creation-date>
    <dc:date>2026-03-12T12:00:00Z</dc:date>
    <meta:editing-cycles>5</meta:editing-cycles>
  </office:meta>
</office:document-meta>"#;

        let mut doc = DocumentModel::new();
        parse_meta_xml(xml, &mut doc).unwrap();
        assert_eq!(
            doc.metadata().created.as_deref(),
            Some("2026-03-11T00:00:00Z")
        );
        assert_eq!(
            doc.metadata().modified.as_deref(),
            Some("2026-03-12T12:00:00Z")
        );
        assert_eq!(doc.metadata().revision, Some(5));
    }

    #[test]
    fn parse_empty_metadata() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0">
  <office:meta/>
</office:document-meta>"#;

        let mut doc = DocumentModel::new();
        parse_meta_xml(xml, &mut doc).unwrap();
        assert!(doc.metadata().title.is_none());
    }
}
