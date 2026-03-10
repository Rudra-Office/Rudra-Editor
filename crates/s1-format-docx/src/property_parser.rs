//! Parse OOXML run properties (`w:rPr`) and paragraph properties (`w:pPr`).

use quick_xml::events::Event;
use quick_xml::Reader;
use s1_model::{
    Alignment, AttributeKey, AttributeMap, AttributeValue, Color, LineSpacing, UnderlineStyle,
};

use crate::error::DocxError;
use crate::xml_util::{get_attr, get_val, half_points_to_points, is_toggle_on, twips_to_points};

/// Parse `<w:rPr>` — run (character) formatting properties.
pub fn parse_run_properties(reader: &mut Reader<&[u8]>) -> Result<AttributeMap, DocxError> {
    let mut attrs = AttributeMap::new();
    parse_rpr_inner(reader, &mut attrs)?;
    Ok(attrs)
}

fn parse_rpr_inner(reader: &mut Reader<&[u8]>, attrs: &mut AttributeMap) -> Result<(), DocxError> {
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"rFonts" => {
                        // Font family: prefer w:ascii, then w:hAnsi
                        if let Some(font) = get_attr(&e, b"ascii")
                            .or_else(|| get_attr(&e, b"hAnsi"))
                            .or_else(|| get_attr(&e, b"cs"))
                        {
                            attrs.set(
                                AttributeKey::FontFamily,
                                AttributeValue::String(font),
                            );
                        }
                        skip_to_end(reader)?;
                    }
                    b"rStyle" => {
                        if let Some(style_id) = get_val(&e) {
                            attrs.set(
                                AttributeKey::StyleId,
                                AttributeValue::String(style_id),
                            );
                        }
                        skip_to_end(reader)?;
                    }
                    _ => {
                        skip_to_end(reader)?;
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"b" => {
                        attrs.set(AttributeKey::Bold, AttributeValue::Bool(is_toggle_on(&e)));
                    }
                    b"i" => {
                        attrs.set(
                            AttributeKey::Italic,
                            AttributeValue::Bool(is_toggle_on(&e)),
                        );
                    }
                    b"strike" => {
                        attrs.set(
                            AttributeKey::Strikethrough,
                            AttributeValue::Bool(is_toggle_on(&e)),
                        );
                    }
                    b"u" => {
                        let style = match get_val(&e).as_deref() {
                            Some("single") => UnderlineStyle::Single,
                            Some("double") => UnderlineStyle::Double,
                            Some("thick") => UnderlineStyle::Thick,
                            Some("dotted") => UnderlineStyle::Dotted,
                            Some("dash") | Some("dashed") => UnderlineStyle::Dashed,
                            Some("wave") => UnderlineStyle::Wave,
                            Some("none") => UnderlineStyle::None,
                            _ => UnderlineStyle::Single,
                        };
                        attrs.set(
                            AttributeKey::Underline,
                            AttributeValue::UnderlineStyle(style),
                        );
                    }
                    b"sz" => {
                        if let Some(val) = get_val(&e) {
                            if let Some(pts) = half_points_to_points(&val) {
                                attrs.set(AttributeKey::FontSize, AttributeValue::Float(pts));
                            }
                        }
                    }
                    b"color" => {
                        if let Some(hex) = get_val(&e) {
                            if hex != "auto" {
                                if let Some(color) = Color::from_hex(&hex) {
                                    attrs.set(
                                        AttributeKey::Color,
                                        AttributeValue::Color(color),
                                    );
                                }
                            }
                        }
                    }
                    b"highlight" => {
                        if let Some(color_name) = get_val(&e) {
                            if let Some(color) = highlight_name_to_color(&color_name) {
                                attrs.set(
                                    AttributeKey::HighlightColor,
                                    AttributeValue::Color(color),
                                );
                            }
                        }
                    }
                    b"rFonts" => {
                        if let Some(font) = get_attr(&e, b"ascii")
                            .or_else(|| get_attr(&e, b"hAnsi"))
                            .or_else(|| get_attr(&e, b"cs"))
                        {
                            attrs.set(
                                AttributeKey::FontFamily,
                                AttributeValue::String(font),
                            );
                        }
                    }
                    b"vertAlign" => {
                        match get_val(&e).as_deref() {
                            Some("superscript") => {
                                attrs.set(
                                    AttributeKey::Superscript,
                                    AttributeValue::Bool(true),
                                );
                            }
                            Some("subscript") => {
                                attrs.set(
                                    AttributeKey::Subscript,
                                    AttributeValue::Bool(true),
                                );
                            }
                            _ => {}
                        }
                    }
                    b"rStyle" => {
                        if let Some(style_id) = get_val(&e) {
                            attrs.set(
                                AttributeKey::StyleId,
                                AttributeValue::String(style_id),
                            );
                        }
                    }
                    b"spacing" => {
                        if let Some(val) = get_val(&e) {
                            if let Some(pts) = twips_to_points(&val) {
                                attrs.set(
                                    AttributeKey::FontSpacing,
                                    AttributeValue::Float(pts),
                                );
                            }
                        }
                    }
                    b"lang" => {
                        if let Some(lang) = get_val(&e) {
                            attrs.set(
                                AttributeKey::Language,
                                AttributeValue::String(lang),
                            );
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"rPr" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    Ok(())
}

/// Parse `<w:pPr>` — paragraph formatting properties.
pub fn parse_paragraph_properties(
    reader: &mut Reader<&[u8]>,
) -> Result<AttributeMap, DocxError> {
    let mut attrs = AttributeMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"pStyle" => {
                        if let Some(style_id) = get_val(&e) {
                            attrs.set(
                                AttributeKey::StyleId,
                                AttributeValue::String(style_id),
                            );
                        }
                        skip_to_end(reader)?;
                    }
                    b"rPr" => {
                        // Default run properties for the paragraph — skip for now
                        skip_to_end(reader)?;
                    }
                    _ => {
                        skip_to_end(reader)?;
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().as_ref().to_vec();
                match name.as_slice() {
                    b"jc" => {
                        let alignment = match get_val(&e).as_deref() {
                            Some("left") | Some("start") => Some(Alignment::Left),
                            Some("center") => Some(Alignment::Center),
                            Some("right") | Some("end") => Some(Alignment::Right),
                            Some("both") | Some("justify") => Some(Alignment::Justify),
                            _ => None,
                        };
                        if let Some(a) = alignment {
                            attrs.set(
                                AttributeKey::Alignment,
                                AttributeValue::Alignment(a),
                            );
                        }
                    }
                    b"pStyle" => {
                        if let Some(style_id) = get_val(&e) {
                            attrs.set(
                                AttributeKey::StyleId,
                                AttributeValue::String(style_id),
                            );
                        }
                    }
                    b"spacing" => {
                        if let Some(before) = get_attr(&e, b"before") {
                            if let Some(pts) = twips_to_points(&before) {
                                attrs.set(
                                    AttributeKey::SpacingBefore,
                                    AttributeValue::Float(pts),
                                );
                            }
                        }
                        if let Some(after) = get_attr(&e, b"after") {
                            if let Some(pts) = twips_to_points(&after) {
                                attrs.set(
                                    AttributeKey::SpacingAfter,
                                    AttributeValue::Float(pts),
                                );
                            }
                        }
                        // Line spacing
                        if let Some(line) = get_attr(&e, b"line") {
                            let rule = get_attr(&e, b"lineRule");
                            if let Ok(line_val) = line.parse::<f64>() {
                                let spacing = match rule.as_deref() {
                                    Some("exact") => LineSpacing::Exact(line_val / 20.0),
                                    Some("atLeast") => LineSpacing::AtLeast(line_val / 20.0),
                                    _ => {
                                        // "auto" — value is in 240ths of a line
                                        let multiple = line_val / 240.0;
                                        if (multiple - 1.0).abs() < 0.01 {
                                            LineSpacing::Single
                                        } else if (multiple - 1.5).abs() < 0.01 {
                                            LineSpacing::OnePointFive
                                        } else if (multiple - 2.0).abs() < 0.01 {
                                            LineSpacing::Double
                                        } else {
                                            LineSpacing::Multiple(multiple)
                                        }
                                    }
                                };
                                attrs.set(
                                    AttributeKey::LineSpacing,
                                    AttributeValue::LineSpacing(spacing),
                                );
                            }
                        }
                    }
                    b"ind" => {
                        if let Some(left) = get_attr(&e, b"left") {
                            if let Some(pts) = twips_to_points(&left) {
                                attrs.set(
                                    AttributeKey::IndentLeft,
                                    AttributeValue::Float(pts),
                                );
                            }
                        }
                        if let Some(right) = get_attr(&e, b"right") {
                            if let Some(pts) = twips_to_points(&right) {
                                attrs.set(
                                    AttributeKey::IndentRight,
                                    AttributeValue::Float(pts),
                                );
                            }
                        }
                        if let Some(first_line) = get_attr(&e, b"firstLine") {
                            if let Some(pts) = twips_to_points(&first_line) {
                                attrs.set(
                                    AttributeKey::IndentFirstLine,
                                    AttributeValue::Float(pts),
                                );
                            }
                        }
                    }
                    b"keepNext" => {
                        attrs.set(
                            AttributeKey::KeepWithNext,
                            AttributeValue::Bool(is_toggle_on(&e)),
                        );
                    }
                    b"keepLines" => {
                        attrs.set(
                            AttributeKey::KeepLinesTogether,
                            AttributeValue::Bool(is_toggle_on(&e)),
                        );
                    }
                    b"pageBreakBefore" => {
                        attrs.set(
                            AttributeKey::PageBreakBefore,
                            AttributeValue::Bool(is_toggle_on(&e)),
                        );
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"pPr" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }

    Ok(attrs)
}

/// Skip to the matching end tag (for elements we want to ignore).
fn skip_to_end(reader: &mut Reader<&[u8]>) -> Result<(), DocxError> {
    let mut depth = 1u32;
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(DocxError::Xml(format!("{e}"))),
            _ => {}
        }
    }
    Ok(())
}

/// Convert OOXML highlight color names to Color values.
fn highlight_name_to_color(name: &str) -> Option<Color> {
    match name {
        "yellow" => Some(Color::new(255, 255, 0)),
        "green" => Some(Color::new(0, 255, 0)),
        "cyan" => Some(Color::new(0, 255, 255)),
        "magenta" => Some(Color::new(255, 0, 255)),
        "blue" => Some(Color::new(0, 0, 255)),
        "red" => Some(Color::new(255, 0, 0)),
        "darkBlue" => Some(Color::new(0, 0, 139)),
        "darkCyan" => Some(Color::new(0, 139, 139)),
        "darkGreen" => Some(Color::new(0, 100, 0)),
        "darkMagenta" => Some(Color::new(139, 0, 139)),
        "darkRed" => Some(Color::new(139, 0, 0)),
        "darkYellow" => Some(Color::new(128, 128, 0)),
        "darkGray" => Some(Color::new(169, 169, 169)),
        "lightGray" => Some(Color::new(211, 211, 211)),
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bold_italic() {
        let xml = r#"<w:rPr><w:b/><w:i/></w:rPr>"#;
        let mut reader = Reader::from_str(xml);
        // Skip the opening rPr tag
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"rPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_run_properties(&mut reader).unwrap();
        assert_eq!(attrs.get_bool(&AttributeKey::Bold), Some(true));
        assert_eq!(attrs.get_bool(&AttributeKey::Italic), Some(true));
    }

    #[test]
    fn parse_bold_false() {
        let xml = r#"<w:rPr><w:b w:val="false"/></w:rPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"rPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_run_properties(&mut reader).unwrap();
        assert_eq!(attrs.get_bool(&AttributeKey::Bold), Some(false));
    }

    #[test]
    fn parse_font_size() {
        let xml = r#"<w:rPr><w:sz w:val="24"/></w:rPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"rPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_run_properties(&mut reader).unwrap();
        // 24 half-points = 12pt
        assert_eq!(attrs.get_f64(&AttributeKey::FontSize), Some(12.0));
    }

    #[test]
    fn parse_color() {
        let xml = r#"<w:rPr><w:color w:val="FF0000"/></w:rPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"rPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_run_properties(&mut reader).unwrap();
        assert_eq!(
            attrs.get_color(&AttributeKey::Color),
            Some(Color::RED)
        );
    }

    #[test]
    fn parse_font_family() {
        let xml = r#"<w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial"/></w:rPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"rPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_run_properties(&mut reader).unwrap();
        assert_eq!(
            attrs.get_string(&AttributeKey::FontFamily),
            Some("Arial")
        );
    }

    #[test]
    fn parse_paragraph_alignment() {
        let xml = r#"<w:pPr><w:jc w:val="center"/></w:pPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"pPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_paragraph_properties(&mut reader).unwrap();
        assert_eq!(
            attrs.get_alignment(&AttributeKey::Alignment),
            Some(Alignment::Center)
        );
    }

    #[test]
    fn parse_paragraph_spacing() {
        let xml = r#"<w:pPr><w:spacing w:before="240" w:after="120"/></w:pPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"pPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_paragraph_properties(&mut reader).unwrap();
        // 240 twips = 12pt, 120 twips = 6pt
        assert_eq!(attrs.get_f64(&AttributeKey::SpacingBefore), Some(12.0));
        assert_eq!(attrs.get_f64(&AttributeKey::SpacingAfter), Some(6.0));
    }

    #[test]
    fn parse_paragraph_indent() {
        let xml = r#"<w:pPr><w:ind w:left="720" w:firstLine="360"/></w:pPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"pPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_paragraph_properties(&mut reader).unwrap();
        // 720 twips = 36pt (0.5in), 360 twips = 18pt
        assert_eq!(attrs.get_f64(&AttributeKey::IndentLeft), Some(36.0));
        assert_eq!(attrs.get_f64(&AttributeKey::IndentFirstLine), Some(18.0));
    }

    #[test]
    fn parse_paragraph_style_ref() {
        let xml = r#"<w:pPr><w:pStyle w:val="Heading1"/></w:pPr>"#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"pPr" => break,
                Ok(Event::Eof) => panic!("unexpected EOF"),
                _ => {}
            }
        }
        let attrs = parse_paragraph_properties(&mut reader).unwrap();
        assert_eq!(
            attrs.get_string(&AttributeKey::StyleId),
            Some("Heading1")
        );
    }
}
