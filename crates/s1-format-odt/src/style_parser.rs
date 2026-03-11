//! Parse ODF styles (named styles from `styles.xml`, automatic styles from `content.xml`).

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{AttributeMap, DocumentModel, Style, StyleType};

use crate::error::OdtError;
use crate::property_parser::{parse_paragraph_properties, parse_text_properties};
use crate::xml_util::get_attr;

/// Parse `styles.xml` and populate `doc` with named styles.
pub fn parse_styles_xml(xml: &str, doc: &mut DocumentModel) -> Result<(), OdtError> {
    let mut reader = Reader::from_str(xml);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"style" => {
                if let Some(style) = parse_style_element(&mut reader, &e)? {
                    doc.set_style(style);
                }
            }
            Ok(Event::Empty(e)) if e.local_name().as_ref() == b"style" => {
                // Self-closing style with no child properties
                if let Some(style) = parse_empty_style(&e) {
                    doc.set_style(style);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }
    Ok(())
}

/// Parse automatic styles from a reader positioned inside `<office:automatic-styles>`.
///
/// Returns a map of style name → merged attributes (paragraph + text props combined).
pub fn parse_automatic_styles(
    reader: &mut Reader<&[u8]>,
) -> Result<HashMap<String, AttributeMap>, OdtError> {
    let mut auto_styles: HashMap<String, AttributeMap> = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"style" => {
                let name = get_attr(e, b"name").unwrap_or_default();
                let family = get_attr(e, b"family").unwrap_or_default();
                let parent = get_attr(e, b"parent-style-name");

                let mut attrs = AttributeMap::new();

                // Record parent style reference
                if let Some(ref parent_name) = parent {
                    attrs.set(
                        s1_model::AttributeKey::StyleId,
                        s1_model::AttributeValue::String(parent_name.clone()),
                    );
                }

                // Parse child property elements
                loop {
                    match reader.read_event() {
                        Ok(Event::Start(ref pe)) | Ok(Event::Empty(ref pe)) => {
                            let local = pe.local_name();
                            match local.as_ref() {
                                b"text-properties" => {
                                    attrs.merge(&parse_text_properties(pe));
                                }
                                b"paragraph-properties" => {
                                    attrs.merge(&parse_paragraph_properties(pe));
                                }
                                _ => {}
                            }
                        }
                        Ok(Event::End(ref ee)) if ee.local_name().as_ref() == b"style" => {
                            break;
                        }
                        Ok(Event::Eof) => break,
                        Err(e) => return Err(OdtError::Xml(e.to_string())),
                        _ => {}
                    }
                }

                if !name.is_empty() {
                    // Store family info for distinguishing paragraph vs text auto-styles
                    if family == "text" {
                        // Mark as text/character style (no paragraph-level props expected)
                    }
                    auto_styles.insert(name, attrs);
                }
            }
            Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"style" => {
                let name = get_attr(e, b"name").unwrap_or_default();
                let parent = get_attr(e, b"parent-style-name");

                let mut attrs = AttributeMap::new();
                if let Some(ref parent_name) = parent {
                    attrs.set(
                        s1_model::AttributeKey::StyleId,
                        s1_model::AttributeValue::String(parent_name.clone()),
                    );
                }
                if !name.is_empty() {
                    auto_styles.insert(name, attrs);
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"automatic-styles" => {
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(auto_styles)
}

/// Parse a `<style:style>` element (non-empty) into a `Style`.
fn parse_style_element(
    reader: &mut Reader<&[u8]>,
    e: &quick_xml::events::BytesStart<'_>,
) -> Result<Option<Style>, OdtError> {
    let name = match get_attr(e, b"name") {
        Some(n) => n,
        None => {
            skip_to_end(reader, b"style")?;
            return Ok(None);
        }
    };

    let display_name = get_attr(e, b"display-name").unwrap_or_else(|| name.clone());
    let family = get_attr(e, b"family").unwrap_or_default();
    let parent_name = get_attr(e, b"parent-style-name");

    let style_type = match family.as_str() {
        "paragraph" => StyleType::Paragraph,
        "text" => StyleType::Character,
        "table" => StyleType::Table,
        "list" => StyleType::List,
        _ => StyleType::Paragraph,
    };

    let mut style = Style::new(&name, &display_name, style_type);
    if let Some(parent) = parent_name {
        style = style.with_parent(parent);
    }

    let mut attrs = AttributeMap::new();

    // Parse child property elements
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref pe)) | Ok(Event::Empty(ref pe)) => {
                let local = pe.local_name();
                match local.as_ref() {
                    b"text-properties" => attrs.merge(&parse_text_properties(pe)),
                    b"paragraph-properties" => attrs.merge(&parse_paragraph_properties(pe)),
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) if ee.local_name().as_ref() == b"style" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    if !attrs.is_empty() {
        style = style.with_attributes(attrs);
    }

    Ok(Some(style))
}

/// Parse a self-closing `<style:style ... />` into a `Style` (no child properties).
fn parse_empty_style(e: &quick_xml::events::BytesStart<'_>) -> Option<Style> {
    let name = get_attr(e, b"name")?;
    let display_name = get_attr(e, b"display-name").unwrap_or_else(|| name.clone());
    let family = get_attr(e, b"family").unwrap_or_default();
    let parent_name = get_attr(e, b"parent-style-name");

    let style_type = match family.as_str() {
        "paragraph" => StyleType::Paragraph,
        "text" => StyleType::Character,
        "table" => StyleType::Table,
        "list" => StyleType::List,
        _ => StyleType::Paragraph,
    };

    let mut style = Style::new(&name, &display_name, style_type);
    if let Some(parent) = parent_name {
        style = style.with_parent(parent);
    }
    Some(style)
}

/// Skip the reader past the end of an element.
fn skip_to_end(reader: &mut Reader<&[u8]>, tag: &[u8]) -> Result<(), OdtError> {
    let mut depth = 1u32;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == tag => depth += 1,
            Ok(Event::End(e)) if e.local_name().as_ref() == tag => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            Ok(Event::Eof) => return Ok(()),
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::AttributeKey;

    #[test]
    fn parse_named_style_paragraph() {
        let xml = r#"<?xml version="1.0"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0">
<office:styles>
  <style:style style:name="Heading1" style:display-name="Heading 1" style:family="paragraph">
    <style:text-properties fo:font-weight="bold" fo:font-size="24pt"/>
    <style:paragraph-properties fo:text-align="center"/>
  </style:style>
</office:styles>
</office:document-styles>"#;

        let mut doc = DocumentModel::new();
        parse_styles_xml(xml, &mut doc).unwrap();

        let style = doc.style_by_id("Heading1").unwrap();
        assert_eq!(style.name, "Heading 1");
        assert_eq!(style.style_type, StyleType::Paragraph);
        assert_eq!(style.attributes.get_bool(&AttributeKey::Bold), Some(true));
        assert_eq!(
            style.attributes.get_f64(&AttributeKey::FontSize),
            Some(24.0)
        );
    }

    #[test]
    fn parse_style_with_parent() {
        let xml = r#"<?xml version="1.0"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0">
<office:styles>
  <style:style style:name="TextBody" style:family="paragraph"/>
  <style:style style:name="MyStyle" style:family="paragraph" style:parent-style-name="TextBody">
    <style:text-properties fo:font-style="italic"/>
  </style:style>
</office:styles>
</office:document-styles>"#;

        let mut doc = DocumentModel::new();
        parse_styles_xml(xml, &mut doc).unwrap();

        let style = doc.style_by_id("MyStyle").unwrap();
        assert_eq!(style.parent_id.as_deref(), Some("TextBody"));
        assert_eq!(style.attributes.get_bool(&AttributeKey::Italic), Some(true));
    }

    #[test]
    fn parse_auto_styles() {
        let xml = br#"<office:automatic-styles>
  <style:style style:name="P1" style:family="paragraph" style:parent-style-name="Standard">
    <style:paragraph-properties fo:text-align="center"/>
  </style:style>
  <style:style style:name="T1" style:family="text">
    <style:text-properties fo:font-weight="bold"/>
  </style:style>
</office:automatic-styles>"#;

        let mut reader = Reader::from_reader(xml.as_ref());
        // Advance past the opening tag
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"automatic-styles" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }

        let auto = parse_automatic_styles(&mut reader).unwrap();
        assert_eq!(auto.len(), 2);

        let p1 = auto.get("P1").unwrap();
        assert_eq!(p1.get_string(&AttributeKey::StyleId), Some("Standard"));

        let t1 = auto.get("T1").unwrap();
        assert_eq!(t1.get_bool(&AttributeKey::Bold), Some(true));
    }

    #[test]
    fn parse_empty_style_element() {
        let xml = r#"<?xml version="1.0"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0">
<office:styles>
  <style:style style:name="Default" style:family="paragraph"/>
</office:styles>
</office:document-styles>"#;

        let mut doc = DocumentModel::new();
        parse_styles_xml(xml, &mut doc).unwrap();

        let style = doc.style_by_id("Default").unwrap();
        assert_eq!(style.style_type, StyleType::Paragraph);
        assert!(style.attributes.is_empty());
    }
}
