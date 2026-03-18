//! Example: Document format converter CLI
//!
//! Usage:
//!   cargo run --example document-converter -- input.docx output.pdf
//!   cargo run --example document-converter -- report.odt report.docx
//!
//! Supported conversions:
//!   DOCX → PDF, ODT, TXT, MD
//!   ODT → PDF, DOCX, TXT, MD
//!   TXT → DOCX, ODT, PDF
//!   MD → DOCX, ODT, PDF

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: document-converter <input> <output>");
        eprintln!("  Supported formats: docx, odt, pdf, txt, md");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  document-converter report.docx report.pdf");
        eprintln!("  document-converter notes.md notes.docx");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Detect output format from extension
    let output_ext = output_path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    let format = match output_ext.as_str() {
        "docx" => s1engine::Format::Docx,
        "odt" => s1engine::Format::Odt,
        "pdf" => s1engine::Format::Pdf,
        "txt" => s1engine::Format::Txt,
        "md" => s1engine::Format::Md,
        other => {
            eprintln!("Unsupported output format: .{other}");
            std::process::exit(1);
        }
    };

    // Read input file
    let input_data = match std::fs::read(input_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read {input_path}: {e}");
            std::process::exit(1);
        }
    };

    println!("Opening {}...", input_path);

    // Open with s1engine (auto-detects format)
    let engine = s1engine::Engine::new();
    let doc = match engine.open(&input_data) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("Failed to open document: {e}");
            std::process::exit(1);
        }
    };

    println!(
        "Document: {} words, {} paragraphs",
        doc.to_plain_text().split_whitespace().count(),
        doc.model().node(doc.model().root_id()).map(|n| n.children.len()).unwrap_or(0)
    );

    // Export to target format
    println!("Converting to {}...", output_ext);
    let output_data = match doc.export(format) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Conversion failed: {e}");
            std::process::exit(1);
        }
    };

    // Write output
    match std::fs::write(output_path, &output_data) {
        Ok(_) => println!("Written: {} ({} bytes)", output_path, output_data.len()),
        Err(e) => {
            eprintln!("Failed to write {output_path}: {e}");
            std::process::exit(1);
        }
    }
}
