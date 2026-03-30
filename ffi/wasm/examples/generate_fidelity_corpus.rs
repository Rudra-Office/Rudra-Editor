use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use s1engine::{Document, DocumentBuilder, Format};
use s1engine_wasm::WasmEngine;
use wasm_bindgen::JsError;

struct CaseSpec {
    id: &'static str,
    source_doc: &'static str,
    layout_json: &'static str,
    page_map_json: &'static str,
    pdf_path: &'static str,
    build: fn() -> Document,
}

fn repeated_paragraph(seed: &str, index: usize) -> String {
    format!(
        "Paragraph {}. {} This content is intentionally long enough to exercise line wrapping, paragraph spacing, and page flow across multiple pages.",
        index + 1,
        seed
    )
}

fn build_basic_paragraphs() -> Document {
    let mut builder = DocumentBuilder::new()
        .title("Tier 1 Basic Paragraphs")
        .author("s1engine fidelity generator")
        .heading(1, "Tier 1 Basic Paragraphs")
        .paragraph(|p| {
            p.text("This document exercises heading, paragraph, and inline formatting behavior. ")
                .bold("Bold text")
                .italic(" italic text")
                .underline(" underlined text")
        })
        .heading(2, "List Content")
        .bullet("First bullet item")
        .bullet("Second bullet item")
        .numbered("First numbered step")
        .numbered("Second numbered step");

    for i in 0..28 {
        let text = repeated_paragraph(
            "Tier 1 baseline paragraph for pagination and line-break validation.",
            i,
        );
        builder = builder.paragraph(|p| p.text(&text));
    }

    builder.build()
}

fn build_headers_footers() -> Document {
    let mut builder = DocumentBuilder::new()
        .title("Tier 1 Headers and Footers")
        .author("s1engine fidelity generator")
        .section_with_header_footer("Tier 1 Header", "Tier 1 Footer")
        .heading(1, "Headers and Footers")
        .paragraph(|p| {
            p.text("This document validates repeated page regions. ")
                .bold("Header and footer placement")
                .text(" should remain stable across multiple pages.")
        });

    for i in 0..34 {
        let text = repeated_paragraph(
            "Header/footer validation content intended to span enough pages to make repeated regions observable.",
            i,
        );
        builder = builder.paragraph(|p| p.text(&text));
    }

    builder.build()
}

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn ensure_parent(path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn write_case(root: &Path, case: &CaseSpec) -> Result<(), Box<dyn Error>> {
    let source_path = root.join(case.source_doc);
    let layout_path = root.join(case.layout_json);
    let page_map_path = root.join(case.page_map_json);
    let pdf_path = root.join(case.pdf_path);

    ensure_parent(&source_path)?;
    ensure_parent(&layout_path)?;
    ensure_parent(&page_map_path)?;
    ensure_parent(&pdf_path)?;

    let doc = (case.build)();
    let docx_bytes = doc.export(Format::Docx)?;
    fs::write(&source_path, &docx_bytes)?;

    let engine = WasmEngine::new();
    let wasm_doc = engine
        .open_as(&docx_bytes, "docx")
        .map_err(|e: JsError| format!("{e:?}"))?;
    let layout_json = wasm_doc
        .to_layout_json()
        .map_err(|e: JsError| format!("{e:?}"))?;
    let page_map_json = wasm_doc
        .get_page_map_json()
        .map_err(|e: JsError| format!("{e:?}"))?;
    let pdf_bytes = wasm_doc.to_pdf().map_err(|e: JsError| format!("{e:?}"))?;

    fs::write(&layout_path, layout_json)?;
    fs::write(&page_map_path, page_map_json)?;
    fs::write(&pdf_path, pdf_bytes)?;

    println!("generated {}", case.id);
    println!("  source: {}", source_path.display());
    println!("  layout: {}", layout_path.display());
    println!("  page_map: {}", page_map_path.display());
    println!("  pdf: {}", pdf_path.display());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let cases = [
        CaseSpec {
            id: "tier1_basic_paragraphs",
            source_doc: "tests/fidelity/corpus/tier1/basic-paragraphs.docx",
            layout_json: "tests/fidelity/artifacts/tier1_basic_paragraphs.engine.layout.json",
            page_map_json: "tests/fidelity/artifacts/tier1_basic_paragraphs.engine.page_map.json",
            pdf_path: "tests/fidelity/artifacts/tier1_basic_paragraphs.engine.pdf",
            build: build_basic_paragraphs,
        },
        CaseSpec {
            id: "tier1_headers_footers",
            source_doc: "tests/fidelity/corpus/tier1/headers-footers.docx",
            layout_json: "tests/fidelity/artifacts/tier1_headers_footers.engine.layout.json",
            page_map_json: "tests/fidelity/artifacts/tier1_headers_footers.engine.page_map.json",
            pdf_path: "tests/fidelity/artifacts/tier1_headers_footers.engine.pdf",
            build: build_headers_footers,
        },
    ];

    for case in &cases {
        write_case(&root, case)?;
    }

    Ok(())
}
