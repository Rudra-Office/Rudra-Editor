//! Write s1-model attributes as ODF style property elements.

use s1_model::{
    Alignment, AttributeKey, AttributeMap, AttributeValue, LineSpacing, UnderlineStyle,
    VerticalAlignment,
};

use crate::xml_util::{escape_xml, points_to_cm};

/// Generate `<style:text-properties .../>` from text-level attributes.
///
/// Returns an empty string if no text properties are present.
pub fn write_text_properties(attrs: &AttributeMap) -> String {
    let mut props = Vec::new();

    if let Some(true) = attrs.get_bool(&AttributeKey::Bold) {
        props.push(r#"fo:font-weight="bold""#.to_string());
    }
    if let Some(true) = attrs.get_bool(&AttributeKey::Italic) {
        props.push(r#"fo:font-style="italic""#.to_string());
    }
    if let Some(size) = attrs.get_f64(&AttributeKey::FontSize) {
        props.push(format!(r#"fo:font-size="{size}pt""#));
    }
    if let Some(family) = attrs.get_string(&AttributeKey::FontFamily) {
        props.push(format!(r#"style:font-name="{}""#, escape_xml(family)));
    }
    if let Some(color) = attrs.get_color(&AttributeKey::Color) {
        props.push(format!("fo:color=\"#{}\"", color.to_hex()));
    }
    if let Some(AttributeValue::UnderlineStyle(style)) = attrs.get(&AttributeKey::Underline) {
        let val = match style {
            UnderlineStyle::Single | UnderlineStyle::Thick => "solid",
            UnderlineStyle::Double => "double",
            UnderlineStyle::Dotted => "dotted",
            UnderlineStyle::Dashed => "dash",
            UnderlineStyle::Wave => "wave",
            UnderlineStyle::None => "none",
        };
        if *style != UnderlineStyle::None {
            props.push(format!(r#"style:text-underline-style="{val}""#));
        }
    }
    if let Some(true) = attrs.get_bool(&AttributeKey::Strikethrough) {
        props.push(r#"style:text-line-through-style="solid""#.to_string());
    }
    if let Some(color) = attrs.get_color(&AttributeKey::HighlightColor) {
        props.push(format!("fo:background-color=\"#{}\"", color.to_hex()));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<style:text-properties {}/>", props.join(" "))
    }
}

/// Generate `<style:paragraph-properties .../>` from paragraph-level attributes.
///
/// Returns an empty string if no paragraph properties are present.
pub fn write_paragraph_properties(attrs: &AttributeMap) -> String {
    let mut props = Vec::new();

    if let Some(AttributeValue::Alignment(a)) = attrs.get(&AttributeKey::Alignment) {
        let val = match a {
            Alignment::Left => "start",
            Alignment::Center => "center",
            Alignment::Right => "end",
            Alignment::Justify => "justify",
        };
        props.push(format!(r#"fo:text-align="{val}""#));
    }
    if let Some(pts) = attrs.get_f64(&AttributeKey::SpacingBefore) {
        props.push(format!(r#"fo:margin-top="{}""#, points_to_cm(pts)));
    }
    if let Some(pts) = attrs.get_f64(&AttributeKey::SpacingAfter) {
        props.push(format!(r#"fo:margin-bottom="{}""#, points_to_cm(pts)));
    }
    if let Some(pts) = attrs.get_f64(&AttributeKey::IndentLeft) {
        props.push(format!(r#"fo:margin-left="{}""#, points_to_cm(pts)));
    }
    if let Some(pts) = attrs.get_f64(&AttributeKey::IndentRight) {
        props.push(format!(r#"fo:margin-right="{}""#, points_to_cm(pts)));
    }
    if let Some(pts) = attrs.get_f64(&AttributeKey::IndentFirstLine) {
        props.push(format!(r#"fo:text-indent="{}""#, points_to_cm(pts)));
    }
    if let Some(AttributeValue::LineSpacing(ls)) = attrs.get(&AttributeKey::LineSpacing) {
        match ls {
            LineSpacing::Multiple(m) => {
                let pct = m * 100.0;
                props.push(format!(r#"fo:line-height="{pct:.0}%""#));
            }
            LineSpacing::Exact(pts) => {
                props.push(format!(r#"fo:line-height="{}""#, points_to_cm(*pts)));
            }
            LineSpacing::Single => {
                props.push(r#"fo:line-height="100%""#.to_string());
            }
            LineSpacing::OnePointFive => {
                props.push(r#"fo:line-height="150%""#.to_string());
            }
            LineSpacing::Double => {
                props.push(r#"fo:line-height="200%""#.to_string());
            }
            LineSpacing::AtLeast(pts) => {
                props.push(format!(
                    r#"style:line-height-at-least="{}""#,
                    points_to_cm(*pts)
                ));
            }
        }
    }
    if let Some(true) = attrs.get_bool(&AttributeKey::PageBreakBefore) {
        props.push(r#"fo:break-before="page""#.to_string());
    }
    if let Some(true) = attrs.get_bool(&AttributeKey::KeepWithNext) {
        props.push(r#"fo:keep-with-next="always""#.to_string());
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<style:paragraph-properties {}/>", props.join(" "))
    }
}

/// Generate `<style:table-cell-properties .../>` from cell-level attributes.
///
/// Returns an empty string if no cell properties are present.
pub fn write_table_cell_properties(attrs: &AttributeMap) -> String {
    let mut props = Vec::new();

    if let Some(AttributeValue::VerticalAlignment(va)) = attrs.get(&AttributeKey::VerticalAlign) {
        let val = match va {
            VerticalAlignment::Top => "top",
            VerticalAlignment::Center => "middle",
            VerticalAlignment::Bottom => "bottom",
        };
        props.push(format!(r#"style:vertical-align="{val}""#));
    }
    if let Some(color) = attrs.get_color(&AttributeKey::CellBackground) {
        props.push(format!("fo:background-color=\"#{}\"", color.to_hex()));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<style:table-cell-properties {}/>", props.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::Color;

    #[test]
    fn write_bold_italic() {
        let attrs = AttributeMap::new().bold(true).italic(true);
        let xml = write_text_properties(&attrs);
        assert!(xml.contains(r#"fo:font-weight="bold""#));
        assert!(xml.contains(r#"fo:font-style="italic""#));
    }

    #[test]
    fn write_font_size_and_family() {
        let attrs = AttributeMap::new().font_size(12.0).font_family("Arial");
        let xml = write_text_properties(&attrs);
        assert!(xml.contains(r#"fo:font-size="12pt""#));
        assert!(xml.contains(r#"style:font-name="Arial""#));
    }

    #[test]
    fn write_color() {
        let attrs = AttributeMap::new().color(Color::new(255, 0, 0));
        let xml = write_text_properties(&attrs);
        assert!(xml.contains("fo:color=\"#FF0000\""));
    }

    #[test]
    fn write_alignment_center() {
        let attrs = AttributeMap::new().alignment(Alignment::Center);
        let xml = write_paragraph_properties(&attrs);
        assert!(xml.contains(r#"fo:text-align="center""#));
    }

    #[test]
    fn write_margins() {
        let mut attrs = AttributeMap::new();
        attrs.set(AttributeKey::SpacingBefore, AttributeValue::Float(14.0));
        attrs.set(AttributeKey::SpacingAfter, AttributeValue::Float(7.0));
        let xml = write_paragraph_properties(&attrs);
        assert!(xml.contains("fo:margin-top="));
        assert!(xml.contains("fo:margin-bottom="));
    }

    #[test]
    fn write_empty_attrs() {
        let attrs = AttributeMap::new();
        assert!(write_text_properties(&attrs).is_empty());
        assert!(write_paragraph_properties(&attrs).is_empty());
        assert!(write_table_cell_properties(&attrs).is_empty());
    }

    #[test]
    fn write_line_spacing_percent() {
        let mut attrs = AttributeMap::new();
        attrs.set(
            AttributeKey::LineSpacing,
            AttributeValue::LineSpacing(LineSpacing::Multiple(1.5)),
        );
        let xml = write_paragraph_properties(&attrs);
        assert!(xml.contains(r#"fo:line-height="150%""#));
    }
}
