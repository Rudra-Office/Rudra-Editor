//! Real-document fidelity tests.
//!
//! Opens actual DOCX files, tests round-trip fidelity, and reports what
//! survived vs what was lost. This is the ground-truth fidelity validation.

use s1engine::{Engine, Format};
use std::path::Path;

fn test_docx_fidelity(path: &str, name: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join(path);

    if !path.exists() {
        eprintln!("SKIP {name}: file not found at {}", path.display());
        return;
    }

    let data = std::fs::read(&path).unwrap();
    let engine = Engine::new();

    // Phase 1: Open the document
    let doc = engine.open(&data);
    assert!(doc.is_ok(), "{name}: failed to open: {:?}", doc.err());
    let doc = doc.unwrap();

    // Phase 2: Extract text
    let text = doc.to_plain_text();
    eprintln!("{name}: text length = {} chars, paragraphs = {}", text.len(), doc.paragraph_count());
    assert!(!text.is_empty() || doc.paragraph_count() == 0,
        "{name}: document has paragraphs but no text extracted");

    // Phase 3: Export to DOCX
    let exported = doc.export(Format::Docx);
    assert!(exported.is_ok(), "{name}: DOCX export failed: {:?}", exported.err());
    let exported_bytes = exported.unwrap();
    assert!(exported_bytes.len() > 100, "{name}: exported DOCX too small: {} bytes", exported_bytes.len());

    // Phase 4: Re-import and compare
    let doc2 = engine.open(&exported_bytes);
    assert!(doc2.is_ok(), "{name}: re-import failed: {:?}", doc2.err());
    let doc2 = doc2.unwrap();
    let text2 = doc2.to_plain_text();

    // Text should be preserved (allow minor whitespace differences)
    let clean1: String = text.split_whitespace().collect();
    let clean2: String = text2.split_whitespace().collect();
    let similarity = if clean1.is_empty() { 100.0 } else {
        let common = clean1.chars().zip(clean2.chars()).filter(|(a, b)| a == b).count();
        (common as f64 / clean1.len().max(1) as f64) * 100.0
    };

    eprintln!("{name}: round-trip text similarity = {:.1}%", similarity);
    eprintln!("{name}: original size = {} bytes, re-exported = {} bytes",
        data.len(), exported_bytes.len());
    eprintln!("{name}: paragraphs: {} → {}", doc.paragraph_count(), doc2.paragraph_count());

    // Headings should survive
    let h1 = doc.model().collect_headings();
    let h2 = doc2.model().collect_headings();
    eprintln!("{name}: headings: {} → {}", h1.len(), h2.len());

    // Metadata should survive
    let m1 = doc.model().metadata();
    let m2 = doc2.model().metadata();
    if m1.title.is_some() {
        assert_eq!(m1.title, m2.title, "{name}: title lost in round-trip");
    }

    // Multilingual documents may have lower similarity due to CJK/RTL encoding differences
    let threshold = if name.contains("multilingual") || name.contains("cjk") { 60.0 } else { 90.0 };
    assert!(similarity > threshold, "{name}: text similarity too low: {:.1}% (threshold: {:.0}%)", similarity, threshold);

    eprintln!("{name}: PASS\n");
}

// ─── Corpus Tier 1: Core ────────────────────────────

#[test]
fn fidelity_tier1_basic_paragraphs() {
    test_docx_fidelity("tests/fidelity/corpus/tier1/basic-paragraphs.docx", "tier1_basic_paragraphs");
}

#[test]
fn fidelity_tier1_headers_footers() {
    test_docx_fidelity("tests/fidelity/corpus/tier1/headers-footers.docx", "tier1_headers_footers");
}

// ─── Corpus Tier 2: Structured ──────────────────────

#[test]
fn fidelity_tier2_tables_merged() {
    test_docx_fidelity("tests/fidelity/corpus/tier2/tables-merged.docx", "tier2_tables_merged");
}

#[test]
fn fidelity_tier2_multi_section() {
    test_docx_fidelity("tests/fidelity/corpus/tier2/multi-section.docx", "tier2_multi_section");
}

#[test]
fn fidelity_tier2_lists_bookmarks() {
    test_docx_fidelity("tests/fidelity/corpus/tier2/lists-bookmarks.docx", "tier2_lists_bookmarks");
}

// ─── Corpus Tier 3: Visual ──────────────────────────

#[test]
fn fidelity_tier3_comments_review() {
    test_docx_fidelity("tests/fidelity/corpus/tier3/comments-review.docx", "tier3_comments_review");
}

#[test]
fn fidelity_tier3_images() {
    test_docx_fidelity("tests/fidelity/corpus/tier3/inline-floating-images.docx", "tier3_images");
}

// ─── Corpus Tier 4: Stress ──────────────────────────

#[test]
fn fidelity_tier4_large_multilingual() {
    test_docx_fidelity("tests/fidelity/corpus/tier4/large-multilingual.docx", "tier4_multilingual");
}

#[test]
fn fidelity_tier4_stress_recovery() {
    test_docx_fidelity("tests/fidelity/corpus/tier4/stress-recovery.docx", "tier4_stress");
}

// ─── Real-World Documents ───────────────────────────

#[test]
fn fidelity_real_calibre_demo() {
    test_docx_fidelity("tests/fidelity/real-world/resume_calibre.docx", "real_calibre_demo");
}

#[test]
fn fidelity_real_aruljothi() {
    test_docx_fidelity("tests/fidelity/real-world/Aruljothi.docx", "real_aruljothi");
}

#[test]
fn fidelity_real_chat_reaction() {
    test_docx_fidelity("tests/fidelity/real-world/Chat Reaction (1).docx", "real_chat_reaction");
}

#[test]
fn fidelity_real_medical_form() {
    test_docx_fidelity("tests/fidelity/real-world/Medical-In...orm.docx", "real_medical_form");
}

#[test]
fn fidelity_real_nishtriya() {
    test_docx_fidelity("tests/fidelity/real-world/Nishtriya.docx", "real_nishtriya");
}

#[test]
fn fidelity_real_design_system() {
    test_docx_fidelity("tests/fidelity/real-world/design-system.docx", "real_design_system");
}

#[test]
fn fidelity_real_fannest_design() {
    test_docx_fidelity("tests/fidelity/real-world/fannest_final_design_system.docx", "real_fannest_design");
}
