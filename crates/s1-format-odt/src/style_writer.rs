//! Write s1-model styles as ODF `styles.xml`.

use s1_model::{DocumentModel, StyleType};

use crate::property_writer::{write_paragraph_properties, write_text_properties};
use crate::xml_util::escape_xml;

/// Generate `styles.xml` content.
///
/// Returns `None` if the document has no named styles.
pub fn write_styles_xml(doc: &DocumentModel) -> Option<String> {
    let styles = doc.styles();
    if styles.is_empty() {
        return None;
    }

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
<office:styles>"#,
    );

    for style in styles {
        let family = match style.style_type {
            StyleType::Paragraph => "paragraph",
            StyleType::Character => "text",
            StyleType::Table => "table",
            StyleType::List => "list",
        };

        xml.push_str(&format!(
            r#"<style:style style:name="{}" style:display-name="{}" style:family="{}""#,
            escape_xml(&style.id),
            escape_xml(&style.name),
            family,
        ));

        if let Some(ref parent) = style.parent_id {
            xml.push_str(&format!(
                r#" style:parent-style-name="{}""#,
                escape_xml(parent)
            ));
        }

        let text_props = write_text_properties(&style.attributes);
        let para_props = write_paragraph_properties(&style.attributes);

        if text_props.is_empty() && para_props.is_empty() {
            xml.push_str("/>");
        } else {
            xml.push('>');
            if !para_props.is_empty() {
                xml.push_str(&para_props);
            }
            if !text_props.is_empty() {
                xml.push_str(&text_props);
            }
            xml.push_str("</style:style>");
        }
    }

    xml.push_str("</office:styles></office:document-styles>");
    Some(xml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::{AttributeMap, Style, StyleType};

    #[test]
    fn write_no_styles() {
        let doc = DocumentModel::new();
        assert!(write_styles_xml(&doc).is_none());
    }

    #[test]
    fn write_paragraph_style() {
        let mut doc = DocumentModel::new();
        let attrs = AttributeMap::new().bold(true).font_size(24.0);
        let style =
            Style::new("Heading1", "Heading 1", StyleType::Paragraph).with_attributes(attrs);
        doc.set_style(style);

        let xml = write_styles_xml(&doc).unwrap();
        assert!(xml.contains(r#"style:name="Heading1""#));
        assert!(xml.contains(r#"style:display-name="Heading 1""#));
        assert!(xml.contains(r#"style:family="paragraph""#));
        assert!(xml.contains(r#"fo:font-weight="bold""#));
        assert!(xml.contains(r#"fo:font-size="24pt""#));
    }

    #[test]
    fn write_style_with_parent() {
        let mut doc = DocumentModel::new();
        let style = Style::new("Child", "Child", StyleType::Paragraph).with_parent("Parent");
        doc.set_style(style);

        let xml = write_styles_xml(&doc).unwrap();
        assert!(xml.contains(r#"style:parent-style-name="Parent""#));
    }

    #[test]
    fn write_character_style() {
        let mut doc = DocumentModel::new();
        let attrs = AttributeMap::new().italic(true);
        let style = Style::new("Emphasis", "Emphasis", StyleType::Character).with_attributes(attrs);
        doc.set_style(style);

        let xml = write_styles_xml(&doc).unwrap();
        assert!(xml.contains(r#"style:family="text""#));
        assert!(xml.contains(r#"fo:font-style="italic""#));
    }
}
