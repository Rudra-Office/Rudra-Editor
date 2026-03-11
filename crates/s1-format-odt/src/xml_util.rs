//! ODF-specific XML utilities (unit parsing, attribute helpers).

use quick_xml::events::BytesStart;

/// Get an attribute value by local name (ignoring namespace prefix).
pub fn get_attr(e: &BytesStart<'_>, local_name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|attr| attr.key.local_name().as_ref() == local_name)
        .and_then(|attr| std::str::from_utf8(&attr.value).ok().map(|s| s.to_string()))
}

/// Parse a CSS-like length string to points.
///
/// Supported units: `in`, `cm`, `mm`, `pt`, `px`, `pc`.
/// Returns `None` if the string cannot be parsed.
pub fn parse_length(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try common unit suffixes
    if let Some(val) = strip_unit(s, "in") {
        return val.parse::<f64>().ok().map(|v| v * 72.0);
    }
    if let Some(val) = strip_unit(s, "cm") {
        return val.parse::<f64>().ok().map(|v| v * 72.0 / 2.54);
    }
    if let Some(val) = strip_unit(s, "mm") {
        return val.parse::<f64>().ok().map(|v| v * 72.0 / 25.4);
    }
    if let Some(val) = strip_unit(s, "pt") {
        return val.parse::<f64>().ok();
    }
    if let Some(val) = strip_unit(s, "px") {
        return val.parse::<f64>().ok().map(|v| v * 0.75);
    }
    if let Some(val) = strip_unit(s, "pc") {
        return val.parse::<f64>().ok().map(|v| v * 12.0);
    }

    // Bare number — assume inches (common in some generators)
    s.parse::<f64>().ok().map(|v| v * 72.0)
}

fn strip_unit<'a>(s: &'a str, unit: &str) -> Option<&'a str> {
    s.strip_suffix(unit)
}

/// Convert points to centimeters as a string for ODF output.
pub fn points_to_cm(pts: f64) -> String {
    let cm = pts * 2.54 / 72.0;
    format!("{:.3}cm", cm)
}

/// Convert points to inches as a string for ODF output.
#[allow(dead_code)]
pub fn points_to_inch(pts: f64) -> String {
    let inches = pts / 72.0;
    format!("{:.4}in", inches)
}

/// Parse a percentage string like "150%" to a ratio (1.5).
pub fn parse_percentage(s: &str) -> Option<f64> {
    s.trim()
        .strip_suffix('%')?
        .parse::<f64>()
        .ok()
        .map(|v| v / 100.0)
}

/// Escape XML special characters in text content.
pub fn escape_xml(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

/// Get MIME type for a file extension (for images).
pub fn mime_for_extension(ext: &str) -> Option<&'static str> {
    match ext.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "bmp" => Some("image/bmp"),
        "tif" | "tiff" => Some("image/tiff"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

/// Get file extension for a MIME type.
pub fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/tiff" => "tif",
        "image/svg+xml" => "svg",
        _ => "bin",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_length_inches() {
        assert!((parse_length("0.5in").unwrap() - 36.0).abs() < 0.01);
        assert!((parse_length("1in").unwrap() - 72.0).abs() < 0.01);
    }

    #[test]
    fn parse_length_cm() {
        assert!((parse_length("2.54cm").unwrap() - 72.0).abs() < 0.01);
        assert!((parse_length("1cm").unwrap() - 28.3465).abs() < 0.01);
    }

    #[test]
    fn parse_length_mm() {
        assert!((parse_length("25.4mm").unwrap() - 72.0).abs() < 0.01);
    }

    #[test]
    fn parse_length_pt() {
        assert!((parse_length("12pt").unwrap() - 12.0).abs() < 0.01);
    }

    #[test]
    fn parse_length_px() {
        assert!((parse_length("96px").unwrap() - 72.0).abs() < 0.01);
    }

    #[test]
    fn parse_length_invalid() {
        assert!(parse_length("abc").is_none());
        assert!(parse_length("").is_none());
    }

    #[test]
    fn points_to_cm_roundtrip() {
        let cm = points_to_cm(72.0);
        assert!(cm.contains("2.540"));
        assert!(cm.ends_with("cm"));
    }

    #[test]
    fn test_parse_percentage() {
        assert!((parse_percentage("150%").unwrap() - 1.5).abs() < 0.001);
        assert!((parse_percentage("100%").unwrap() - 1.0).abs() < 0.001);
        assert!(parse_percentage("abc").is_none());
    }
}
