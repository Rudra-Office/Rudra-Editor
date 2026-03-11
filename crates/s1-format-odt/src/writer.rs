//! ODT writer — serialize a `DocumentModel` into an ODT ZIP archive.

use std::io::{Cursor, Write as IoWrite};

use s1_model::DocumentModel;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::content_writer::write_content_xml;
use crate::error::OdtError;
use crate::manifest_writer::write_manifest_xml;
use crate::metadata_writer::write_meta_xml;
use crate::style_writer::write_styles_xml;

/// Write a `DocumentModel` as ODT bytes.
///
/// # Errors
///
/// Returns `OdtError` if ZIP writing fails.
pub fn write(doc: &DocumentModel) -> Result<Vec<u8>, OdtError> {
    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);
    let mut zip = ZipWriter::new(cursor);

    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let deflated =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. mimetype — MUST be first entry, stored (uncompressed), no extra fields
    zip.start_file("mimetype", stored)?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")?;

    // 2. Generate content.xml + collect image entries
    let (content_xml, image_entries) = write_content_xml(doc);

    // 3. Generate styles.xml (optional)
    let styles_xml = write_styles_xml(doc);

    // 4. Generate meta.xml (optional)
    let meta_xml = write_meta_xml(doc);

    // 5. Write content.xml
    zip.start_file("content.xml", deflated)?;
    zip.write_all(content_xml.as_bytes())?;

    // 6. Write styles.xml
    if let Some(ref styles) = styles_xml {
        zip.start_file("styles.xml", deflated)?;
        zip.write_all(styles.as_bytes())?;
    }

    // 7. Write meta.xml
    if let Some(ref meta) = meta_xml {
        zip.start_file("meta.xml", deflated)?;
        zip.write_all(meta.as_bytes())?;
    }

    // 8. Write images to Pictures/
    for entry in &image_entries {
        if let Some(media) = doc.media().get(entry.media_id) {
            zip.start_file(&entry.href, deflated)?;
            zip.write_all(&media.data)?;
        }
    }

    // 9. Write META-INF/manifest.xml
    let image_paths: Vec<&str> = image_entries.iter().map(|e| e.href.as_str()).collect();
    let manifest = write_manifest_xml(&image_paths);
    zip.start_file("META-INF/manifest.xml", deflated)?;
    zip.write_all(manifest.as_bytes())?;

    zip.finish()?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::{AttributeMap, Node, NodeType, Style, StyleType};

    #[test]
    fn write_minimal_odt() {
        let doc = DocumentModel::new();
        let bytes = write(&doc).unwrap();

        // Verify it's a valid ZIP
        let cursor = Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();

        // Check mimetype
        let mut mimetype = String::new();
        archive
            .by_name("mimetype")
            .unwrap()
            .read_to_string(&mut mimetype)
            .unwrap();
        assert_eq!(mimetype, "application/vnd.oasis.opendocument.text");

        // Check content.xml exists
        assert!(archive.by_name("content.xml").is_ok());
        assert!(archive.by_name("META-INF/manifest.xml").is_ok());
    }

    #[test]
    fn write_with_content() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        let para = Node::new(para_id, NodeType::Paragraph);
        doc.insert_node(body_id, 0, para).unwrap();

        let run_id = doc.next_id();
        let run = Node::new(run_id, NodeType::Run);
        doc.insert_node(para_id, 0, run).unwrap();

        let text_id = doc.next_id();
        let text = Node::text(text_id, "Hello ODT");
        doc.insert_node(run_id, 0, text).unwrap();

        let bytes = write(&doc).unwrap();

        // Verify content.xml contains our text
        let cursor = Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut content = String::new();
        archive
            .by_name("content.xml")
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert!(content.contains("Hello ODT"));
    }

    #[test]
    fn write_with_styles() {
        let mut doc = DocumentModel::new();
        let style = Style::new("Heading1", "Heading 1", StyleType::Paragraph)
            .with_attributes(AttributeMap::new().bold(true));
        doc.set_style(style);

        let bytes = write(&doc).unwrap();
        let cursor = Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();

        // styles.xml should exist
        assert!(archive.by_name("styles.xml").is_ok());

        let mut styles = String::new();
        archive
            .by_name("styles.xml")
            .unwrap()
            .read_to_string(&mut styles)
            .unwrap();
        assert!(styles.contains("Heading1"));
    }

    #[test]
    fn write_with_metadata() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().title = Some("Test Doc".to_string());
        doc.metadata_mut().creator = Some("Author".to_string());

        let bytes = write(&doc).unwrap();
        let cursor = Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();

        assert!(archive.by_name("meta.xml").is_ok());

        let mut meta = String::new();
        archive
            .by_name("meta.xml")
            .unwrap()
            .read_to_string(&mut meta)
            .unwrap();
        assert!(meta.contains("Test Doc"));
        assert!(meta.contains("Author"));
    }

    #[test]
    fn roundtrip_basic() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let run_id = doc.next_id();
        doc.insert_node(para_id, 0, Node::new(run_id, NodeType::Run))
            .unwrap();

        let text_id = doc.next_id();
        doc.insert_node(run_id, 0, Node::text(text_id, "Round trip test"))
            .unwrap();

        let bytes = write(&doc).unwrap();
        let doc2 = crate::reader::read(&bytes).unwrap();

        let body_id2 = doc2.body_id().unwrap();
        let body2 = doc2.node(body_id2).unwrap();
        assert_eq!(body2.children.len(), 1);

        // Verify text content is preserved
        let plain = doc2.to_plain_text();
        assert!(plain.contains("Round trip test"));
    }

    #[test]
    fn roundtrip_metadata() {
        let mut doc = DocumentModel::new();
        doc.metadata_mut().title = Some("My Title".to_string());
        doc.metadata_mut().creator = Some("Author Name".to_string());

        let bytes = write(&doc).unwrap();
        let doc2 = crate::reader::read(&bytes).unwrap();

        assert_eq!(doc2.metadata().title.as_deref(), Some("My Title"));
        assert_eq!(doc2.metadata().creator.as_deref(), Some("Author Name"));
    }

    #[test]
    fn roundtrip_styles() {
        let mut doc = DocumentModel::new();
        let style = Style::new("MyStyle", "My Style", StyleType::Paragraph)
            .with_attributes(AttributeMap::new().bold(true).font_size(16.0));
        doc.set_style(style);

        let bytes = write(&doc).unwrap();
        let doc2 = crate::reader::read(&bytes).unwrap();

        let s = doc2.style_by_id("MyStyle").unwrap();
        assert_eq!(s.name, "My Style");
        assert_eq!(
            s.attributes.get_bool(&s1_model::AttributeKey::Bold),
            Some(true)
        );
        assert_eq!(
            s.attributes.get_f64(&s1_model::AttributeKey::FontSize),
            Some(16.0)
        );
    }

    use std::io::Read as _;
}
