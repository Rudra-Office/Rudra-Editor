//! Tier A Fidelity Tests — Basic document structure and formatting.
//!
//! Each test opens a fixture, generates DOCY, validates binary structure,
//! and checks feature inventory against expected counts.

use s1engine::Engine;
use s1_model::{NodeType, AttributeKey, AttributeValue};

struct FidelityScore {
    name: String,
    // Structural
    expected_paras: usize,
    actual_paras: usize,
    expected_tables: usize,
    actual_tables: usize,
    expected_runs: usize,
    actual_runs: usize,
    // Formatting
    has_bold: bool,
    expected_bold: bool,
    has_italic: bool,
    expected_italic: bool,
    has_headings: bool,
    expected_headings: bool,
    // Binary
    binary_errors: usize,
    doc_elements: usize,
}

impl FidelityScore {
    fn structural_score(&self) -> f64 {
        let mut score = 0.0;
        let mut total = 0.0;

        // Paragraph count match (40% of structural)
        total += 40.0;
        if self.expected_paras > 0 {
            let ratio = self.actual_paras as f64 / self.expected_paras as f64;
            score += 40.0 * ratio.min(1.0);
        } else if self.actual_paras == 0 {
            score += 40.0;
        }

        // Table count match (20%)
        total += 20.0;
        if self.expected_tables == self.actual_tables {
            score += 20.0;
        }

        // Run count match (20%)
        total += 20.0;
        if self.expected_runs > 0 {
            let ratio = self.actual_runs as f64 / self.expected_runs as f64;
            score += 20.0 * ratio.min(1.0);
        } else if self.actual_runs == 0 {
            score += 20.0;
        }

        // Binary integrity (20%)
        total += 20.0;
        if self.binary_errors == 0 {
            score += 20.0;
        }

        (score / total) * 100.0
    }

    fn format_score(&self) -> f64 {
        let mut checks = 0;
        let mut passed = 0;

        if self.expected_bold { checks += 1; if self.has_bold { passed += 1; } }
        if self.expected_italic { checks += 1; if self.has_italic { passed += 1; } }
        if self.expected_headings { checks += 1; if self.has_headings { passed += 1; } }

        if checks == 0 { return 100.0; }
        (passed as f64 / checks as f64) * 100.0
    }

    fn open_score(&self) -> f64 {
        if self.binary_errors > 0 { return 0.0; }
        if self.doc_elements <= 1 { return 0.0; }
        100.0
    }

    fn overall(&self) -> f64 {
        // Weighted: open=20%, structural=40%, format=40%
        self.open_score() * 0.20 + self.structural_score() * 0.40 + self.format_score() * 0.40
    }

    fn print(&self) {
        println!("\n=== {} Fidelity Report ===", self.name);
        println!("  Open:       {:.0}%", self.open_score());
        println!("  Structural: {:.0}% (paras={}/{}, tables={}/{}, runs={}/{})",
            self.structural_score(),
            self.actual_paras, self.expected_paras,
            self.actual_tables, self.expected_tables,
            self.actual_runs, self.expected_runs);
        println!("  Format:     {:.0}% (bold={}/{}, italic={}/{}, headings={}/{})",
            self.format_score(),
            self.has_bold, self.expected_bold,
            self.has_italic, self.expected_italic,
            self.has_headings, self.expected_headings);
        println!("  Binary:     {} errors, {} doc elements", self.binary_errors, self.doc_elements);
        println!("  OVERALL:    {:.1}%", self.overall());
    }
}

fn analyze_model(path: &str) -> (s1engine::Document, FidelityScore) {
    let engine = Engine::new();
    let bytes = std::fs::read(path).expect(&format!("file not found: {}", path));
    let doc = engine.open(&bytes).unwrap();

    let model = doc.model();
    let body_id = model.body_id().unwrap();
    let body = model.node(body_id).unwrap();

    let mut paras = 0;
    let mut tables = 0;
    let mut runs = 0;
    let mut has_bold = false;
    let mut has_italic = false;
    let mut has_headings = false;

    for cid in &body.children {
        if let Some(n) = model.node(*cid) {
            match n.node_type {
                NodeType::Paragraph => {
                    paras += 1;
                    if let Some(style) = n.attributes.get_string(&AttributeKey::StyleId) {
                        if style.starts_with("Heading") || style == "Title" || style == "Subtitle" {
                            has_headings = true;
                        }
                    }
                    for kid in &n.children {
                        if let Some(k) = model.node(*kid) {
                            if k.node_type == NodeType::Run {
                                runs += 1;
                                if let Some(true) = k.attributes.get_bool(&AttributeKey::Bold) {
                                    has_bold = true;
                                }
                                if let Some(true) = k.attributes.get_bool(&AttributeKey::Italic) {
                                    has_italic = true;
                                }
                            }
                        }
                    }
                }
                NodeType::Table => { tables += 1; }
                _ => {}
            }
        }
    }

    let score = FidelityScore {
        name: path.rsplit('/').next().unwrap_or(path).to_string(),
        expected_paras: paras,
        actual_paras: 0,
        expected_tables: tables,
        actual_tables: 0,
        expected_runs: runs,
        actual_runs: 0,
        has_bold: false,
        expected_bold: has_bold,
        has_italic: false,
        expected_italic: has_italic,
        has_headings: false,
        expected_headings: has_headings,
        binary_errors: 0,
        doc_elements: 0,
    };

    (doc, score)
}

fn validate_docy(doc: &s1engine::Document, score: &mut FidelityScore) {
    let model = doc.model();
    let docy = s1_format_docy::write(model);

    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD, parts[3]
    ).unwrap();

    // Count doc elements and check binary integrity
    let count = binary[0] as usize;
    let mut doc_off = 0usize;
    for i in 0..count {
        let pos = 1 + i * 5;
        let t = binary[pos];
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize;
        if t == 6 { doc_off = off; }
    }

    if doc_off + 4 <= binary.len() {
        let doc_len = u32::from_le_bytes([binary[doc_off], binary[doc_off+1], binary[doc_off+2], binary[doc_off+3]]) as usize;
        let mut cur = doc_off + 4;
        let end = doc_off + 4 + doc_len;
        let mut elems = 0;
        let mut paras = 0;
        let mut tables = 0;
        while cur + 5 <= end {
            let typ = binary[cur];
            let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
            if cur + 5 + len > binary.len() {
                score.binary_errors += 1;
                break;
            }
            match typ {
                0 => { paras += 1; analyze_para_binary(&binary, cur+5, len, score); }
                3 => { tables += 1; }
                _ => {}
            }
            elems += 1;
            cur = cur + 5 + len;
        }
        if cur != end { score.binary_errors += 1; }
        score.doc_elements = elems;
        score.actual_paras = paras;
        score.actual_tables = tables;
    }
}

fn analyze_para_binary(binary: &[u8], start: usize, length: usize, score: &mut FidelityScore) {
    let mut cur = start;
    let end = start + length;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end { score.binary_errors += 1; return; }
        if typ == 2 { // Content
            analyze_content_binary(binary, cur+5, len, score);
        }
        if typ == 1 { // pPr — check for heading style
            check_ppr_for_heading(binary, cur+5, len, score);
        }
        cur = cur + 5 + len;
    }
}

fn analyze_content_binary(binary: &[u8], start: usize, length: usize, score: &mut FidelityScore) {
    let mut cur = start;
    let end = start + length;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end { score.binary_errors += 1; return; }
        if typ == 5 { // Run
            score.actual_runs += 1;
            check_run_for_formatting(binary, cur+5, len, score);
        }
        cur = cur + 5 + len;
    }
}

fn check_run_for_formatting(binary: &[u8], start: usize, length: usize, score: &mut FidelityScore) {
    let mut cur = start;
    let end = start + length;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end { return; }
        if typ == 1 && len > 0 { // rPr
            check_rpr_binary(binary, cur+5, len, score);
        }
        cur = cur + 5 + len;
    }
}

fn check_rpr_binary(binary: &[u8], start: usize, length: usize, score: &mut FidelityScore) {
    // Read2 format: type + lenType + value
    let mut cur = start;
    let end = start + length;
    while cur + 2 <= end {
        let typ = binary[cur];
        let len_type = binary[cur+1];
        cur += 2;
        match len_type {
            1 => { // Byte
                if cur < end {
                    let val = binary[cur];
                    if typ == 0 && val == 1 { score.has_bold = true; } // BOLD
                    if typ == 1 && val == 1 { score.has_italic = true; } // ITALIC
                }
                cur += 1;
            }
            4 => { cur += 4; }
            5 => { cur += 8; }
            6 => {
                if cur + 4 <= end {
                    let vlen = u32::from_le_bytes([binary[cur], binary[cur+1], binary[cur+2], binary[cur+3]]) as usize;
                    cur += 4 + vlen;
                } else { return; }
            }
            _ => { return; }
        }
    }
}

fn check_ppr_for_heading(binary: &[u8], start: usize, length: usize, score: &mut FidelityScore) {
    // Read2: look for PARA_STYLE (21) with heading-like value
    let mut cur = start;
    let end = start + length;
    while cur + 2 <= end {
        let typ = binary[cur];
        let len_type = binary[cur+1];
        cur += 2;
        match len_type {
            1 => { cur += 1; }
            4 => { cur += 4; }
            5 => { cur += 8; }
            6 => {
                if cur + 4 <= end {
                    let vlen = u32::from_le_bytes([binary[cur], binary[cur+1], binary[cur+2], binary[cur+3]]) as usize;
                    if typ == 21 && vlen > 0 && cur + 4 + vlen <= end {
                        // Decode UTF-16LE style name
                        let data = &binary[cur+4..cur+4+vlen];
                        let chars: Vec<u16> = data.chunks(2)
                            .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
                            .collect();
                        if let Ok(s) = String::from_utf16(&chars) {
                            if s.starts_with("Heading") || s == "Title" || s == "Subtitle" {
                                score.has_headings = true;
                            }
                        }
                    }
                    cur += 4 + vlen;
                } else { return; }
            }
            _ => { return; }
        }
    }
}

// ── Tier A Tests ──────────────────────────────────────────────────

#[test]
fn tier_a_text_only() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = format!("{}/../s1engine/tests/fixtures/text-only.docx", manifest);
    let (doc, mut score) = analyze_model(&path);
    validate_docy(&doc, &mut score);
    score.print();
    assert!(score.overall() >= 90.0, "Fidelity {:.1}% below 90% threshold", score.overall());
}

#[test]
fn tier_a_formatted() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = format!("{}/../s1engine/tests/fixtures/formatted.docx", manifest);
    let (doc, mut score) = analyze_model(&path);
    validate_docy(&doc, &mut score);
    score.print();
    assert!(score.overall() >= 90.0, "Fidelity {:.1}% below 90% threshold", score.overall());
}

#[test]
fn tier_b_chat_reaction() {
    let (doc, mut score) = analyze_model("/Users/sachin/Downloads/Chat Reaction.docx");
    validate_docy(&doc, &mut score);
    score.print();
    assert!(score.open_score() >= 100.0, "Open failed");
    assert!(score.structural_score() >= 80.0, "Structural fidelity too low: {:.1}%", score.structural_score());
}

#[test]
fn tier_b_sds() {
    let (doc, mut score) = analyze_model("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx");
    validate_docy(&doc, &mut score);
    score.print();
    assert!(score.open_score() >= 100.0, "Open failed");
    assert!(score.structural_score() >= 80.0, "Structural fidelity too low: {:.1}%", score.structural_score());
}

#[test]
fn tier_b_aruljothi() {
    let (doc, mut score) = analyze_model("/Users/sachin/Downloads/Aruljothi.docx");
    validate_docy(&doc, &mut score);
    score.print();
    assert!(score.open_score() >= 100.0, "Open failed");
}

#[test]
fn tier_b_nishtriya() {
    let (doc, mut score) = analyze_model("/Users/sachin/Downloads/Nishtriya.docx");
    validate_docy(&doc, &mut score);
    score.print();
    assert!(score.open_score() >= 100.0, "Open failed");
}

#[test]
fn regression_all_files_no_binary_errors() {
    let files = [
        "/Users/sachin/Downloads/Chat Reaction.docx",
        "/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx",
        "/Users/sachin/Downloads/Aruljothi.docx",
        "/Users/sachin/Downloads/Nishtriya.docx",
    ];
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let fixtures = [
        format!("{}/../s1engine/tests/fixtures/text-only.docx", manifest),
        format!("{}/../s1engine/tests/fixtures/formatted.docx", manifest),
    ];

    for path in files.iter().chain(fixtures.iter().map(|s| s.as_str()).collect::<Vec<_>>().iter()) {
        let (doc, mut score) = analyze_model(path);
        validate_docy(&doc, &mut score);
        assert_eq!(score.binary_errors, 0, "{}: binary errors", score.name);
        assert!(score.doc_elements > 1, "{}: only {} doc elements", score.name, score.doc_elements);
        println!("  {} — {:.1}% ({} elements, 0 errors)", score.name, score.overall(), score.doc_elements);
    }
}
