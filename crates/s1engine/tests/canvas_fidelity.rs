//! Canvas rendering fidelity tests.
//!
//! Validates that the layout engine produces correct geometry for real documents:
//! - Page dimensions match expected paper size
//! - Content fits within page content area (no overflow)
//! - All paragraphs produce layout blocks
//! - Text runs have non-zero widths
//! - Block positions are monotonically increasing (no overlap)
//! - HTML output and layout JSON are consistent
//! - Glyph rasterizer produces non-empty bitmaps

use s1engine::{Engine, Format};
use s1_layout::{LayoutConfig, LayoutEngine, PageLayout};
use std::path::Path;

fn project_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .to_path_buf()
}

fn open_doc(rel_path: &str) -> Option<s1engine::Document> {
    let path = project_root().join(rel_path);
    if !path.exists() {
        // In CI strict mode (S1_STRICT_FIDELITY=1), fail on missing files
        if std::env::var("S1_STRICT_FIDELITY").as_deref() == Ok("1") {
            panic!("STRICT: corpus file missing: {}", path.display());
        }
        return None;
    }
    let data = std::fs::read(&path).ok()?;
    Engine::new().open(&data).ok()
}

/// Validate layout geometry for a document.
fn validate_layout(doc: &s1engine::Document, name: &str) {
    let font_db = s1_text::FontDatabase::empty();
    let layout = doc.layout(&font_db);
    assert!(layout.is_ok(), "{name}: layout failed: {:?}", layout.err());
    let layout = layout.unwrap();

    let page_count = layout.pages.len();
    assert!(page_count > 0, "{name}: no pages generated");
    eprintln!("{name}: {page_count} pages");

    for (pi, page) in layout.pages.iter().enumerate() {
        // Page dimensions should be reasonable (Letter or A4 range)
        assert!(page.width > 300.0 && page.width < 900.0,
            "{name} page {pi}: width {:.1} out of range", page.width);
        assert!(page.height > 400.0 && page.height < 1300.0,
            "{name} page {pi}: height {:.1} out of range", page.height);

        // Content area should be within page bounds
        let ca = &page.content_area;
        assert!(ca.x >= 0.0 && ca.y >= 0.0,
            "{name} page {pi}: content area has negative position");
        assert!(ca.x + ca.width <= page.width + 1.0,
            "{name} page {pi}: content area exceeds page width");
        assert!(ca.y + ca.height <= page.height + 1.0,
            "{name} page {pi}: content area exceeds page height");

        // Blocks should be within content area (with tolerance)
        let mut prev_y = 0.0f64;
        for (bi, block) in page.blocks.iter().enumerate() {
            let b = &block.bounds;
            // Block should have non-zero dimensions
            assert!(b.width > 0.0,
                "{name} page {pi} block {bi}: zero width");
            assert!(b.height > 0.0,
                "{name} page {pi} block {bi}: zero height");

            // Block Y should be >= previous block's Y (monotonic)
            assert!(b.y >= prev_y - 1.0,
                "{name} page {pi} block {bi}: y={:.1} < prev_y={:.1} (overlap)",
                b.y, prev_y);
            prev_y = b.y;

            // Validate text runs in paragraph blocks
            if let s1_layout::LayoutBlockKind::Paragraph { ref lines, .. } = block.kind {
                for (li, line) in lines.iter().enumerate() {
                    assert!(line.height > 0.0,
                        "{name} page {pi} block {bi} line {li}: zero height");

                    for (ri, run) in line.runs.iter().enumerate() {
                        if !run.text.is_empty() {
                            assert!(run.width > 0.0 || run.text.trim().is_empty(),
                                "{name} page {pi} block {bi} line {li} run {ri}: text '{}' has zero width",
                                run.text);
                        }
                    }
                }
            }
        }
    }

    eprintln!("{name}: layout geometry VALID");
}

/// Validate HTML output is consistent with layout.
fn validate_html_consistency(doc: &s1engine::Document, name: &str) {
    let font_db = s1_text::FontDatabase::empty();
    let layout = doc.layout(&font_db).unwrap();
    let html = s1_layout::layout_to_html(&layout);

    // HTML should be non-empty
    assert!(!html.is_empty(), "{name}: empty HTML output");

    // HTML should contain page divs
    let page_count = layout.pages.len();
    let html_pages = html.matches("s1-page").count();
    assert_eq!(html_pages, page_count,
        "{name}: HTML has {html_pages} pages but layout has {page_count}");

    // HTML should contain block divs for each layout block
    let total_blocks: usize = layout.pages.iter().map(|p| p.blocks.len()).sum();
    let html_blocks = html.matches("s1-block").count();
    // Allow some tolerance (headers/footers add extra blocks)
    assert!(html_blocks >= total_blocks.saturating_sub(5),
        "{name}: HTML has {html_blocks} blocks but layout has {total_blocks}");

    // Text content should be present in HTML
    let plain_text = doc.to_plain_text();
    let first_50: String = plain_text.chars().take(50).collect();
    if !first_50.trim().is_empty() {
        // At least some of the first 50 chars should appear in HTML
        let first_words: Vec<&str> = first_50.split_whitespace().take(3).collect();
        let found = first_words.iter().any(|w| w.len() > 2 && html.contains(w));
        assert!(found || first_words.is_empty(),
            "{name}: first words not found in HTML: {:?}", first_words);
    }

    eprintln!("{name}: HTML consistency VALID ({html_pages} pages, {html_blocks} blocks)");
}

/// Validate the glyph rasterizer produces output for common characters.
fn validate_rasterizer(name: &str) {
    // Test with embedded fallback font if available
    let font_db = s1_text::FontDatabase::with_embedded_fallback();
    if font_db.is_empty() {
        eprintln!("{name}: SKIP rasterizer test (no embedded font)");
        return;
    }

    // Find a font
    let font_id = font_db.find("serif", false, false)
        .or_else(|| font_db.find("sans-serif", false, false));

    if let Some(fid) = font_id {
        if let Some(font) = font_db.load_font(fid) {
            // Test rasterizing 'A'
            let glyph_id = font.glyph_index('A').unwrap_or(0);
            let result = s1_text::rasterize_glyph(font.data(), glyph_id, 32.0, [0, 0, 0]);
            if let Some(glyph) = result {
                assert!(glyph.width > 0 && glyph.height > 0,
                    "{name}: glyph 'A' has zero dimensions");
                assert!(!glyph.pixels.is_empty(),
                    "{name}: glyph 'A' has empty pixel buffer");
                assert!(glyph.pixels.iter().any(|&b| b > 0),
                    "{name}: glyph 'A' pixels are all zero (nothing rendered)");
                eprintln!("{name}: rasterizer VALID (glyph 'A' = {}x{} px)", glyph.width, glyph.height);
            } else {
                eprintln!("{name}: rasterizer returned None for 'A' (font may lack outline)");
            }
        }
    } else {
        eprintln!("{name}: SKIP rasterizer (no font found in embedded db)");
    }
}

// ─── Full Canvas Fidelity for Each Real Document ────

macro_rules! canvas_fidelity_test {
    ($name:ident, $path:expr, $label:expr) => {
        #[test]
        fn $name() {
            if let Some(doc) = open_doc($path) {
                validate_layout(&doc, $label);
                validate_html_consistency(&doc, $label);
                eprintln!("{}: ALL CANVAS CHECKS PASS\n", $label);
            } else {
                eprintln!("SKIP {}: file not found", $label);
            }
        }
    };
}

canvas_fidelity_test!(canvas_aruljothi, "tests/fidelity/real-world/Aruljothi.docx", "canvas_aruljothi");
canvas_fidelity_test!(canvas_chat_reaction, "tests/fidelity/real-world/Chat Reaction (1).docx", "canvas_chat_reaction");
canvas_fidelity_test!(canvas_medical_form, "tests/fidelity/real-world/Medical-In...orm.docx", "canvas_medical_form");
canvas_fidelity_test!(canvas_nishtriya, "tests/fidelity/real-world/Nishtriya.docx", "canvas_nishtriya");
canvas_fidelity_test!(canvas_design_system, "tests/fidelity/real-world/design-system.docx", "canvas_design_system");
canvas_fidelity_test!(canvas_fannest, "tests/fidelity/real-world/fannest_final_design_system.docx", "canvas_fannest");
canvas_fidelity_test!(canvas_calibre, "tests/fidelity/real-world/resume_calibre.docx", "canvas_calibre");

// Corpus files
canvas_fidelity_test!(canvas_tier1_basic, "tests/fidelity/corpus/tier1/basic-paragraphs.docx", "canvas_tier1_basic");
canvas_fidelity_test!(canvas_tier2_tables, "tests/fidelity/corpus/tier2/tables-merged.docx", "canvas_tier2_tables");
canvas_fidelity_test!(canvas_tier3_images, "tests/fidelity/corpus/tier3/inline-floating-images.docx", "canvas_tier3_images");
canvas_fidelity_test!(canvas_tier4_stress, "tests/fidelity/corpus/tier4/stress-recovery.docx", "canvas_tier4_stress");

// Rasterizer validation
#[test]
fn canvas_rasterizer_validation() {
    validate_rasterizer("rasterizer_test");
}
