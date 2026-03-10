//! Generate `word/document.xml` from a DocumentModel.

use s1_model::{
    Alignment, AttributeKey, AttributeValue, Color, DocumentModel, LineSpacing, NodeId, NodeType,
    UnderlineStyle,
};

use crate::xml_writer::escape_xml;

/// Generate `word/document.xml` content.
pub fn write_document_xml(doc: &DocumentModel) -> String {
    let mut xml = String::new();

    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push('\n');
    xml.push_str(
        r#"<w:document xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas" xmlns:mo="http://schemas.microsoft.com/office/mac/office/2008/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:mv="urn:schemas-microsoft-com:mac:vml" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:w10="urn:schemas-microsoft-com:office:word" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:wne="http://schemas.microsoft.com/office/word/2006/wordml">"#,
    );
    xml.push('\n');

    // Write body
    if let Some(body_id) = doc.body_id() {
        xml.push_str("<w:body>");
        write_body_children(doc, body_id, &mut xml);
        xml.push_str("</w:body>");
    }

    xml.push_str("</w:document>");
    xml
}

/// Write the children of the body node.
fn write_body_children(doc: &DocumentModel, body_id: NodeId, xml: &mut String) {
    let body = match doc.node(body_id) {
        Some(n) => n,
        None => return,
    };

    let children: Vec<NodeId> = body.children.clone();
    for child_id in children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };

        if child.node_type == NodeType::Paragraph {
            write_paragraph(doc, child_id, xml);
        }
    }
}

/// Write a `<w:p>` element.
fn write_paragraph(doc: &DocumentModel, para_id: NodeId, xml: &mut String) {
    let para = match doc.node(para_id) {
        Some(n) => n,
        None => return,
    };

    xml.push_str("<w:p>");

    // Paragraph properties
    let ppr = write_paragraph_properties(para);
    if !ppr.is_empty() {
        xml.push_str("<w:pPr>");
        xml.push_str(&ppr);
        xml.push_str("</w:pPr>");
    }

    // Inline children
    let children: Vec<NodeId> = para.children.clone();
    for child_id in children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };

        match child.node_type {
            NodeType::Run => write_run(doc, child_id, xml),
            NodeType::LineBreak => {
                // Wrap in a run
                xml.push_str("<w:r><w:br/></w:r>");
            }
            NodeType::PageBreak => {
                xml.push_str(r#"<w:r><w:br w:type="page"/></w:r>"#);
            }
            NodeType::ColumnBreak => {
                xml.push_str(r#"<w:r><w:br w:type="column"/></w:r>"#);
            }
            NodeType::Tab => {
                xml.push_str("<w:r><w:tab/></w:r>");
            }
            _ => {}
        }
    }

    xml.push_str("</w:p>");
}

/// Generate paragraph properties XML from a Node.
fn write_paragraph_properties(para: &s1_model::Node) -> String {
    write_paragraph_properties_from_attrs(&para.attributes)
}

/// Generate paragraph properties XML from an AttributeMap.
///
/// Public so the style writer can reuse it.
pub fn write_paragraph_properties_from_attrs(attrs: &s1_model::AttributeMap) -> String {
    let mut ppr = String::new();

    // Style reference
    if let Some(style_id) = attrs.get_string(&AttributeKey::StyleId) {
        ppr.push_str(&format!(
            r#"<w:pStyle w:val="{}"/>"#,
            escape_xml(style_id)
        ));
    }

    // Alignment
    if let Some(alignment) = attrs.get_alignment(&AttributeKey::Alignment) {
        let val = match alignment {
            Alignment::Left => "left",
            Alignment::Center => "center",
            Alignment::Right => "right",
            Alignment::Justify => "both",
        };
        ppr.push_str(&format!(r#"<w:jc w:val="{val}"/>"#));
    }

    // Spacing
    let before = attrs.get_f64(&AttributeKey::SpacingBefore);
    let after = attrs.get_f64(&AttributeKey::SpacingAfter);
    let line_spacing = attrs.get(&AttributeKey::LineSpacing);

    if before.is_some() || after.is_some() || line_spacing.is_some() {
        let mut spacing_attrs = String::new();
        if let Some(pts) = before {
            spacing_attrs.push_str(&format!(r#" w:before="{}""#, points_to_twips(pts)));
        }
        if let Some(pts) = after {
            spacing_attrs.push_str(&format!(r#" w:after="{}""#, points_to_twips(pts)));
        }
        if let Some(AttributeValue::LineSpacing(ls)) = line_spacing {
            match ls {
                LineSpacing::Single => {
                    spacing_attrs.push_str(r#" w:line="240" w:lineRule="auto""#);
                }
                LineSpacing::OnePointFive => {
                    spacing_attrs.push_str(r#" w:line="360" w:lineRule="auto""#);
                }
                LineSpacing::Double => {
                    spacing_attrs.push_str(r#" w:line="480" w:lineRule="auto""#);
                }
                LineSpacing::Multiple(m) => {
                    let val = (m * 240.0) as i64;
                    spacing_attrs.push_str(&format!(r#" w:line="{val}" w:lineRule="auto""#));
                }
                LineSpacing::Exact(pts) => {
                    let val = points_to_twips(*pts);
                    spacing_attrs.push_str(&format!(r#" w:line="{val}" w:lineRule="exact""#));
                }
                LineSpacing::AtLeast(pts) => {
                    let val = points_to_twips(*pts);
                    spacing_attrs.push_str(&format!(r#" w:line="{val}" w:lineRule="atLeast""#));
                }
            }
        }
        ppr.push_str(&format!("<w:spacing{spacing_attrs}/>"));
    }

    // Indentation
    let left = attrs.get_f64(&AttributeKey::IndentLeft);
    let right = attrs.get_f64(&AttributeKey::IndentRight);
    let first_line = attrs.get_f64(&AttributeKey::IndentFirstLine);

    if left.is_some() || right.is_some() || first_line.is_some() {
        let mut ind_attrs = String::new();
        if let Some(pts) = left {
            ind_attrs.push_str(&format!(r#" w:left="{}""#, points_to_twips(pts)));
        }
        if let Some(pts) = right {
            ind_attrs.push_str(&format!(r#" w:right="{}""#, points_to_twips(pts)));
        }
        if let Some(pts) = first_line {
            ind_attrs.push_str(&format!(r#" w:firstLine="{}""#, points_to_twips(pts)));
        }
        ppr.push_str(&format!("<w:ind{ind_attrs}/>"));
    }

    // Toggle properties
    if attrs.get_bool(&AttributeKey::KeepWithNext) == Some(true) {
        ppr.push_str("<w:keepNext/>");
    }
    if attrs.get_bool(&AttributeKey::KeepLinesTogether) == Some(true) {
        ppr.push_str("<w:keepLines/>");
    }
    if attrs.get_bool(&AttributeKey::PageBreakBefore) == Some(true) {
        ppr.push_str("<w:pageBreakBefore/>");
    }

    ppr
}

/// Write a `<w:r>` element.
fn write_run(doc: &DocumentModel, run_id: NodeId, xml: &mut String) {
    let run = match doc.node(run_id) {
        Some(n) => n,
        None => return,
    };

    xml.push_str("<w:r>");

    // Run properties
    let rpr = write_run_properties(run);
    if !rpr.is_empty() {
        xml.push_str("<w:rPr>");
        xml.push_str(&rpr);
        xml.push_str("</w:rPr>");
    }

    // Text children
    let children: Vec<NodeId> = run.children.clone();
    for child_id in children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };

        if child.node_type == NodeType::Text {
            if let Some(text) = &child.text_content {
                xml.push_str(r#"<w:t xml:space="preserve">"#);
                xml.push_str(&escape_xml(text));
                xml.push_str("</w:t>");
            }
        }
    }

    xml.push_str("</w:r>");
}

/// Generate run properties XML from a Node.
fn write_run_properties(run: &s1_model::Node) -> String {
    write_run_properties_from_attrs(&run.attributes)
}

/// Generate run properties XML from an AttributeMap.
///
/// Public so the style writer can reuse it.
pub fn write_run_properties_from_attrs(attrs: &s1_model::AttributeMap) -> String {
    let mut rpr = String::new();

    // Style reference
    if let Some(style_id) = attrs.get_string(&AttributeKey::StyleId) {
        rpr.push_str(&format!(
            r#"<w:rStyle w:val="{}"/>"#,
            escape_xml(style_id)
        ));
    }

    // Font family
    if let Some(font) = attrs.get_string(&AttributeKey::FontFamily) {
        let escaped = escape_xml(font);
        rpr.push_str(&format!(
            r#"<w:rFonts w:ascii="{escaped}" w:hAnsi="{escaped}"/>"#
        ));
    }

    // Bold
    if let Some(bold) = attrs.get_bool(&AttributeKey::Bold) {
        if bold {
            rpr.push_str("<w:b/>");
        } else {
            rpr.push_str(r#"<w:b w:val="false"/>"#);
        }
    }

    // Italic
    if let Some(italic) = attrs.get_bool(&AttributeKey::Italic) {
        if italic {
            rpr.push_str("<w:i/>");
        } else {
            rpr.push_str(r#"<w:i w:val="false"/>"#);
        }
    }

    // Strikethrough
    if let Some(true) = attrs.get_bool(&AttributeKey::Strikethrough) {
        rpr.push_str("<w:strike/>");
    }

    // Underline
    if let Some(AttributeValue::UnderlineStyle(style)) = attrs.get(&AttributeKey::Underline) {
        let val = match style {
            UnderlineStyle::None => "none",
            UnderlineStyle::Single => "single",
            UnderlineStyle::Double => "double",
            UnderlineStyle::Thick => "thick",
            UnderlineStyle::Dotted => "dotted",
            UnderlineStyle::Dashed => "dash",
            UnderlineStyle::Wave => "wave",
        };
        rpr.push_str(&format!(r#"<w:u w:val="{val}"/>"#));
    }

    // Font size (points → half-points)
    if let Some(pts) = attrs.get_f64(&AttributeKey::FontSize) {
        let half_pts = (pts * 2.0) as i64;
        rpr.push_str(&format!(r#"<w:sz w:val="{half_pts}"/>"#));
    }

    // Text color
    if let Some(color) = attrs.get_color(&AttributeKey::Color) {
        rpr.push_str(&format!(r#"<w:color w:val="{}"/>"#, color.to_hex()));
    }

    // Highlight color
    if let Some(color) = attrs.get_color(&AttributeKey::HighlightColor) {
        let name = color_to_highlight_name(color);
        rpr.push_str(&format!(r#"<w:highlight w:val="{name}"/>"#));
    }

    // Superscript / Subscript
    if attrs.get_bool(&AttributeKey::Superscript) == Some(true) {
        rpr.push_str(r#"<w:vertAlign w:val="superscript"/>"#);
    } else if attrs.get_bool(&AttributeKey::Subscript) == Some(true) {
        rpr.push_str(r#"<w:vertAlign w:val="subscript"/>"#);
    }

    // Language
    if let Some(lang) = attrs.get_string(&AttributeKey::Language) {
        rpr.push_str(&format!(
            r#"<w:lang w:val="{}"/>"#,
            escape_xml(lang)
        ));
    }

    rpr
}

/// Convert points to twips (twentieths of a point).
fn points_to_twips(pts: f64) -> i64 {
    (pts * 20.0) as i64
}

/// Best-effort mapping of Color to OOXML highlight color name.
fn color_to_highlight_name(color: Color) -> &'static str {
    match (color.r, color.g, color.b) {
        (255, 255, 0) => "yellow",
        (0, 255, 0) => "green",
        (0, 255, 255) => "cyan",
        (255, 0, 255) => "magenta",
        (0, 0, 255) => "blue",
        (255, 0, 0) => "red",
        (0, 0, 0) => "black",
        (255, 255, 255) => "white",
        _ => "yellow", // fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::{AttributeMap, Node};

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
    fn write_simple_document() {
        let doc = make_simple_doc("Hello World");
        let xml = write_document_xml(&doc);

        assert!(xml.contains("<w:t xml:space=\"preserve\">Hello World</w:t>"));
        assert!(xml.contains("<w:body>"));
        assert!(xml.contains("<w:p>"));
        assert!(xml.contains("<w:r>"));
    }

    #[test]
    fn write_bold_run() {
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
        doc.insert_node(run_id, 0, Node::text(text_id, "Bold"))
            .unwrap();

        let xml = write_document_xml(&doc);

        assert!(xml.contains("<w:b/>"));
        assert!(xml.contains(r#"<w:sz w:val="48"/>"#)); // 24pt = 48 half-points
    }

    #[test]
    fn write_paragraph_alignment() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        let mut para = Node::new(para_id, NodeType::Paragraph);
        para.attributes = AttributeMap::new().alignment(Alignment::Center);
        doc.insert_node(body_id, 0, para).unwrap();

        let xml = write_document_xml(&doc);
        assert!(xml.contains(r#"<w:jc w:val="center"/>"#));
    }

    #[test]
    fn write_paragraph_spacing() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        let mut para = Node::new(para_id, NodeType::Paragraph);
        para.attributes.set(
            AttributeKey::SpacingBefore,
            AttributeValue::Float(12.0),
        );
        para.attributes.set(
            AttributeKey::SpacingAfter,
            AttributeValue::Float(6.0),
        );
        doc.insert_node(body_id, 0, para).unwrap();

        let xml = write_document_xml(&doc);
        assert!(xml.contains(r#"w:before="240""#)); // 12pt = 240 twips
        assert!(xml.contains(r#"w:after="120""#)); // 6pt = 120 twips
    }

    #[test]
    fn write_escapes_special_chars() {
        let doc = make_simple_doc("A & B < C");
        let xml = write_document_xml(&doc);
        assert!(xml.contains("A &amp; B &lt; C"));
    }

    #[test]
    fn write_empty_paragraph() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let xml = write_document_xml(&doc);
        assert!(xml.contains("<w:p></w:p>"));
    }

    #[test]
    fn write_line_break() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let br_id = doc.next_id();
        doc.insert_node(para_id, 0, Node::new(br_id, NodeType::LineBreak))
            .unwrap();

        let xml = write_document_xml(&doc);
        assert!(xml.contains("<w:r><w:br/></w:r>"));
    }

    #[test]
    fn write_font_and_color() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let run_id = doc.next_id();
        let mut run = Node::new(run_id, NodeType::Run);
        run.attributes = AttributeMap::new()
            .font_family("Arial")
            .color(Color::RED);
        doc.insert_node(para_id, 0, run).unwrap();

        let text_id = doc.next_id();
        doc.insert_node(run_id, 0, Node::text(text_id, "Red"))
            .unwrap();

        let xml = write_document_xml(&doc);
        assert!(xml.contains(r#"w:ascii="Arial""#));
        assert!(xml.contains(r#"<w:color w:val="FF0000"/>"#));
    }
}
