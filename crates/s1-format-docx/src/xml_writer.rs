//! Simple XML string builder for generating OOXML output.

/// Escape special XML characters in text content.
pub fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_basic() {
        assert_eq!(escape_xml("Hello World"), "Hello World");
    }

    #[test]
    fn escape_special_chars() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml(r#"say "hi""#), "say &quot;hi&quot;");
    }

    #[test]
    fn escape_empty() {
        assert_eq!(escape_xml(""), "");
    }
}
