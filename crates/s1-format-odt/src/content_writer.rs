//! Write `<office:text>` body content for ODF `content.xml`.

use std::collections::HashMap;

use s1_model::{AttributeKey, AttributeMap, AttributeValue, DocumentModel, FieldType, NodeType};

use crate::property_writer::{
    write_paragraph_properties, write_table_cell_properties, write_text_properties,
};
use crate::xml_util::{escape_xml, points_to_cm};

/// An auto-style definition collected during writing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AutoStyleKey {
    text_props: String,
    para_props: String,
    cell_props: String,
    family: String,
}

/// Image entry discovered during writing.
pub struct ImageEntry {
    pub href: String,
    pub media_id: s1_model::MediaId,
}

/// Write the full `content.xml` from a `DocumentModel`.
///
/// Returns `(xml_string, image_entries)`.
pub fn write_content_xml(doc: &DocumentModel) -> (String, Vec<ImageEntry>) {
    let mut auto_styles: HashMap<AutoStyleKey, String> = HashMap::new();
    let mut auto_counter = 0u32;
    let mut images: Vec<ImageEntry> = Vec::new();

    // First pass: collect body XML and auto-styles
    let body_xml = write_body(doc, &mut auto_styles, &mut auto_counter, &mut images);

    // Build the full content.xml
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0">"#,
    );

    // Write automatic styles
    if !auto_styles.is_empty() {
        xml.push_str("<office:automatic-styles>");
        // Sort for deterministic output
        let mut sorted: Vec<_> = auto_styles.iter().collect();
        sorted.sort_by(|(_, a), (_, b)| a.cmp(b));
        for (key, name) in sorted {
            xml.push_str(&format!(
                r#"<style:style style:name="{}" style:family="{}""#,
                name, key.family
            ));
            // Check if there's a parent style reference
            xml.push('>');
            if !key.para_props.is_empty() {
                xml.push_str(&key.para_props);
            }
            if !key.text_props.is_empty() {
                xml.push_str(&key.text_props);
            }
            if !key.cell_props.is_empty() {
                xml.push_str(&key.cell_props);
            }
            xml.push_str("</style:style>");
        }
        xml.push_str("</office:automatic-styles>");
    }

    xml.push_str("<office:body><office:text>");
    xml.push_str(&body_xml);
    xml.push_str("</office:text></office:body></office:document-content>");

    (xml, images)
}

/// Write the body children (paragraphs, tables, etc.).
fn write_body(
    doc: &DocumentModel,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
    images: &mut Vec<ImageEntry>,
) -> String {
    let mut xml = String::new();

    let body_id = match doc.body_id() {
        Some(id) => id,
        None => return xml,
    };

    let body = match doc.node(body_id) {
        Some(n) => n,
        None => return xml,
    };

    // Track list state for reconstructing nested lists
    let mut list_stack: Vec<u8> = Vec::new(); // stack of list levels

    for &child_id in &body.children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };

        match child.node_type {
            NodeType::Paragraph => {
                // Check if this paragraph is a list item
                let list_info = child.attributes.get(&AttributeKey::ListInfo);
                if let Some(AttributeValue::ListInfo(info)) = list_info {
                    let target_level = info.level;

                    // Close list levels that are deeper than target
                    while list_stack.last().copied().unwrap_or(255) > target_level {
                        xml.push_str("</text:list-item></text:list>");
                        list_stack.pop();
                    }

                    // Open new list levels as needed
                    while list_stack.len() <= target_level as usize {
                        let current_depth = list_stack.len() as u8;
                        if current_depth > 0 {
                            // Open a nested list within the current list-item
                        }
                        xml.push_str("<text:list>");
                        list_stack.push(current_depth);
                    }

                    xml.push_str("<text:list-item>");
                    write_paragraph(doc, child_id, &mut xml, auto_styles, counter, images);
                    // Don't close list-item yet — next item might need nesting
                    xml.push_str("</text:list-item>");
                } else {
                    // Close any open lists
                    close_list_stack(&mut list_stack, &mut xml);

                    // Check if heading
                    let is_heading = child
                        .attributes
                        .get_string(&AttributeKey::StyleId)
                        .is_some_and(|s| s.starts_with("Heading"));

                    if is_heading {
                        let level = child
                            .attributes
                            .get_string(&AttributeKey::StyleId)
                            .and_then(|s| s.strip_prefix("Heading"))
                            .and_then(|l| l.parse::<u8>().ok())
                            .unwrap_or(1);
                        write_heading(doc, child_id, level, &mut xml, auto_styles, counter, images);
                    } else {
                        write_paragraph(doc, child_id, &mut xml, auto_styles, counter, images);
                    }
                }
            }
            NodeType::Table => {
                close_list_stack(&mut list_stack, &mut xml);
                write_table(doc, child_id, &mut xml, auto_styles, counter, images);
            }
            _ => {}
        }
    }

    close_list_stack(&mut list_stack, &mut xml);
    xml
}

/// Close all open list levels.
fn close_list_stack(stack: &mut Vec<u8>, xml: &mut String) {
    while !stack.is_empty() {
        xml.push_str("</text:list>");
        stack.pop();
    }
}

/// Write a paragraph as `<text:p>`.
fn write_paragraph(
    doc: &DocumentModel,
    para_id: s1_model::NodeId,
    xml: &mut String,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
    images: &mut Vec<ImageEntry>,
) {
    let para = match doc.node(para_id) {
        Some(n) => n,
        None => return,
    };

    // Check if we need an auto-style for paragraph formatting
    let style_name = get_or_create_paragraph_auto_style(&para.attributes, auto_styles, counter);

    if let Some(ref name) = style_name {
        xml.push_str(&format!(r#"<text:p text:style-name="{name}">"#));
    } else if let Some(sid) = para.attributes.get_string(&AttributeKey::StyleId) {
        xml.push_str(&format!(
            r#"<text:p text:style-name="{}">"#,
            escape_xml(sid)
        ));
    } else {
        xml.push_str("<text:p>");
    }

    write_paragraph_children(doc, para_id, xml, auto_styles, counter, images);

    xml.push_str("</text:p>");
}

/// Write a heading as `<text:h>`.
fn write_heading(
    doc: &DocumentModel,
    para_id: s1_model::NodeId,
    level: u8,
    xml: &mut String,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
    images: &mut Vec<ImageEntry>,
) {
    let para = match doc.node(para_id) {
        Some(n) => n,
        None => return,
    };

    let style_ref = para
        .attributes
        .get_string(&AttributeKey::StyleId)
        .unwrap_or("Heading1");

    xml.push_str(&format!(
        r#"<text:h text:style-name="{}" text:outline-level="{}">"#,
        escape_xml(style_ref),
        level,
    ));

    write_paragraph_children(doc, para_id, xml, auto_styles, counter, images);
    xml.push_str("</text:h>");
}

/// Write children of a paragraph (runs, breaks, fields, images).
fn write_paragraph_children(
    doc: &DocumentModel,
    para_id: s1_model::NodeId,
    xml: &mut String,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
    images: &mut Vec<ImageEntry>,
) {
    let para = match doc.node(para_id) {
        Some(n) => n,
        None => return,
    };

    for &child_id in &para.children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };

        match child.node_type {
            NodeType::Run => {
                write_run(doc, child_id, xml, auto_styles, counter);
            }
            NodeType::LineBreak => {
                xml.push_str("<text:line-break/>");
            }
            NodeType::Tab => {
                xml.push_str("<text:tab/>");
            }
            NodeType::Field => {
                write_field(child, xml);
            }
            NodeType::Image => {
                write_image(doc, child, xml, images);
            }
            _ => {}
        }
    }
}

/// Write a run as `<text:span>` (or bare text if no formatting).
fn write_run(
    doc: &DocumentModel,
    run_id: s1_model::NodeId,
    xml: &mut String,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
) {
    let run = match doc.node(run_id) {
        Some(n) => n,
        None => return,
    };

    let text_props = write_text_properties(&run.attributes);
    let has_formatting = !text_props.is_empty();

    if has_formatting {
        let key = AutoStyleKey {
            text_props: text_props.clone(),
            para_props: String::new(),
            cell_props: String::new(),
            family: "text".to_string(),
        };
        let name = auto_styles
            .entry(key)
            .or_insert_with(|| {
                *counter += 1;
                format!("T{}", *counter)
            })
            .clone();
        xml.push_str(&format!(r#"<text:span text:style-name="{name}">"#));
    }

    // Write run children (text nodes)
    for &child_id in &run.children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };
        match child.node_type {
            NodeType::Text => {
                if let Some(ref text) = child.text_content {
                    xml.push_str(&escape_xml(text));
                }
            }
            NodeType::LineBreak => xml.push_str("<text:line-break/>"),
            NodeType::Tab => xml.push_str("<text:tab/>"),
            _ => {}
        }
    }

    if has_formatting {
        xml.push_str("</text:span>");
    }
}

/// Write a field node.
fn write_field(node: &s1_model::Node, xml: &mut String) {
    if let Some(AttributeValue::FieldType(ft)) = node.attributes.get(&AttributeKey::FieldType) {
        match ft {
            FieldType::PageNumber => xml.push_str("<text:page-number/>"),
            FieldType::PageCount => xml.push_str("<text:page-count/>"),
            _ => {}
        }
    }
}

/// Write an image node as `<draw:frame><draw:image>`.
fn write_image(
    doc: &DocumentModel,
    node: &s1_model::Node,
    xml: &mut String,
    images: &mut Vec<ImageEntry>,
) {
    let media_id = match node.attributes.get(&AttributeKey::ImageMediaId) {
        Some(AttributeValue::MediaId(id)) => *id,
        _ => return,
    };

    let width = node
        .attributes
        .get_f64(&AttributeKey::ImageWidth)
        .unwrap_or(72.0);
    let height = node
        .attributes
        .get_f64(&AttributeKey::ImageHeight)
        .unwrap_or(72.0);
    let alt_text = node
        .attributes
        .get_string(&AttributeKey::ImageAltText)
        .unwrap_or("");

    // Determine image path in ODT
    let ext = doc
        .media()
        .get(media_id)
        .map(|m| crate::xml_util::extension_for_mime(&m.content_type).to_string())
        .unwrap_or_else(|| "png".to_string());

    let href = format!("Pictures/{}.{}", media_id.0, ext);

    images.push(ImageEntry {
        href: href.clone(),
        media_id,
    });

    xml.push_str(&format!(
        r#"<draw:frame draw:name="{}" svg:width="{}" svg:height="{}">"#,
        escape_xml(alt_text),
        points_to_cm(width),
        points_to_cm(height),
    ));
    xml.push_str(&format!(
        r#"<draw:image xlink:href="{}" xlink:type="simple" xlink:show="embed" xlink:actuate="onLoad"/>"#,
        escape_xml(&href),
    ));
    xml.push_str("</draw:frame>");
}

/// Write a table as `<table:table>`.
fn write_table(
    doc: &DocumentModel,
    table_id: s1_model::NodeId,
    xml: &mut String,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
    images: &mut Vec<ImageEntry>,
) {
    let table = match doc.node(table_id) {
        Some(n) => n,
        None => return,
    };

    xml.push_str("<table:table>");

    for &row_id in &table.children {
        let row = match doc.node(row_id) {
            Some(n) if n.node_type == NodeType::TableRow => n,
            _ => continue,
        };

        xml.push_str("<table:table-row>");

        for &cell_id in &row.children {
            let cell = match doc.node(cell_id) {
                Some(n) if n.node_type == NodeType::TableCell => n,
                _ => continue,
            };

            // Cell style
            let cell_style = get_or_create_cell_auto_style(&cell.attributes, auto_styles, counter);

            let mut cell_tag = String::from("<table:table-cell");
            if let Some(ref name) = cell_style {
                cell_tag.push_str(&format!(r#" table:style-name="{name}""#));
            }

            // Col span
            if let Some(n) = cell.attributes.get_i64(&AttributeKey::ColSpan) {
                if n > 1 {
                    cell_tag.push_str(&format!(r#" table:number-columns-spanned="{n}""#));
                }
            }
            // Row span
            if let Some(n) = cell.attributes.get_i64(&AttributeKey::RowSpan) {
                if n > 1 {
                    cell_tag.push_str(&format!(r#" table:number-rows-spanned="{n}""#));
                }
            }

            cell_tag.push('>');
            xml.push_str(&cell_tag);

            // Cell contents (paragraphs)
            if cell.children.is_empty() {
                // ODF requires at least one <text:p/> in a cell
                xml.push_str("<text:p/>");
            } else {
                for &cc_id in &cell.children {
                    let cc = match doc.node(cc_id) {
                        Some(n) => n,
                        None => continue,
                    };
                    if cc.node_type == NodeType::Paragraph {
                        write_paragraph(doc, cc_id, xml, auto_styles, counter, images);
                    }
                }
            }

            xml.push_str("</table:table-cell>");
        }

        xml.push_str("</table:table-row>");
    }

    xml.push_str("</table:table>");
}

/// Get or create a paragraph-level auto-style. Returns `None` if no formatting needed.
fn get_or_create_paragraph_auto_style(
    attrs: &AttributeMap,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
) -> Option<String> {
    let para_props = write_paragraph_properties(attrs);

    if para_props.is_empty() {
        return None;
    }

    let key = AutoStyleKey {
        text_props: String::new(),
        para_props,
        cell_props: String::new(),
        family: "paragraph".to_string(),
    };

    let name = auto_styles
        .entry(key)
        .or_insert_with(|| {
            *counter += 1;
            format!("P{}", *counter)
        })
        .clone();

    Some(name)
}

/// Get or create a table-cell auto-style. Returns `None` if no formatting needed.
fn get_or_create_cell_auto_style(
    attrs: &AttributeMap,
    auto_styles: &mut HashMap<AutoStyleKey, String>,
    counter: &mut u32,
) -> Option<String> {
    let cell_props = write_table_cell_properties(attrs);

    if cell_props.is_empty() {
        return None;
    }

    let key = AutoStyleKey {
        text_props: String::new(),
        para_props: String::new(),
        cell_props,
        family: "table-cell".to_string(),
    };

    let name = auto_styles
        .entry(key)
        .or_insert_with(|| {
            *counter += 1;
            format!("C{}", *counter)
        })
        .clone();

    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::{Node, NodeType};

    fn build_simple_doc(text: &str) -> DocumentModel {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        let para = Node::new(para_id, NodeType::Paragraph);
        doc.insert_node(body_id, 0, para).unwrap();

        let run_id = doc.next_id();
        let run = Node::new(run_id, NodeType::Run);
        doc.insert_node(para_id, 0, run).unwrap();

        let text_id = doc.next_id();
        let text_node = Node::text(text_id, text);
        doc.insert_node(run_id, 0, text_node).unwrap();

        doc
    }

    #[test]
    fn write_single_paragraph() {
        let doc = build_simple_doc("Hello world");
        let (xml, _) = write_content_xml(&doc);
        assert!(xml.contains("<text:p>Hello world</text:p>"));
    }

    #[test]
    fn write_bold_text() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        let para = Node::new(para_id, NodeType::Paragraph);
        doc.insert_node(body_id, 0, para).unwrap();

        let run_id = doc.next_id();
        let mut run = Node::new(run_id, NodeType::Run);
        run.attributes = AttributeMap::new().bold(true);
        doc.insert_node(para_id, 0, run).unwrap();

        let text_id = doc.next_id();
        let text_node = Node::text(text_id, "bold text");
        doc.insert_node(run_id, 0, text_node).unwrap();

        let (xml, _) = write_content_xml(&doc);
        assert!(xml.contains("text:span"));
        assert!(xml.contains("bold text"));
        assert!(xml.contains(r#"fo:font-weight="bold""#));
    }

    #[test]
    fn write_escapes_special_chars() {
        let doc = build_simple_doc("A < B & C > D");
        let (xml, _) = write_content_xml(&doc);
        assert!(xml.contains("A &lt; B &amp; C &gt; D"));
    }

    #[test]
    fn write_empty_paragraph() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        let para = Node::new(para_id, NodeType::Paragraph);
        doc.insert_node(body_id, 0, para).unwrap();

        let (xml, _) = write_content_xml(&doc);
        assert!(xml.contains("<text:p></text:p>") || xml.contains("<text:p/>"));
    }

    #[test]
    fn write_table() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let table_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(table_id, NodeType::Table))
            .unwrap();

        let row_id = doc.next_id();
        doc.insert_node(table_id, 0, Node::new(row_id, NodeType::TableRow))
            .unwrap();

        let cell_id = doc.next_id();
        doc.insert_node(row_id, 0, Node::new(cell_id, NodeType::TableCell))
            .unwrap();

        let para_id = doc.next_id();
        doc.insert_node(cell_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let run_id = doc.next_id();
        doc.insert_node(para_id, 0, Node::new(run_id, NodeType::Run))
            .unwrap();

        let text_id = doc.next_id();
        doc.insert_node(run_id, 0, Node::text(text_id, "Cell"))
            .unwrap();

        let (xml, _) = write_content_xml(&doc);
        assert!(xml.contains("<table:table>"));
        assert!(xml.contains("<table:table-row>"));
        assert!(xml.contains("<table:table-cell>"));
        assert!(xml.contains("Cell"));
    }
}
