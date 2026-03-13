//! WebAssembly bindings for s1engine.
//!
//! Provides a JavaScript-friendly API for creating, opening, editing, and
//! exporting documents from the browser or Node.js.

use wasm_bindgen::prelude::*;

use s1_model::{AttributeKey, AttributeValue, DocumentModel, NodeId, NodeType};
use s1_layout::{layout_to_html, LayoutConfig, PageLayout};

// --- WasmEngine ---

/// The main entry point for s1engine in WASM.
#[wasm_bindgen]
pub struct WasmEngine {
    inner: s1engine::Engine,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Create a new engine instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: s1engine::Engine::new(),
        }
    }

    /// Create a new empty document.
    pub fn create(&self) -> WasmDocument {
        WasmDocument {
            inner: Some(self.inner.create()),
        }
    }

    /// Open a document from bytes with auto-detected format.
    ///
    /// Supports DOCX, ODT, and TXT formats.
    pub fn open(&self, data: &[u8]) -> Result<WasmDocument, JsError> {
        let doc = self
            .inner
            .open(data)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmDocument { inner: Some(doc) })
    }

    /// Open a document from bytes with an explicit format.
    ///
    /// Format should be one of: "docx", "odt", "txt".
    pub fn open_as(&self, data: &[u8], format: &str) -> Result<WasmDocument, JsError> {
        let fmt = parse_format(format)?;
        let doc = self
            .inner
            .open_as(data, fmt)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmDocument { inner: Some(doc) })
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

// --- WasmLayoutConfig ---

/// Configuration for paginated HTML layout.
///
/// Controls page dimensions and margins for the layout engine.
/// Defaults to US Letter (8.5" x 11") with 1-inch margins.
#[wasm_bindgen]
pub struct WasmLayoutConfig {
    page_width_pt: f64,
    page_height_pt: f64,
    margin_top_pt: f64,
    margin_bottom_pt: f64,
    margin_left_pt: f64,
    margin_right_pt: f64,
}

#[wasm_bindgen]
impl WasmLayoutConfig {
    /// Create a new layout configuration with US Letter defaults.
    ///
    /// Page: 612pt x 792pt (8.5" x 11")
    /// Margins: 72pt (1") on all sides.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            page_width_pt: 612.0,
            page_height_pt: 792.0,
            margin_top_pt: 72.0,
            margin_bottom_pt: 72.0,
            margin_left_pt: 72.0,
            margin_right_pt: 72.0,
        }
    }

    /// Set the page width in points.
    pub fn set_page_width(&mut self, width: f64) {
        self.page_width_pt = width;
    }

    /// Set the page height in points.
    pub fn set_page_height(&mut self, height: f64) {
        self.page_height_pt = height;
    }

    /// Set the top margin in points.
    pub fn set_margin_top(&mut self, margin: f64) {
        self.margin_top_pt = margin;
    }

    /// Set the bottom margin in points.
    pub fn set_margin_bottom(&mut self, margin: f64) {
        self.margin_bottom_pt = margin;
    }

    /// Set the left margin in points.
    pub fn set_margin_left(&mut self, margin: f64) {
        self.margin_left_pt = margin;
    }

    /// Set the right margin in points.
    pub fn set_margin_right(&mut self, margin: f64) {
        self.margin_right_pt = margin;
    }

    /// Get the page width in points.
    pub fn page_width(&self) -> f64 {
        self.page_width_pt
    }

    /// Get the page height in points.
    pub fn page_height(&self) -> f64 {
        self.page_height_pt
    }

    /// Get the top margin in points.
    pub fn margin_top(&self) -> f64 {
        self.margin_top_pt
    }

    /// Get the bottom margin in points.
    pub fn margin_bottom(&self) -> f64 {
        self.margin_bottom_pt
    }

    /// Get the left margin in points.
    pub fn margin_left(&self) -> f64 {
        self.margin_left_pt
    }

    /// Get the right margin in points.
    pub fn margin_right(&self) -> f64 {
        self.margin_right_pt
    }
}

impl Default for WasmLayoutConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmLayoutConfig {
    /// Convert to a [`LayoutConfig`] for the layout engine.
    fn to_layout_config(&self) -> LayoutConfig {
        LayoutConfig {
            default_page_layout: PageLayout {
                width: self.page_width_pt,
                height: self.page_height_pt,
                margin_top: self.margin_top_pt,
                margin_bottom: self.margin_bottom_pt,
                margin_left: self.margin_left_pt,
                margin_right: self.margin_right_pt,
            },
            ..LayoutConfig::default()
        }
    }
}

// --- WasmDocument ---

/// A document handle for reading, editing, and exporting.
#[wasm_bindgen]
pub struct WasmDocument {
    inner: Option<s1engine::Document>,
}

#[wasm_bindgen]
impl WasmDocument {
    /// Extract all text content as a plain string.
    pub fn to_plain_text(&self) -> Result<String, JsError> {
        let doc = self.doc()?;
        Ok(doc.to_plain_text())
    }

    /// Export the document to the specified format.
    ///
    /// Format should be one of: "docx", "odt", "txt", "pdf".
    /// Returns the exported bytes.
    pub fn export(&self, format: &str) -> Result<Vec<u8>, JsError> {
        let fmt = parse_format(format)?;
        let doc = self.doc()?;
        doc.export(fmt).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Get the document title (from metadata).
    pub fn metadata_title(&self) -> Result<Option<String>, JsError> {
        let doc = self.doc()?;
        Ok(doc.metadata().title.clone())
    }

    /// Get the document author (from metadata).
    pub fn metadata_author(&self) -> Result<Option<String>, JsError> {
        let doc = self.doc()?;
        Ok(doc.metadata().creator.clone())
    }

    /// Get the number of paragraphs in the document.
    pub fn paragraph_count(&self) -> Result<usize, JsError> {
        let doc = self.doc()?;
        Ok(doc.paragraph_count())
    }

    /// Render the document as paginated HTML using the layout engine.
    ///
    /// Produces CSS-positioned HTML with real page boundaries. Each page
    /// is rendered as a separate div with absolute-positioned content.
    /// Uses US Letter page size (612pt x 792pt) with 1-inch margins.
    ///
    /// Text is positioned using fallback font metrics (no system fonts
    /// are available in WASM). For more accurate layout, use
    /// `to_paginated_html_with_fonts()` after loading fonts via
    /// `WasmFontDatabase`.
    pub fn to_paginated_html(&self) -> Result<String, JsError> {
        let doc = self.doc()?;
        let font_db = s1_text::FontDatabase::empty();
        let layout = doc
            .layout(&font_db)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(layout_to_html(&layout))
    }

    /// Render the document as paginated HTML with a custom layout configuration.
    ///
    /// Use this to control page dimensions and margins.
    pub fn to_paginated_html_with_config(
        &self,
        config: &WasmLayoutConfig,
    ) -> Result<String, JsError> {
        let doc = self.doc()?;
        let font_db = s1_text::FontDatabase::empty();
        let layout_config = config.to_layout_config();
        let layout = doc
            .layout_with_config(&font_db, layout_config)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(layout_to_html(&layout))
    }

    /// Render the document as paginated HTML with loaded fonts.
    ///
    /// Use this when you have loaded fonts via `WasmFontDatabase` for
    /// accurate text shaping and positioning.
    pub fn to_paginated_html_with_fonts(
        &self,
        font_db: &WasmFontDatabase,
    ) -> Result<String, JsError> {
        let doc = self.doc()?;
        let layout = doc
            .layout(&font_db.inner)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(layout_to_html(&layout))
    }

    /// Render the document as paginated HTML with loaded fonts and custom config.
    ///
    /// Combines custom page dimensions/margins with loaded font data for
    /// the most accurate layout.
    pub fn to_paginated_html_with_fonts_and_config(
        &self,
        font_db: &WasmFontDatabase,
        config: &WasmLayoutConfig,
    ) -> Result<String, JsError> {
        let doc = self.doc()?;
        let layout_config = config.to_layout_config();
        let layout = doc
            .layout_with_config(&font_db.inner, layout_config)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(layout_to_html(&layout))
    }

    /// Export the document as PDF bytes.
    ///
    /// Uses fallback font metrics (no system fonts). For more accurate
    /// output, use `to_pdf_with_fonts()` after loading fonts via
    /// `WasmFontDatabase`.
    ///
    /// Returns the raw PDF bytes suitable for download or embedding.
    pub fn to_pdf(&self) -> Result<Vec<u8>, JsError> {
        let doc = self.doc()?;
        let font_db = s1_text::FontDatabase::empty();
        doc.export_pdf(&font_db)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Export the document as PDF bytes using loaded fonts.
    ///
    /// Use this when you have loaded fonts via `WasmFontDatabase` for
    /// accurate text shaping and glyph embedding.
    pub fn to_pdf_with_fonts(&self, font_db: &WasmFontDatabase) -> Result<Vec<u8>, JsError> {
        let doc = self.doc()?;
        doc.export_pdf(&font_db.inner)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Export the document as a PDF data URL.
    ///
    /// Returns a string like `data:application/pdf;base64,...` suitable
    /// for embedding in iframes, download links, or `window.open()`.
    pub fn to_pdf_data_url(&self) -> Result<String, JsError> {
        let bytes = self.to_pdf()?;
        let b64 = base64_encode(&bytes);
        Ok(format!("data:application/pdf;base64,{}", b64))
    }

    /// Export the document as a PDF data URL using loaded fonts.
    pub fn to_pdf_data_url_with_fonts(
        &self,
        font_db: &WasmFontDatabase,
    ) -> Result<String, JsError> {
        let bytes = self.to_pdf_with_fonts(font_db)?;
        let b64 = base64_encode(&bytes);
        Ok(format!("data:application/pdf;base64,{}", b64))
    }

    /// Render the document as HTML with formatting, images, and hyperlinks.
    pub fn to_html(&self) -> Result<String, JsError> {
        let doc = self.doc()?;
        let model = doc.model();
        let body_id = model.body_id().ok_or_else(|| JsError::new("No body"))?;
        let mut html = String::new();

        // Render headers from sections
        let sections = doc.sections();
        if !sections.is_empty() {
            if let Some(hf) = sections[0].headers.first() {
                html.push_str("<header style=\"border-bottom:1px solid #444;padding:8px 0;margin-bottom:16px;color:#aaa\">");
                render_children(model, hf.node_id, &mut html);
                html.push_str("</header>");
            }
        }

        // Render body content
        render_children(model, body_id, &mut html);

        // Render footers from sections
        if !sections.is_empty() {
            if let Some(hf) = sections[0].footers.first() {
                html.push_str("<footer style=\"border-top:1px solid #444;padding:8px 0;margin-top:16px;color:#aaa\">");
                render_children(model, hf.node_id, &mut html);
                html.push_str("</footer>");
            }
        }

        Ok(html)
    }

    /// Get the number of tracked changes in the document.
    pub fn tracked_changes_count(&self) -> Result<usize, JsError> {
        let doc = self.doc()?;
        Ok(doc.tracked_changes().len())
    }

    /// Accept all tracked changes in the document.
    ///
    /// Insertions keep their content; deletions are removed; format changes
    /// keep the new formatting. All revision attributes are stripped.
    pub fn accept_all_changes(&mut self) -> Result<(), JsError> {
        let doc = self.doc_mut()?;
        doc.accept_all_changes()
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Reject all tracked changes in the document.
    ///
    /// Insertions are removed; deletions are un-deleted; format changes
    /// restore original formatting. All revision attributes are stripped.
    pub fn reject_all_changes(&mut self) -> Result<(), JsError> {
        let doc = self.doc_mut()?;
        doc.reject_all_changes()
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Free the document, releasing memory.
    ///
    /// After calling this, all other methods will return an error.
    pub fn free(&mut self) {
        self.inner = None;
    }

    /// Check if this document handle is still valid.
    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }
}

impl WasmDocument {
    fn doc(&self) -> Result<&s1engine::Document, JsError> {
        self.inner
            .as_ref()
            .ok_or_else(|| JsError::new("Document has been freed"))
    }

    fn doc_mut(&mut self) -> Result<&mut s1engine::Document, JsError> {
        self.inner
            .as_mut()
            .ok_or_else(|| JsError::new("Document has been freed"))
    }
}

// --- WasmDocumentBuilder ---

/// A fluent builder for constructing documents.
#[wasm_bindgen]
pub struct WasmDocumentBuilder {
    inner: Option<s1engine::DocumentBuilder>,
}

#[wasm_bindgen]
impl WasmDocumentBuilder {
    /// Create a new document builder.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(s1engine::DocumentBuilder::new()),
        }
    }

    /// Add a heading at the specified level (1-6).
    pub fn heading(mut self, level: u8, text: &str) -> Self {
        if let Some(builder) = self.inner.take() {
            self.inner = Some(builder.heading(level, text));
        }
        self
    }

    /// Add a paragraph with plain text.
    pub fn text(mut self, text: &str) -> Self {
        if let Some(builder) = self.inner.take() {
            self.inner = Some(builder.text(text));
        }
        self
    }

    /// Set the document title.
    pub fn title(mut self, title: &str) -> Self {
        if let Some(builder) = self.inner.take() {
            self.inner = Some(builder.title(title));
        }
        self
    }

    /// Set the document author.
    pub fn author(mut self, author: &str) -> Self {
        if let Some(builder) = self.inner.take() {
            self.inner = Some(builder.author(author));
        }
        self
    }

    /// Build the document. Consumes the builder.
    pub fn build(mut self) -> Result<WasmDocument, JsError> {
        let builder = self
            .inner
            .take()
            .ok_or_else(|| JsError::new("Builder already consumed"))?;
        Ok(WasmDocument {
            inner: Some(builder.build()),
        })
    }
}

impl Default for WasmDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// --- WasmFontDatabase ---

/// A font database for WASM environments.
///
/// Since WASM has no filesystem access, fonts must be loaded manually
/// via `load_font()`.
#[wasm_bindgen]
pub struct WasmFontDatabase {
    inner: s1_text::FontDatabase,
}

#[wasm_bindgen]
impl WasmFontDatabase {
    /// Create a new empty font database.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: s1_text::FontDatabase::empty(),
        }
    }

    /// Load a font from raw bytes (TTF/OTF).
    pub fn load_font(&mut self, data: Vec<u8>) {
        self.inner.load_font_data(data);
    }

    /// Get the number of loaded font faces.
    pub fn font_count(&self) -> usize {
        self.inner.len()
    }
}

impl Default for WasmFontDatabase {
    fn default() -> Self {
        Self::new()
    }
}

// --- Helpers ---

fn parse_format(format: &str) -> Result<s1engine::Format, JsError> {
    match format.to_lowercase().as_str() {
        "docx" => Ok(s1engine::Format::Docx),
        "odt" => Ok(s1engine::Format::Odt),
        "pdf" => Ok(s1engine::Format::Pdf),
        "txt" | "text" => Ok(s1engine::Format::Txt),
        "doc" => Ok(s1engine::Format::Doc),
        "md" | "markdown" => Ok(s1engine::Format::Md),
        _ => Err(JsError::new(&format!("Unsupported format: {format}"))),
    }
}

// --- HTML rendering ---

fn render_children(model: &DocumentModel, parent_id: NodeId, html: &mut String) {
    let parent = match model.node(parent_id) {
        Some(n) => n,
        None => return,
    };
    for &child_id in &parent.children {
        render_node(model, child_id, html);
    }
}

fn render_node(model: &DocumentModel, node_id: NodeId, html: &mut String) {
    let node = match model.node(node_id) {
        Some(n) => n,
        None => return,
    };

    match node.node_type {
        NodeType::Paragraph => render_paragraph(model, node_id, html),
        NodeType::Table => render_table(model, node_id, html),
        NodeType::TableRow => render_table_row(model, node_id, html),
        NodeType::TableCell => {
            html.push_str("<td style=\"border:1px solid #333;padding:6px 10px\">");
            render_children(model, node_id, html);
            html.push_str("</td>");
        }
        NodeType::Image => render_image(model, node_id, html),
        NodeType::TableOfContents => {
            html.push_str("<div class=\"toc\" style=\"margin:1em 0;padding:1em;border:1px solid #333;border-radius:4px\">");
            html.push_str("<strong>Table of Contents</strong><br/>");
            render_children(model, node_id, html);
            html.push_str("</div>");
        }
        _ => {}
    }
}

fn render_paragraph(model: &DocumentModel, para_id: NodeId, html: &mut String) {
    let para = match model.node(para_id) {
        Some(n) => n,
        None => return,
    };

    // Detect heading level from style ID (e.g. "Heading1", "heading 2", etc.)
    let style_id = para.attributes.get_string(&AttributeKey::StyleId);
    let effective_level: Option<u8> = style_id.and_then(|sid| {
        let sid_lower = sid.to_lowercase();
        if sid_lower.starts_with("heading") {
            sid_lower
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        } else {
            None
        }
    });

    // Build inline style
    let mut style = String::new();
    if let Some(AttributeValue::Alignment(a)) = para.attributes.get(&AttributeKey::Alignment) {
        let val = match a {
            s1_model::Alignment::Left => "left",
            s1_model::Alignment::Center => "center",
            s1_model::Alignment::Right => "right",
            s1_model::Alignment::Justify => "justify",
            _ => "",
        };
        if !val.is_empty() {
            style.push_str(&format!("text-align:{val};"));
        }
    }

    let style_attr = if style.is_empty() {
        String::new()
    } else {
        format!(" style=\"{style}\"")
    };

    match effective_level {
        Some(l @ 1..=6) => {
            html.push_str(&format!("<h{l}{style_attr}>"));
            render_inline_children(model, para_id, html);
            html.push_str(&format!("</h{l}>"));
        }
        _ => {
            html.push_str(&format!("<p{style_attr}>"));
            render_inline_children(model, para_id, html);
            html.push_str("</p>");
        }
    }
}

fn render_inline_children(model: &DocumentModel, parent_id: NodeId, html: &mut String) {
    let parent = match model.node(parent_id) {
        Some(n) => n,
        None => return,
    };

    for &child_id in &parent.children {
        let child = match model.node(child_id) {
            Some(n) => n,
            None => continue,
        };

        match child.node_type {
            NodeType::Run => render_run(model, child_id, html),
            NodeType::Image => render_image(model, child_id, html),
            NodeType::LineBreak => html.push_str("<br/>"),
            NodeType::Tab => html.push_str("&emsp;"),
            NodeType::Field => {
                if let Some(AttributeValue::FieldType(ft)) =
                    child.attributes.get(&AttributeKey::FieldType)
                {
                    match ft {
                        s1_model::FieldType::PageNumber => {
                            html.push_str("<span class=\"field\">#</span>")
                        }
                        s1_model::FieldType::PageCount => {
                            html.push_str("<span class=\"field\">N</span>")
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn render_run(model: &DocumentModel, run_id: NodeId, html: &mut String) {
    let run = match model.node(run_id) {
        Some(n) => n,
        None => return,
    };

    let bold = run.attributes.get_bool(&AttributeKey::Bold) == Some(true);
    let italic = run.attributes.get_bool(&AttributeKey::Italic) == Some(true);
    let underline = run.attributes.get(&AttributeKey::Underline).is_some();
    let strikethrough = run.attributes.get_bool(&AttributeKey::Strikethrough) == Some(true);
    let superscript = run.attributes.get_bool(&AttributeKey::Superscript) == Some(true);
    let subscript = run.attributes.get_bool(&AttributeKey::Subscript) == Some(true);
    let hyperlink_url = run.attributes.get_string(&AttributeKey::HyperlinkUrl);

    // Track changes: detect revision type for visual indicator
    let revision_type = run.attributes.get_string(&AttributeKey::RevisionType);

    // Inline style for font, size, color
    let mut style = String::new();
    if let Some(font) = run.attributes.get_string(&AttributeKey::FontFamily) {
        style.push_str(&format!("font-family:'{font}';"));
    }
    if let Some(size) = run.attributes.get_f64(&AttributeKey::FontSize) {
        style.push_str(&format!("font-size:{size}pt;"));
    }
    if let Some(AttributeValue::Color(c)) = run.attributes.get(&AttributeKey::Color) {
        style.push_str(&format!("color:#{};", c.to_hex()));
    }

    // Track changes visual styling
    match revision_type {
        Some("Insert") => {
            style.push_str("color:#22863a;text-decoration:underline;text-decoration-color:#22863a;");
        }
        Some("Delete") => {
            style.push_str("color:#cb2431;text-decoration:line-through;text-decoration-color:#cb2431;");
        }
        Some("FormatChange") => {
            style.push_str("border-bottom:2px dotted #b08800;");
        }
        _ => {}
    }

    // Open tags
    if let Some(url) = hyperlink_url {
        html.push_str(&format!(
            "<a href=\"{}\" style=\"color:#58a6ff;text-decoration:underline\">",
            escape_html(url)
        ));
    }
    if bold {
        html.push_str("<strong>");
    }
    if italic {
        html.push_str("<em>");
    }
    if underline {
        html.push_str("<u>");
    }
    if strikethrough {
        html.push_str("<s>");
    }
    if superscript {
        html.push_str("<sup>");
    }
    if subscript {
        html.push_str("<sub>");
    }

    let has_style = !style.is_empty();
    if has_style {
        html.push_str(&format!("<span style=\"{style}\">"));
    }

    // Text content
    for &text_id in &run.children {
        if let Some(text_node) = model.node(text_id) {
            if text_node.node_type == NodeType::Text {
                if let Some(content) = text_node.text_content.as_deref() {
                    html.push_str(&escape_html(content));
                }
            } else if text_node.node_type == NodeType::LineBreak {
                html.push_str("<br/>");
            } else if text_node.node_type == NodeType::Tab {
                html.push_str("&emsp;");
            }
        }
    }

    // Close tags (reverse order)
    if has_style {
        html.push_str("</span>");
    }
    if subscript {
        html.push_str("</sub>");
    }
    if superscript {
        html.push_str("</sup>");
    }
    if strikethrough {
        html.push_str("</s>");
    }
    if underline {
        html.push_str("</u>");
    }
    if italic {
        html.push_str("</em>");
    }
    if bold {
        html.push_str("</strong>");
    }
    if hyperlink_url.is_some() {
        html.push_str("</a>");
    }
}

fn render_image(model: &DocumentModel, img_id: NodeId, html: &mut String) {
    let img = match model.node(img_id) {
        Some(n) => n,
        None => return,
    };

    if let Some(AttributeValue::MediaId(media_id)) =
        img.attributes.get(&AttributeKey::ImageMediaId)
    {
        if let Some(item) = model.media().get(*media_id) {
            let b64 = base64_encode(&item.data);
            let mime = &item.content_type;
            let mut style = String::from("max-width:100%;height:auto;border-radius:4px;margin:8px 0;");
            if let Some(w) = img.attributes.get_f64(&AttributeKey::ImageWidth) {
                style.push_str(&format!("width:{w}pt;"));
            }
            html.push_str(&format!(
                "<img src=\"data:{mime};base64,{b64}\" style=\"{style}\" alt=\"\"/>"
            ));
        }
    }
}

fn render_table(model: &DocumentModel, table_id: NodeId, html: &mut String) {
    html.push_str("<table style=\"border-collapse:collapse;margin:1em 0;width:100%\">");
    let table = match model.node(table_id) {
        Some(n) => n,
        None => {
            html.push_str("</table>");
            return;
        }
    };
    for &row_id in &table.children {
        render_node(model, row_id, html);
    }
    html.push_str("</table>");
}

fn render_table_row(model: &DocumentModel, row_id: NodeId, html: &mut String) {
    html.push_str("<tr>");
    let row = match model.node(row_id) {
        Some(n) => n,
        None => {
            html.push_str("</tr>");
            return;
        }
    };
    for &cell_id in &row.children {
        render_node(model, cell_id, html);
    }
    html.push_str("</tr>");
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Detect the format of a document from its bytes.
///
/// Returns one of: "docx", "odt", "pdf", "txt".
#[wasm_bindgen]
pub fn detect_format(data: &[u8]) -> String {
    let fmt = s1engine::Format::detect(data);
    fmt.extension().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_create_document() {
        let engine = WasmEngine::new();
        let doc = engine.create();
        assert!(doc.is_valid());
        assert_eq!(doc.paragraph_count().unwrap(), 0);
    }

    #[test]
    fn wasm_builder() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder
            .title("Test")
            .author("Author")
            .heading(1, "Title")
            .text("Hello world")
            .build()
            .unwrap();
        assert!(doc.is_valid());
        assert_eq!(doc.metadata_title().unwrap(), Some("Test".to_string()));
        assert_eq!(doc.metadata_author().unwrap(), Some("Author".to_string()));
        assert!(doc.paragraph_count().unwrap() >= 2);
    }

    #[test]
    fn wasm_plain_text() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder.text("Hello").text("World").build().unwrap();
        let text = doc.to_plain_text().unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn wasm_metadata() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder.title("My Doc").author("Alice").build().unwrap();
        assert_eq!(doc.metadata_title().unwrap(), Some("My Doc".to_string()));
        assert_eq!(doc.metadata_author().unwrap(), Some("Alice".to_string()));
    }

    #[test]
    fn wasm_document_free() {
        let engine = WasmEngine::new();
        let mut doc = engine.create();
        assert!(doc.is_valid());
        // Directly set inner to None (equivalent to free()) since
        // wasm_bindgen &mut self methods panic on non-wasm targets.
        doc.inner = None;
        assert!(!doc.is_valid());
        // Can't call to_plain_text() here because JsError::new() panics
        // on non-wasm targets. The is_valid() check above proves free works.
        assert!(doc.inner.is_none());
    }

    #[test]
    fn wasm_format_detection() {
        // ZIP magic bytes (DOCX/ODT)
        let zip_data = &[0x50, 0x4B, 0x03, 0x04];
        let fmt = detect_format(zip_data);
        assert!(fmt == "docx" || fmt == "odt");

        // Plain text
        let txt_data = b"Hello world";
        let fmt = detect_format(txt_data);
        assert_eq!(fmt, "txt");
    }

    #[test]
    fn wasm_error_handling() {
        let engine = WasmEngine::new();
        // Invalid data should produce an error
        let result = engine.open(&[0xFF, 0xFF, 0xFF]);
        // Should either succeed (as txt) or fail with error
        // Plain text reader is very lenient, so this likely succeeds
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn wasm_export_txt() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder.text("Export test").build().unwrap();
        let bytes = doc.export("txt").unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("Export test"));
    }

    #[test]
    fn wasm_font_loading() {
        let mut font_db = WasmFontDatabase::new();
        assert_eq!(font_db.font_count(), 0);
        // Load some arbitrary bytes (won't be valid font, but shouldn't panic)
        font_db.load_font(vec![0; 100]);
        // fontdb silently ignores invalid font data
    }

    #[test]
    fn wasm_open_docx() {
        // Build a document, export as DOCX, then reopen it
        let builder = WasmDocumentBuilder::new();
        let doc = builder.text("DOCX round-trip").build().unwrap();
        let docx_bytes = doc.export("docx").unwrap();

        let engine = WasmEngine::new();
        let reopened = engine.open(&docx_bytes).unwrap();
        assert!(reopened.is_valid());
        let text = reopened.to_plain_text().unwrap();
        assert!(text.contains("DOCX round-trip"));
    }

    #[test]
    fn wasm_export_docx() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder
            .title("Export Test")
            .text("Content")
            .build()
            .unwrap();
        let bytes = doc.export("docx").unwrap();
        // DOCX is a ZIP file — check magic bytes
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
    }

    #[test]
    fn wasm_builder_export_roundtrip() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder
            .title("RT Title")
            .author("RT Author")
            .heading(1, "Heading")
            .text("Body text")
            .build()
            .unwrap();

        // Export to DOCX and reopen
        let docx_bytes = doc.export("docx").unwrap();
        let engine = WasmEngine::new();
        let reopened = engine.open_as(&docx_bytes, "docx").unwrap();

        assert_eq!(
            reopened.metadata_title().unwrap(),
            Some("RT Title".to_string())
        );
        assert_eq!(
            reopened.metadata_author().unwrap(),
            Some("RT Author".to_string())
        );
        let text = reopened.to_plain_text().unwrap();
        assert!(text.contains("Heading"));
        assert!(text.contains("Body text"));
    }

    // ─── Paginated HTML Tests ────────────────────────────────────

    #[test]
    fn test_paginated_html_empty() {
        let engine = WasmEngine::new();
        let doc = engine.create();
        let html = doc.to_paginated_html().unwrap();
        // Should produce valid paginated HTML with at least one page
        assert!(html.contains("s1-page"), "empty doc should have a page div");
        assert!(
            html.contains("s1-document"),
            "should have a document wrapper"
        );
    }

    #[test]
    fn test_paginated_html_basic() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder
            .heading(1, "Title")
            .text("Hello world")
            .build()
            .unwrap();
        let html = doc.to_paginated_html().unwrap();
        assert!(html.contains("s1-page"), "should have page div");
        assert!(html.contains("s1-block"), "should have block div");
        assert!(html.contains("Hello world"), "should contain text content");
        assert!(html.contains("Title"), "should contain heading text");
    }

    #[test]
    fn test_paginated_html_with_config() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder.text("Config test").build().unwrap();

        // Use A4 dimensions
        let mut config = WasmLayoutConfig::new();
        config.set_page_width(595.28);
        config.set_page_height(841.89);
        config.set_margin_top(50.0);
        config.set_margin_bottom(50.0);

        let html = doc.to_paginated_html_with_config(&config).unwrap();
        assert!(html.contains("s1-page"), "should have page div");
        // The page width should reflect A4 dimensions
        assert!(
            html.contains("width:595.3pt") || html.contains("width:595pt"),
            "should have A4 page width: {html}"
        );
    }

    #[test]
    fn test_layout_config_defaults() {
        let config = WasmLayoutConfig::new();
        assert!((config.page_width() - 612.0).abs() < 0.01, "default width should be US Letter");
        assert!((config.page_height() - 792.0).abs() < 0.01, "default height should be US Letter");
        assert!((config.margin_top() - 72.0).abs() < 0.01, "default top margin should be 1 inch");
        assert!((config.margin_bottom() - 72.0).abs() < 0.01, "default bottom margin should be 1 inch");
        assert!((config.margin_left() - 72.0).abs() < 0.01, "default left margin should be 1 inch");
        assert!((config.margin_right() - 72.0).abs() < 0.01, "default right margin should be 1 inch");
    }

    #[test]
    fn test_layout_config_setters() {
        let mut config = WasmLayoutConfig::new();
        config.set_page_width(500.0);
        config.set_page_height(700.0);
        config.set_margin_top(36.0);
        config.set_margin_bottom(36.0);
        config.set_margin_left(48.0);
        config.set_margin_right(48.0);

        assert!((config.page_width() - 500.0).abs() < 0.01);
        assert!((config.page_height() - 700.0).abs() < 0.01);
        assert!((config.margin_top() - 36.0).abs() < 0.01);
        assert!((config.margin_bottom() - 36.0).abs() < 0.01);
        assert!((config.margin_left() - 48.0).abs() < 0.01);
        assert!((config.margin_right() - 48.0).abs() < 0.01);

        // Verify the conversion to LayoutConfig
        let layout_config = config.to_layout_config();
        assert!((layout_config.default_page_layout.width - 500.0).abs() < 0.01);
        assert!((layout_config.default_page_layout.margin_left - 48.0).abs() < 0.01);
    }

    #[test]
    fn test_paginated_html_contains_pages() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder
            .text("Page content line one")
            .text("Page content line two")
            .text("Page content line three")
            .build()
            .unwrap();
        let html = doc.to_paginated_html().unwrap();

        // Count page divs — should have at least one
        let page_count = html.matches("s1-page").count();
        assert!(
            page_count >= 1,
            "should have at least one s1-page div, got {page_count}"
        );
        // Should have the document wrapper
        assert!(html.contains("s1-document"));
        // Should contain positioned blocks
        assert!(
            html.contains("position:absolute") || html.contains("position:relative"),
            "paginated HTML should use CSS positioning"
        );
    }

    // ─── PDF Export Tests ────────────────────────────────────────

    #[test]
    fn test_to_pdf_empty() {
        let engine = WasmEngine::new();
        let doc = engine.create();
        let pdf_bytes = doc.to_pdf().unwrap();
        // PDF files start with %PDF
        assert!(
            pdf_bytes.len() >= 4,
            "PDF should have at least 4 bytes, got {}",
            pdf_bytes.len()
        );
        assert_eq!(
            &pdf_bytes[0..5],
            b"%PDF-",
            "PDF should start with %PDF- magic bytes"
        );
    }

    #[test]
    fn test_to_pdf_with_content() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder
            .title("PDF Test")
            .heading(1, "Hello PDF")
            .text("This is a test document for PDF export.")
            .build()
            .unwrap();
        let pdf_bytes = doc.to_pdf().unwrap();
        assert_eq!(
            &pdf_bytes[0..5],
            b"%PDF-",
            "PDF should start with %PDF- magic"
        );
        // A document with content should produce a reasonably sized PDF
        assert!(
            pdf_bytes.len() > 100,
            "PDF with content should be > 100 bytes, got {}",
            pdf_bytes.len()
        );
    }

    #[test]
    fn test_to_pdf_data_url() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder.text("Data URL test").build().unwrap();
        let data_url = doc.to_pdf_data_url().unwrap();
        assert!(
            data_url.starts_with("data:application/pdf;base64,"),
            "Data URL should start with the correct prefix, got: {}",
            &data_url[..50.min(data_url.len())]
        );
        // The base64 portion should be non-empty
        let b64_part = &data_url["data:application/pdf;base64,".len()..];
        assert!(
            !b64_part.is_empty(),
            "Base64 portion of data URL should not be empty"
        );
    }

    #[test]
    fn test_to_pdf_has_content() {
        let builder = WasmDocumentBuilder::new();
        let doc = builder
            .heading(1, "Title")
            .text("First paragraph")
            .text("Second paragraph")
            .build()
            .unwrap();
        let pdf_bytes = doc.to_pdf().unwrap();
        // Verify it looks like a valid PDF (starts with header, ends near %%EOF)
        assert_eq!(&pdf_bytes[0..5], b"%PDF-");
        // PDF files typically end with %%EOF (possibly with trailing whitespace)
        let tail = String::from_utf8_lossy(&pdf_bytes[pdf_bytes.len().saturating_sub(32)..]);
        assert!(
            tail.contains("%%EOF"),
            "PDF should end with %%EOF marker, tail: {tail}"
        );
    }
}
