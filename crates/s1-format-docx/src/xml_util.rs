//! XML parsing utilities for OOXML documents.

use quick_xml::events::BytesStart;

/// Get an attribute value by local name (ignoring namespace prefix).
///
/// For `<w:jc w:val="center"/>`, `get_attr(e, b"val")` returns `Some("center")`.
pub fn get_attr(e: &BytesStart<'_>, local_name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|attr| attr.key.local_name().as_ref() == local_name)
        .and_then(|attr| std::str::from_utf8(&attr.value).ok().map(|s| s.to_string()))
}

/// Get the `w:val` attribute (the most common OOXML attribute).
pub fn get_val(e: &BytesStart<'_>) -> Option<String> {
    get_attr(e, b"val")
}

/// Check if a toggle property is "on".
///
/// In OOXML, toggle properties like `<w:b/>` mean `true` by presence.
/// `<w:b w:val="false"/>` or `<w:b w:val="0"/>` means `false`.
pub fn is_toggle_on(e: &BytesStart<'_>) -> bool {
    match get_val(e) {
        None => true, // presence without val = true
        Some(v) => v != "false" && v != "0",
    }
}

/// Parse a twips value (twentieths of a point) to points.
/// 1 inch = 1440 twips = 72 points.
pub fn twips_to_points(twips: &str) -> Option<f64> {
    twips.parse::<f64>().ok().map(|t| t / 20.0)
}

/// Parse half-points to points.
/// Font sizes in OOXML are stored in half-points (24 = 12pt).
pub fn half_points_to_points(half_pts: &str) -> Option<f64> {
    half_pts.parse::<f64>().ok().map(|hp| hp / 2.0)
}

/// EMU (English Metric Units) per point. 1 inch = 914400 EMU = 72 points.
const EMU_PER_POINT: f64 = 12700.0;

/// Parse EMU string to points.
pub fn emu_to_points(emu: &str) -> Option<f64> {
    emu.parse::<f64>().ok().map(|e| e / EMU_PER_POINT)
}

/// Convert points to EMU.
pub fn points_to_emu(pts: f64) -> i64 {
    (pts * EMU_PER_POINT) as i64
}

/// Get MIME type for a file extension.
pub fn mime_for_extension(ext: &str) -> Option<&'static str> {
    match ext.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "bmp" => Some("image/bmp"),
        "tif" | "tiff" => Some("image/tiff"),
        "svg" => Some("image/svg+xml"),
        "emf" => Some("image/x-emf"),
        "wmf" => Some("image/x-wmf"),
        _ => None,
    }
}

/// Get a file extension for a MIME type.
pub fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/tiff" => "tif",
        "image/svg+xml" => "svg",
        "image/x-emf" => "emf",
        "image/x-wmf" => "wmf",
        _ => "bin",
    }
}
