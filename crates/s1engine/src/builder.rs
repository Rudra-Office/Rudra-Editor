//! Ergonomic document builder.
//!
//! [`DocumentBuilder`] provides a fluent API for constructing documents
//! without manually creating nodes and operations.
//!
//! # Example
//!
//! ```
//! use s1engine::DocumentBuilder;
//!
//! let doc = DocumentBuilder::new()
//!     .heading(1, "Introduction")
//!     .paragraph(|p| {
//!         p.text("This is ")
//!          .bold("important")
//!          .text(" content.")
//!     })
//!     .build();
//!
//! assert_eq!(doc.to_plain_text(), "Introduction\nThis is important content.");
//! ```

use s1_model::{
    AttributeMap, Color, DocumentModel, Node, NodeId, NodeType, Style, StyleType, UnderlineStyle,
};

use crate::document::Document;

/// Fluent builder for constructing documents.
pub struct DocumentBuilder {
    model: DocumentModel,
}

impl DocumentBuilder {
    /// Create a new builder with an empty document.
    pub fn new() -> Self {
        Self {
            model: DocumentModel::new(),
        }
    }

    /// Add a heading paragraph.
    ///
    /// `level` should be 1-6. The heading is given a style reference
    /// `"Heading{level}"` and a corresponding style is auto-created if
    /// it doesn't already exist.
    pub fn heading(mut self, level: u8, text: &str) -> Self {
        let level = level.clamp(1, 6);
        let style_id = format!("Heading{level}");

        // Auto-create the heading style if it doesn't exist
        if self.model.style_by_id(&style_id).is_none() {
            let name = format!("Heading {level}");
            let font_size = match level {
                1 => 24.0,
                2 => 18.0,
                3 => 14.0,
                _ => 12.0,
            };
            let mut style = Style::new(&style_id, &name, StyleType::Paragraph);
            style.attributes = AttributeMap::new().bold(true).font_size(font_size);
            self.model.set_style(style);
        }

        let body_id = self.model.body_id().unwrap();
        let child_count = self.model.node(body_id).unwrap().children.len();

        let para_id = self.model.next_id();
        let mut para = Node::new(para_id, NodeType::Paragraph);
        para.attributes.set(
            s1_model::AttributeKey::StyleId,
            s1_model::AttributeValue::String(style_id.clone()),
        );
        self.model
            .insert_node(body_id, child_count, para)
            .unwrap();

        let run_id = self.model.next_id();
        let mut run = Node::new(run_id, NodeType::Run);
        let font_size = match level {
            1 => 24.0,
            2 => 18.0,
            3 => 14.0,
            _ => 12.0,
        };
        run.attributes = AttributeMap::new().bold(true).font_size(font_size);
        self.model.insert_node(para_id, 0, run).unwrap();

        let text_id = self.model.next_id();
        self.model
            .insert_node(run_id, 0, Node::text(text_id, text))
            .unwrap();

        self
    }

    /// Add a paragraph built with a [`ParagraphBuilder`].
    pub fn paragraph(mut self, f: impl FnOnce(ParagraphBuilder) -> ParagraphBuilder) -> Self {
        let body_id = self.model.body_id().unwrap();
        let child_count = self.model.node(body_id).unwrap().children.len();

        let para_id = self.model.next_id();
        self.model
            .insert_node(body_id, child_count, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let pb = ParagraphBuilder {
            model: &mut self.model,
            para_id,
        };
        f(pb);

        self
    }

    /// Add a plain text paragraph (shorthand for `.paragraph(|p| p.text(...))`).
    pub fn text(self, text: &str) -> Self {
        let t = text.to_string();
        self.paragraph(move |p| p.text(&t))
    }

    /// Set document title metadata.
    pub fn title(mut self, title: &str) -> Self {
        self.model.metadata_mut().title = Some(title.to_string());
        self
    }

    /// Set document author/creator metadata.
    pub fn author(mut self, author: &str) -> Self {
        self.model.metadata_mut().creator = Some(author.to_string());
        self
    }

    /// Consume the builder and produce a [`Document`].
    pub fn build(self) -> Document {
        Document::from_model(self.model)
    }
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for inline content within a paragraph.
pub struct ParagraphBuilder<'a> {
    model: &'a mut DocumentModel,
    para_id: NodeId,
}

impl<'a> ParagraphBuilder<'a> {
    /// Add a plain text run.
    pub fn text(self, text: &str) -> Self {
        self.run_with_attrs(text, AttributeMap::new())
    }

    /// Add a bold text run.
    pub fn bold(self, text: &str) -> Self {
        self.run_with_attrs(text, AttributeMap::new().bold(true))
    }

    /// Add an italic text run.
    pub fn italic(self, text: &str) -> Self {
        self.run_with_attrs(text, AttributeMap::new().italic(true))
    }

    /// Add a bold+italic text run.
    pub fn bold_italic(self, text: &str) -> Self {
        self.run_with_attrs(text, AttributeMap::new().bold(true).italic(true))
    }

    /// Add an underlined text run.
    pub fn underline(self, text: &str) -> Self {
        let mut attrs = AttributeMap::new();
        attrs.set(
            s1_model::AttributeKey::Underline,
            s1_model::AttributeValue::UnderlineStyle(UnderlineStyle::Single),
        );
        self.run_with_attrs(text, attrs)
    }

    /// Add a text run with a specific font and size.
    pub fn styled(self, text: &str, font: &str, size: f64) -> Self {
        self.run_with_attrs(text, AttributeMap::new().font_family(font).font_size(size))
    }

    /// Add a text run with a specific color.
    pub fn colored(self, text: &str, color: Color) -> Self {
        self.run_with_attrs(text, AttributeMap::new().color(color))
    }

    /// Add a line break within the paragraph.
    pub fn line_break(self) -> Self {
        let child_count = self.model.node(self.para_id).unwrap().children.len();
        let br_id = self.model.next_id();
        self.model
            .insert_node(self.para_id, child_count, Node::new(br_id, NodeType::LineBreak))
            .unwrap();
        self
    }

    /// Add a run with custom attributes.
    fn run_with_attrs(self, text: &str, attrs: AttributeMap) -> Self {
        let child_count = self.model.node(self.para_id).unwrap().children.len();

        let run_id = self.model.next_id();
        let mut run = Node::new(run_id, NodeType::Run);
        run.attributes = attrs;
        self.model
            .insert_node(self.para_id, child_count, run)
            .unwrap();

        let text_id = self.model.next_id();
        self.model
            .insert_node(run_id, 0, Node::text(text_id, text))
            .unwrap();

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_empty_document() {
        let doc = DocumentBuilder::new().build();
        assert_eq!(doc.to_plain_text(), "");
    }

    #[test]
    fn build_single_paragraph() {
        let doc = DocumentBuilder::new()
            .text("Hello World")
            .build();
        assert_eq!(doc.to_plain_text(), "Hello World");
    }

    #[test]
    fn build_heading() {
        let doc = DocumentBuilder::new()
            .heading(1, "Title")
            .build();
        assert_eq!(doc.to_plain_text(), "Title");

        // Should have created a Heading1 style
        assert!(doc.style_by_id("Heading1").is_some());
    }

    #[test]
    fn build_mixed_content() {
        let doc = DocumentBuilder::new()
            .heading(1, "Introduction")
            .paragraph(|p| {
                p.text("This is ")
                    .bold("important")
                    .text(" content.")
            })
            .text("Plain paragraph.")
            .build();

        assert_eq!(
            doc.to_plain_text(),
            "Introduction\nThis is important content.\nPlain paragraph."
        );
        assert_eq!(doc.paragraph_count(), 3);
    }

    #[test]
    fn build_with_formatting() {
        let doc = DocumentBuilder::new()
            .paragraph(|p| {
                p.bold("Bold")
                    .text(" and ")
                    .italic("italic")
                    .text(" and ")
                    .bold_italic("both")
            })
            .build();

        assert_eq!(doc.to_plain_text(), "Bold and italic and both");

        // Check bold attribute on first run
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();
        let bold_run = doc.node(para.children[0]).unwrap();
        assert_eq!(
            bold_run.attributes.get_bool(&s1_model::AttributeKey::Bold),
            Some(true)
        );
    }

    #[test]
    fn build_with_metadata() {
        let doc = DocumentBuilder::new()
            .title("My Report")
            .author("Alice")
            .text("Content")
            .build();

        assert_eq!(doc.metadata().title.as_deref(), Some("My Report"));
        assert_eq!(doc.metadata().creator.as_deref(), Some("Alice"));
    }

    #[test]
    fn build_with_underline() {
        let doc = DocumentBuilder::new()
            .paragraph(|p| p.underline("underlined"))
            .build();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();
        let run = doc.node(para.children[0]).unwrap();
        assert!(run.attributes.get(&s1_model::AttributeKey::Underline).is_some());
    }

    #[test]
    fn build_heading_levels() {
        let doc = DocumentBuilder::new()
            .heading(1, "H1")
            .heading(2, "H2")
            .heading(3, "H3")
            .build();

        assert_eq!(doc.to_plain_text(), "H1\nH2\nH3");
        assert!(doc.style_by_id("Heading1").is_some());
        assert!(doc.style_by_id("Heading2").is_some());
        assert!(doc.style_by_id("Heading3").is_some());
    }

    #[test]
    fn build_with_line_break() {
        let doc = DocumentBuilder::new()
            .paragraph(|p| p.text("Line 1").line_break().text("Line 2"))
            .build();

        // The paragraph has: Run("Line 1"), LineBreak, Run("Line 2")
        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        let para = doc.node(body.children[0]).unwrap();
        assert_eq!(para.children.len(), 3);
        assert_eq!(
            doc.node(para.children[1]).unwrap().node_type,
            NodeType::LineBreak
        );
    }

    #[cfg(feature = "docx")]
    #[test]
    fn build_and_export_docx() {
        let doc = DocumentBuilder::new()
            .title("Builder Test")
            .heading(1, "Hello")
            .paragraph(|p| p.text("World"))
            .build();

        let bytes = doc.export(crate::Format::Docx).unwrap();

        // Re-open and verify
        let engine = crate::Engine::new();
        let doc2 = engine.open(&bytes).unwrap();
        assert_eq!(doc2.to_plain_text(), "Hello\nWorld");
        assert_eq!(doc2.metadata().title.as_deref(), Some("Builder Test"));
    }
}
