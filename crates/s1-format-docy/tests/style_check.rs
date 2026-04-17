use s1engine::{Engine, Format};

#[test]
fn check_styles_from_docx_fixture() {
    let engine = Engine::new();
    // Load the formatted fixture (has headings, bold, italic)
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = std::path::Path::new(&manifest).join("../s1engine/tests/fixtures/formatted.docx");
    let bytes = std::fs::read(&path).expect(&format!("fixture not found at {:?}", path));
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();

    println!("Styles count: {}", model.styles().len());
    for s in model.styles() {
        println!("  {} (id={}, type={:?}, parent={:?})", s.name, s.id, s.style_type, s.parent_id);
    }

    let body_id = model.body_id().unwrap();
    let body = model.node(body_id).unwrap();
    println!("\nBody children: {}", body.children.len());

    // Check DOCY output
    let docy = s1_format_docy::write(model);
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD, parts[3]
    ).unwrap();

    println!("\nDOCY binary: {} bytes", binary.len());
    println!("Table count: {}", binary[0]);
    for i in 0..binary[0] as usize {
        let pos = 1 + i * 5;
        let t = binary[pos];
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]);
        let names = ["Sig","Info","Media","Num","HdrFtr","Style","Doc","Other","Comments","Settings"];
        let name = if (t as usize) < names.len() { names[t as usize] } else { "?" };

        let next_off = if i + 1 < binary[0] as usize {
            u32::from_le_bytes([binary[1+(i+1)*5+1], binary[1+(i+1)*5+2], binary[1+(i+1)*5+3], binary[1+(i+1)*5+4]])
        } else { binary.len() as u32 };
        let size = next_off - off;
        println!("  Table {}: {} ({}) offset={} size={}", i, t, name, off, size);
    }
}
