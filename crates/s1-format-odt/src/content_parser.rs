//! Parse `<office:body><office:text>` content from ODF `content.xml`.

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{
    AttributeKey, AttributeMap, AttributeValue, DocumentModel, FieldType, ListFormat, ListInfo,
    MediaId, Node, NodeType,
};

use crate::error::OdtError;
use crate::xml_util::get_attr;

/// Context passed to the content parser.
pub struct ParseContext {
    /// Automatic styles resolved from `<office:automatic-styles>`.
    pub auto_styles: HashMap<String, AttributeMap>,
    /// Map of image href paths → MediaId (populated by reader after extracting images).
    pub image_map: HashMap<String, MediaId>,
}

/// Parse the body of `content.xml` from a reader positioned at `<office:text>`.
///
/// The reader should be positioned just after consuming the `<office:text>` start tag.
pub fn parse_content_body(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    ctx: &ParseContext,
) -> Result<(), OdtError> {
    let body_id = doc
        .body_id()
        .ok_or_else(|| OdtError::InvalidStructure("Document has no body node".to_string()))?;

    let mut body_child_index = doc.node(body_id).map_or(0, |n| n.children.len());

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"p" => {
                        parse_paragraph_into(
                            reader,
                            doc,
                            e,
                            ctx,
                            body_id,
                            body_child_index,
                            false,
                            None,
                        )?;
                        body_child_index += 1;
                    }
                    b"h" => {
                        let level = get_attr(e, b"outline-level")
                            .and_then(|v| v.parse::<u8>().ok())
                            .unwrap_or(1);
                        parse_paragraph_into(
                            reader,
                            doc,
                            e,
                            ctx,
                            body_id,
                            body_child_index,
                            true,
                            Some(level),
                        )?;
                        body_child_index += 1;
                    }
                    b"list" => {
                        let count = parse_list(reader, doc, ctx, body_id, body_child_index, 0)?;
                        body_child_index += count;
                    }
                    b"table" => {
                        parse_table_into(reader, doc, ctx, body_id, body_child_index)?;
                        body_child_index += 1;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"text" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(())
}

/// Parse a `<text:p>` or `<text:h>` element and insert it into `parent_id` at `index`.
#[allow(clippy::too_many_arguments)]
fn parse_paragraph_into(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    start: &quick_xml::events::BytesStart<'_>,
    ctx: &ParseContext,
    parent_id: s1_model::NodeId,
    index: usize,
    is_heading: bool,
    heading_level: Option<u8>,
) -> Result<s1_model::NodeId, OdtError> {
    let para_id = doc.next_id();
    let mut para_node = Node::new(para_id, NodeType::Paragraph);

    // Apply auto-style or named style reference
    if let Some(style_name) = get_attr(start, b"style-name") {
        if let Some(auto_attrs) = ctx.auto_styles.get(&style_name) {
            para_node.attributes.merge(auto_attrs);
        } else {
            para_node
                .attributes
                .set(AttributeKey::StyleId, AttributeValue::String(style_name));
        }
    }

    // Set heading style
    if is_heading {
        if let Some(level) = heading_level {
            let style_name = format!("Heading{level}");
            para_node
                .attributes
                .set(AttributeKey::StyleId, AttributeValue::String(style_name));
        }
    }

    // Insert paragraph into parent
    doc.insert_node(parent_id, index, para_node)
        .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;

    // Now parse children and add them to the paragraph
    let end_tag: &[u8] = if is_heading { b"h" } else { b"p" };
    let mut child_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"span" => {
                        let added = parse_span_into(reader, doc, e, ctx, para_id, child_index)?;
                        child_index += added;
                    }
                    b"a" => {
                        // Hyperlink — treat inner content as runs
                        let added = parse_span_into(reader, doc, e, ctx, para_id, child_index)?;
                        child_index += added;
                    }
                    b"frame" => {
                        if parse_frame_into(reader, doc, e, ctx, para_id, child_index)? {
                            child_index += 1;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"line-break" => {
                        let lb_id = doc.next_id();
                        doc.insert_node(
                            para_id,
                            child_index,
                            Node::new(lb_id, NodeType::LineBreak),
                        )
                        .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;
                        child_index += 1;
                    }
                    b"tab" => {
                        let tab_id = doc.next_id();
                        doc.insert_node(para_id, child_index, Node::new(tab_id, NodeType::Tab))
                            .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;
                        child_index += 1;
                    }
                    b"s" => {
                        let count = get_attr(e, b"c")
                            .and_then(|v| v.parse::<usize>().ok())
                            .unwrap_or(1);
                        let run_id = doc.next_id();
                        doc.insert_node(para_id, child_index, Node::new(run_id, NodeType::Run))
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        let text_id = doc.next_id();
                        doc.insert_node(run_id, 0, Node::text(text_id, " ".repeat(count)))
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        child_index += 1;
                    }
                    b"page-number" => {
                        let field_id = doc.next_id();
                        let mut field_node = Node::new(field_id, NodeType::Field);
                        field_node.attributes.set(
                            AttributeKey::FieldType,
                            AttributeValue::FieldType(FieldType::PageNumber),
                        );
                        doc.insert_node(para_id, child_index, field_node)
                            .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;
                        child_index += 1;
                    }
                    b"page-count" => {
                        let field_id = doc.next_id();
                        let mut field_node = Node::new(field_id, NodeType::Field);
                        field_node.attributes.set(
                            AttributeKey::FieldType,
                            AttributeValue::FieldType(FieldType::PageCount),
                        );
                        doc.insert_node(para_id, child_index, field_node)
                            .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;
                        child_index += 1;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) => {
                if let Ok(text) = t.unescape() {
                    let text = text.to_string();
                    if !text.is_empty() {
                        let run_id = doc.next_id();
                        doc.insert_node(para_id, child_index, Node::new(run_id, NodeType::Run))
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        let text_id = doc.next_id();
                        doc.insert_node(run_id, 0, Node::text(text_id, text))
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        child_index += 1;
                    }
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == end_tag => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(para_id)
}

/// Parse a `<text:span>` and insert Run+Text nodes into `parent_id`.
///
/// Returns the number of nodes inserted at the parent level.
fn parse_span_into(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    start: &quick_xml::events::BytesStart<'_>,
    ctx: &ParseContext,
    parent_id: s1_model::NodeId,
    start_index: usize,
) -> Result<usize, OdtError> {
    let mut count = 0;

    // Get style attributes for this span
    let mut run_attrs = AttributeMap::new();
    if let Some(style_name) = get_attr(start, b"style-name") {
        if let Some(auto_attrs) = ctx.auto_styles.get(&style_name) {
            run_attrs.merge(auto_attrs);
        }
    }

    let end_tag = start.local_name();
    let end_tag_bytes = end_tag.as_ref().to_vec();

    loop {
        match reader.read_event() {
            Ok(Event::Text(ref t)) => {
                if let Ok(text) = t.unescape() {
                    let text = text.to_string();
                    if !text.is_empty() {
                        let run_id = doc.next_id();
                        let mut run_node = Node::new(run_id, NodeType::Run);
                        run_node.attributes.merge(&run_attrs);
                        doc.insert_node(parent_id, start_index + count, run_node)
                            .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;

                        let text_id = doc.next_id();
                        doc.insert_node(run_id, 0, Node::text(text_id, text))
                            .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;
                        count += 1;
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"line-break" => {
                        let lb_id = doc.next_id();
                        doc.insert_node(
                            parent_id,
                            start_index + count,
                            Node::new(lb_id, NodeType::LineBreak),
                        )
                        .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        count += 1;
                    }
                    b"tab" => {
                        let tab_id = doc.next_id();
                        doc.insert_node(
                            parent_id,
                            start_index + count,
                            Node::new(tab_id, NodeType::Tab),
                        )
                        .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        count += 1;
                    }
                    b"s" => {
                        let sc = get_attr(e, b"c")
                            .and_then(|v| v.parse::<usize>().ok())
                            .unwrap_or(1);
                        let run_id = doc.next_id();
                        let mut run_node = Node::new(run_id, NodeType::Run);
                        run_node.attributes.merge(&run_attrs);
                        doc.insert_node(parent_id, start_index + count, run_node)
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        let text_id = doc.next_id();
                        doc.insert_node(run_id, 0, Node::text(text_id, " ".repeat(sc)))
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        count += 1;
                    }
                    b"page-number" => {
                        let field_id = doc.next_id();
                        let mut field_node = Node::new(field_id, NodeType::Field);
                        field_node.attributes.set(
                            AttributeKey::FieldType,
                            AttributeValue::FieldType(FieldType::PageNumber),
                        );
                        doc.insert_node(parent_id, start_index + count, field_node)
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        count += 1;
                    }
                    b"page-count" => {
                        let field_id = doc.next_id();
                        let mut field_node = Node::new(field_id, NodeType::Field);
                        field_node.attributes.set(
                            AttributeKey::FieldType,
                            AttributeValue::FieldType(FieldType::PageCount),
                        );
                        doc.insert_node(parent_id, start_index + count, field_node)
                            .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                        count += 1;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == end_tag_bytes.as_slice() => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(count)
}

/// Parse a `<draw:frame>` element and insert an Image node if possible.
///
/// Returns true if an image node was inserted.
fn parse_frame_into(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    start: &quick_xml::events::BytesStart<'_>,
    ctx: &ParseContext,
    parent_id: s1_model::NodeId,
    index: usize,
) -> Result<bool, OdtError> {
    let alt_text = get_attr(start, b"name").unwrap_or_default();

    let width = get_attr(start, b"width").and_then(|v| crate::xml_util::parse_length(&v));
    let height = get_attr(start, b"height").and_then(|v| crate::xml_util::parse_length(&v));

    let mut href: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if e.local_name().as_ref() == b"image" =>
            {
                href = get_attr(e, b"href");
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"frame" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    let href = match href {
        Some(h) => h,
        None => return Ok(false),
    };

    let media_id = match ctx.image_map.get(&href) {
        Some(id) => *id,
        None => return Ok(false),
    };

    let img_id = doc.next_id();
    let mut img_node = Node::new(img_id, NodeType::Image);
    img_node.attributes.set(
        AttributeKey::ImageMediaId,
        AttributeValue::MediaId(media_id),
    );
    if let Some(w) = width {
        img_node
            .attributes
            .set(AttributeKey::ImageWidth, AttributeValue::Float(w));
    }
    if let Some(h) = height {
        img_node
            .attributes
            .set(AttributeKey::ImageHeight, AttributeValue::Float(h));
    }
    if !alt_text.is_empty() {
        img_node
            .attributes
            .set(AttributeKey::ImageAltText, AttributeValue::String(alt_text));
    }

    doc.insert_node(parent_id, index, img_node)
        .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;

    Ok(true)
}

/// Parse a `<text:list>` element, flattening items as Paragraph children of `parent_id`.
///
/// Returns the number of paragraphs added to the parent.
fn parse_list(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    ctx: &ParseContext,
    parent_id: s1_model::NodeId,
    start_index: usize,
    level: u8,
) -> Result<usize, OdtError> {
    let mut count = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"list-item" => {
                let item_count =
                    parse_list_item(reader, doc, ctx, parent_id, start_index + count, level)?;
                count += item_count;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"list" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(count)
}

/// Parse a `<text:list-item>` element.
fn parse_list_item(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    ctx: &ParseContext,
    parent_id: s1_model::NodeId,
    start_index: usize,
    level: u8,
) -> Result<usize, OdtError> {
    let mut count = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"p" | b"h" => {
                        let is_heading = local.as_ref() == b"h";
                        let heading_level = if is_heading {
                            get_attr(e, b"outline-level").and_then(|v| v.parse::<u8>().ok())
                        } else {
                            None
                        };
                        let node_id = parse_paragraph_into(
                            reader,
                            doc,
                            e,
                            ctx,
                            parent_id,
                            start_index + count,
                            is_heading,
                            heading_level,
                        )?;
                        // Set list info on the paragraph
                        if let Some(node) = doc.node_mut(node_id) {
                            node.attributes.set(
                                AttributeKey::ListInfo,
                                AttributeValue::ListInfo(ListInfo {
                                    level,
                                    num_format: ListFormat::Bullet,
                                    num_id: 0,
                                    start: None,
                                }),
                            );
                        }
                        count += 1;
                    }
                    b"list" => {
                        // Nested list → increment level
                        let nested = parse_list(
                            reader,
                            doc,
                            ctx,
                            parent_id,
                            start_index + count,
                            level + 1,
                        )?;
                        count += nested;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"list-item" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(count)
}

/// Parse a `<table:table>` element and insert it into `parent_id`.
fn parse_table_into(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    ctx: &ParseContext,
    parent_id: s1_model::NodeId,
    index: usize,
) -> Result<s1_model::NodeId, OdtError> {
    let table_id = doc.next_id();
    let table_node = Node::new(table_id, NodeType::Table);
    doc.insert_node(parent_id, index, table_node)
        .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;

    let mut row_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"table-row" => {
                        parse_table_row_into(reader, doc, ctx, table_id, row_index)?;
                        row_index += 1;
                    }
                    b"table-column" => {
                        // Skip column definitions
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"table" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(table_id)
}

/// Parse a `<table:table-row>` element and insert into parent table.
fn parse_table_row_into(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    ctx: &ParseContext,
    table_id: s1_model::NodeId,
    index: usize,
) -> Result<s1_model::NodeId, OdtError> {
    let row_id = doc.next_id();
    let row_node = Node::new(row_id, NodeType::TableRow);
    doc.insert_node(table_id, index, row_node)
        .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;

    let mut cell_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"table-cell" => {
                parse_table_cell_into(reader, doc, e, ctx, row_id, cell_index)?;
                cell_index += 1;
            }
            Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"table-cell" => {
                // Empty cell
                let cell_id = doc.next_id();
                doc.insert_node(row_id, cell_index, Node::new(cell_id, NodeType::TableCell))
                    .map_err(|er| OdtError::InvalidStructure(format!("{er:?}")))?;
                cell_index += 1;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"table-row" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(row_id)
}

/// Parse a `<table:table-cell>` element and insert into parent row.
fn parse_table_cell_into(
    reader: &mut Reader<&[u8]>,
    doc: &mut DocumentModel,
    start: &quick_xml::events::BytesStart<'_>,
    ctx: &ParseContext,
    row_id: s1_model::NodeId,
    index: usize,
) -> Result<s1_model::NodeId, OdtError> {
    let cell_id = doc.next_id();
    let mut cell_node = Node::new(cell_id, NodeType::TableCell);

    // Column span
    if let Some(span) = get_attr(start, b"number-columns-spanned") {
        if let Ok(n) = span.parse::<i64>() {
            if n > 1 {
                cell_node
                    .attributes
                    .set(AttributeKey::ColSpan, AttributeValue::Int(n));
            }
        }
    }

    // Row span
    if let Some(span) = get_attr(start, b"number-rows-spanned") {
        if let Ok(n) = span.parse::<i64>() {
            if n > 1 {
                cell_node
                    .attributes
                    .set(AttributeKey::RowSpan, AttributeValue::Int(n));
            }
        }
    }

    // Apply cell style
    if let Some(style_name) = get_attr(start, b"style-name") {
        if let Some(auto_attrs) = ctx.auto_styles.get(&style_name) {
            // Extract cell-relevant attributes
            if let Some(va) = auto_attrs.get(&AttributeKey::VerticalAlign) {
                cell_node
                    .attributes
                    .set(AttributeKey::VerticalAlign, va.clone());
            }
            if let Some(bg) = auto_attrs.get(&AttributeKey::CellBackground) {
                cell_node
                    .attributes
                    .set(AttributeKey::CellBackground, bg.clone());
            }
        }
    }

    doc.insert_node(row_id, index, cell_node)
        .map_err(|e| OdtError::InvalidStructure(format!("{e:?}")))?;

    let mut child_index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"p" | b"h" => {
                        let is_heading = local.as_ref() == b"h";
                        let heading_level = if is_heading {
                            get_attr(e, b"outline-level").and_then(|v| v.parse::<u8>().ok())
                        } else {
                            None
                        };
                        parse_paragraph_into(
                            reader,
                            doc,
                            e,
                            ctx,
                            cell_id,
                            child_index,
                            is_heading,
                            heading_level,
                        )?;
                        child_index += 1;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"table-cell" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }
    }

    Ok(cell_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_ctx() -> ParseContext {
        ParseContext {
            auto_styles: HashMap::new(),
            image_map: HashMap::new(),
        }
    }

    fn parse_body_xml(xml: &str) -> DocumentModel {
        let full = format!(
            r#"<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"><office:body><office:text>{}</office:text></office:body></office:document-content>"#,
            xml
        );
        let mut doc = DocumentModel::new();
        let ctx = make_ctx();
        let mut reader = Reader::from_reader(full.as_bytes());

        // Advance to <office:text>
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"text" => break,
                Ok(Event::Eof) => panic!("no <office:text> found"),
                _ => {}
            }
        }

        parse_content_body(&mut reader, &mut doc, &ctx).unwrap();
        doc
    }

    #[test]
    fn parse_single_paragraph() {
        let doc = parse_body_xml("<text:p>Hello world</text:p>");
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1);

        let para = doc.node(body.children[0]).unwrap();
        assert_eq!(para.node_type, NodeType::Paragraph);
        assert_eq!(para.children.len(), 1); // one run

        let run = doc.node(para.children[0]).unwrap();
        assert_eq!(run.node_type, NodeType::Run);
        assert_eq!(run.children.len(), 1);

        let text = doc.node(run.children[0]).unwrap();
        assert_eq!(text.text_content.as_deref(), Some("Hello world"));
    }

    #[test]
    fn parse_multiple_paragraphs() {
        let doc =
            parse_body_xml("<text:p>First</text:p><text:p>Second</text:p><text:p>Third</text:p>");
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 3);
    }

    #[test]
    fn parse_span_formatting() {
        let full = r#"<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"><office:automatic-styles><style:style style:name="T1" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style></office:automatic-styles><office:body><office:text><text:p>Hello <text:span text:style-name="T1">bold</text:span> world</text:p></office:text></office:body></office:document-content>"#;

        let mut doc = DocumentModel::new();
        let mut reader = Reader::from_reader(full.as_bytes());

        // Parse auto styles
        let mut auto_styles = HashMap::new();
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"automatic-styles" => {
                    auto_styles = crate::style_parser::parse_automatic_styles(&mut reader).unwrap();
                    break;
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
        }

        // Advance to <office:text>
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"text" => break,
                Ok(Event::Eof) => panic!("no <office:text> found"),
                _ => {}
            }
        }

        let ctx = ParseContext {
            auto_styles,
            image_map: HashMap::new(),
        };
        parse_content_body(&mut reader, &mut doc, &ctx).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1);

        let para = doc.node(body.children[0]).unwrap();
        // Should have 3 runs: "Hello ", bold "bold", " world"
        assert!(para.children.len() >= 2);

        // Check the bold run has Bold attribute
        let bold_run = doc.node(para.children[1]).unwrap();
        assert_eq!(
            bold_run.attributes.get_bool(&AttributeKey::Bold),
            Some(true)
        );
    }

    #[test]
    fn parse_list_items() {
        let doc = parse_body_xml(
            r#"<text:list><text:list-item><text:p>Item 1</text:p></text:list-item><text:list-item><text:p>Item 2</text:p></text:list-item></text:list>"#,
        );
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        // List items flattened as body children
        assert_eq!(body.children.len(), 2);

        let p1 = doc.node(body.children[0]).unwrap();
        assert_eq!(p1.node_type, NodeType::Paragraph);
        // Should have ListInfo attribute
        assert!(p1.attributes.get(&AttributeKey::ListInfo).is_some());
    }

    #[test]
    fn parse_table_basic() {
        let doc = parse_body_xml(
            r#"<table:table><table:table-row><table:table-cell><text:p>A1</text:p></table:table-cell><table:table-cell><text:p>B1</text:p></table:table-cell></table:table-row></table:table>"#,
        );
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1);

        let table = doc.node(body.children[0]).unwrap();
        assert_eq!(table.node_type, NodeType::Table);
        assert_eq!(table.children.len(), 1); // 1 row

        let row = doc.node(table.children[0]).unwrap();
        assert_eq!(row.node_type, NodeType::TableRow);
        assert_eq!(row.children.len(), 2); // 2 cells
    }

    #[test]
    fn parse_heading() {
        let doc = parse_body_xml(r#"<text:h text:outline-level="1">Chapter 1</text:h>"#);
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1);

        let heading = doc.node(body.children[0]).unwrap();
        assert_eq!(heading.node_type, NodeType::Paragraph);
        assert_eq!(
            heading.attributes.get_string(&AttributeKey::StyleId),
            Some("Heading1")
        );
    }

    #[test]
    fn parse_empty_paragraph() {
        let doc = parse_body_xml("<text:p></text:p>");
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1);
        let para = doc.node(body.children[0]).unwrap();
        assert!(para.children.is_empty());
    }
}
