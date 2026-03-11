//! Parse ODF formatting properties into `AttributeMap`.

use quick_xml::events::BytesStart;
use s1_model::{
    Alignment, AttributeKey, AttributeMap, AttributeValue, Color, LineSpacing, UnderlineStyle,
};

use crate::xml_util::{get_attr, parse_length, parse_percentage};

/// Parse `<style:text-properties>` attributes into an `AttributeMap`.
pub fn parse_text_properties(e: &BytesStart<'_>) -> AttributeMap {
    let mut attrs = AttributeMap::new();

    // Bold: fo:font-weight="bold"
    if let Some(fw) = get_attr(e, b"font-weight") {
        attrs.set(AttributeKey::Bold, AttributeValue::Bool(fw == "bold"));
    }

    // Italic: fo:font-style="italic"
    if let Some(fs) = get_attr(e, b"font-style") {
        attrs.set(AttributeKey::Italic, AttributeValue::Bool(fs == "italic"));
    }

    // Font size: fo:font-size="12pt"
    if let Some(sz) = get_attr(e, b"font-size") {
        if let Some(pts) = parse_font_size(&sz) {
            attrs.set(AttributeKey::FontSize, AttributeValue::Float(pts));
        }
    }

    // Font family: style:font-name or fo:font-family
    if let Some(ff) = get_attr(e, b"font-name").or_else(|| get_attr(e, b"font-family")) {
        // Strip quotes if present
        let ff = ff.trim_matches('\'').trim_matches('"').to_string();
        attrs.set(AttributeKey::FontFamily, AttributeValue::String(ff));
    }

    // Color: fo:color="#FF0000"
    if let Some(c) = get_attr(e, b"color") {
        if let Some(color) = Color::from_hex(c.trim_start_matches('#')) {
            attrs.set(AttributeKey::Color, AttributeValue::Color(color));
        }
    }

    // Underline: style:text-underline-style="solid"
    if let Some(ul) = get_attr(e, b"text-underline-style") {
        let style = match ul.as_str() {
            "solid" => UnderlineStyle::Single,
            "double" => UnderlineStyle::Double,
            "dotted" => UnderlineStyle::Dotted,
            "dash" => UnderlineStyle::Dashed,
            "wave" => UnderlineStyle::Wave,
            "none" => UnderlineStyle::None,
            _ => UnderlineStyle::Single,
        };
        if style != UnderlineStyle::None {
            attrs.set(
                AttributeKey::Underline,
                AttributeValue::UnderlineStyle(style),
            );
        }
    }

    // Strikethrough: style:text-line-through-style="solid"
    if let Some(lt) = get_attr(e, b"text-line-through-style") {
        if lt != "none" {
            attrs.set(AttributeKey::Strikethrough, AttributeValue::Bool(true));
        }
    }

    // Highlight/background: fo:background-color="#FFFF00"
    if let Some(bg) = get_attr(e, b"background-color") {
        if bg != "transparent" {
            if let Some(color) = Color::from_hex(bg.trim_start_matches('#')) {
                attrs.set(AttributeKey::HighlightColor, AttributeValue::Color(color));
            }
        }
    }

    attrs
}

/// Parse `<style:paragraph-properties>` attributes.
pub fn parse_paragraph_properties(e: &BytesStart<'_>) -> AttributeMap {
    let mut attrs = AttributeMap::new();

    // Alignment: fo:text-align
    if let Some(align) = get_attr(e, b"text-align") {
        let a = match align.as_str() {
            "start" | "left" => Alignment::Left,
            "center" => Alignment::Center,
            "end" | "right" => Alignment::Right,
            "justify" => Alignment::Justify,
            _ => Alignment::Left,
        };
        attrs.set(AttributeKey::Alignment, AttributeValue::Alignment(a));
    }

    // Spacing before: fo:margin-top
    if let Some(mt) = get_attr(e, b"margin-top") {
        if let Some(pts) = parse_length(&mt) {
            attrs.set(AttributeKey::SpacingBefore, AttributeValue::Float(pts));
        }
    }

    // Spacing after: fo:margin-bottom
    if let Some(mb) = get_attr(e, b"margin-bottom") {
        if let Some(pts) = parse_length(&mb) {
            attrs.set(AttributeKey::SpacingAfter, AttributeValue::Float(pts));
        }
    }

    // Left indent: fo:margin-left
    if let Some(ml) = get_attr(e, b"margin-left") {
        if let Some(pts) = parse_length(&ml) {
            attrs.set(AttributeKey::IndentLeft, AttributeValue::Float(pts));
        }
    }

    // Right indent: fo:margin-right
    if let Some(mr) = get_attr(e, b"margin-right") {
        if let Some(pts) = parse_length(&mr) {
            attrs.set(AttributeKey::IndentRight, AttributeValue::Float(pts));
        }
    }

    // First-line indent: fo:text-indent
    if let Some(ti) = get_attr(e, b"text-indent") {
        if let Some(pts) = parse_length(&ti) {
            attrs.set(AttributeKey::IndentFirstLine, AttributeValue::Float(pts));
        }
    }

    // Line spacing: fo:line-height
    if let Some(lh) = get_attr(e, b"line-height") {
        if let Some(pct) = parse_percentage(&lh) {
            attrs.set(
                AttributeKey::LineSpacing,
                AttributeValue::LineSpacing(LineSpacing::Multiple(pct)),
            );
        } else if let Some(pts) = parse_length(&lh) {
            attrs.set(
                AttributeKey::LineSpacing,
                AttributeValue::LineSpacing(LineSpacing::Exact(pts)),
            );
        }
    }

    // Page break before: fo:break-before="page"
    if let Some(bb) = get_attr(e, b"break-before") {
        if bb == "page" {
            attrs.set(AttributeKey::PageBreakBefore, AttributeValue::Bool(true));
        }
    }

    // Keep with next: fo:keep-with-next="always"
    if let Some(kwn) = get_attr(e, b"keep-with-next") {
        if kwn == "always" {
            attrs.set(AttributeKey::KeepWithNext, AttributeValue::Bool(true));
        }
    }

    attrs
}

/// Parse a font size string (e.g. "12pt", "16px", "1cm") to points.
fn parse_font_size(s: &str) -> Option<f64> {
    parse_length(s)
}

/// Parse `<style:table-cell-properties>` attributes.
#[allow(dead_code)]
pub fn parse_table_cell_properties(e: &BytesStart<'_>) -> AttributeMap {
    let mut attrs = AttributeMap::new();

    // Vertical alignment
    if let Some(va) = get_attr(e, b"vertical-align") {
        let va_enum = match va.as_str() {
            "top" => s1_model::VerticalAlignment::Top,
            "middle" => s1_model::VerticalAlignment::Center,
            "bottom" => s1_model::VerticalAlignment::Bottom,
            _ => s1_model::VerticalAlignment::Top,
        };
        attrs.set(
            AttributeKey::VerticalAlign,
            AttributeValue::VerticalAlignment(va_enum),
        );
    }

    // Background color
    if let Some(bg) = get_attr(e, b"background-color") {
        if bg != "transparent" {
            if let Some(color) = Color::from_hex(bg.trim_start_matches('#')) {
                attrs.set(AttributeKey::CellBackground, AttributeValue::Color(color));
            }
        }
    }

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::events::Event;
    use quick_xml::Reader;

    fn parse_text_attrs(xml: &str) -> AttributeMap {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) | Ok(Event::Empty(e))
                    if e.local_name().as_ref() == b"text-properties" =>
                {
                    return parse_text_properties(&e);
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
        }
        AttributeMap::new()
    }

    fn parse_para_attrs(xml: &str) -> AttributeMap {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) | Ok(Event::Empty(e))
                    if e.local_name().as_ref() == b"paragraph-properties" =>
                {
                    return parse_paragraph_properties(&e);
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
        }
        AttributeMap::new()
    }

    #[test]
    fn parse_bold_italic() {
        let attrs = parse_text_attrs(
            r#"<style:text-properties fo:font-weight="bold" fo:font-style="italic"/>"#,
        );
        assert_eq!(attrs.get_bool(&AttributeKey::Bold), Some(true));
        assert_eq!(attrs.get_bool(&AttributeKey::Italic), Some(true));
    }

    #[test]
    fn parse_font_size_and_family() {
        let attrs = parse_text_attrs(
            r#"<style:text-properties fo:font-size="12pt" style:font-name="Arial"/>"#,
        );
        assert_eq!(attrs.get_f64(&AttributeKey::FontSize), Some(12.0));
        assert_eq!(attrs.get_string(&AttributeKey::FontFamily), Some("Arial"));
    }

    #[test]
    fn parse_color() {
        let attrs = parse_text_attrs("<style:text-properties fo:color=\"#ff0000\"/>");
        match attrs.get(&AttributeKey::Color) {
            Some(AttributeValue::Color(c)) => {
                assert_eq!(c.r, 255);
                assert_eq!(c.g, 0);
            }
            other => panic!("Expected Color, got {:?}", other),
        }
    }

    #[test]
    fn parse_underline() {
        let attrs =
            parse_text_attrs(r#"<style:text-properties style:text-underline-style="solid"/>"#);
        assert!(attrs.get(&AttributeKey::Underline).is_some());
    }

    #[test]
    fn parse_alignment() {
        let attrs = parse_para_attrs(r#"<style:paragraph-properties fo:text-align="center"/>"#);
        match attrs.get(&AttributeKey::Alignment) {
            Some(AttributeValue::Alignment(a)) => assert_eq!(*a, Alignment::Center),
            other => panic!("Expected Alignment, got {:?}", other),
        }
    }

    #[test]
    fn parse_spacing() {
        let attrs = parse_para_attrs(
            r#"<style:paragraph-properties fo:margin-top="0.5cm" fo:margin-bottom="0.3cm"/>"#,
        );
        let before = attrs.get_f64(&AttributeKey::SpacingBefore).unwrap();
        assert!((before - 14.173).abs() < 0.1); // 0.5cm in points
    }

    #[test]
    fn parse_indent() {
        let attrs = parse_para_attrs(
            r#"<style:paragraph-properties fo:margin-left="1in" fo:text-indent="0.5in"/>"#,
        );
        assert!((attrs.get_f64(&AttributeKey::IndentLeft).unwrap() - 72.0).abs() < 0.01);
        assert!((attrs.get_f64(&AttributeKey::IndentFirstLine).unwrap() - 36.0).abs() < 0.01);
    }

    #[test]
    fn parse_line_spacing_percent() {
        let attrs = parse_para_attrs(r#"<style:paragraph-properties fo:line-height="150%"/>"#);
        match attrs.get(&AttributeKey::LineSpacing) {
            Some(AttributeValue::LineSpacing(LineSpacing::Multiple(m))) => {
                assert!((*m - 1.5).abs() < 0.001);
            }
            other => panic!("Expected LineSpacing::Multiple, got {:?}", other),
        }
    }
}
