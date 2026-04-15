use s1_format_docy;
use s1_model::DocumentModel;
use s1engine::{Engine, Format};
use base64::engine::Engine as _;

#[test]
fn dump_empty_docy_bytes() {
    let model = DocumentModel::new();
    let docy = s1_format_docy::write(&model);
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::general_purpose::STANDARD.decode(parts[3]).unwrap();

    println!("\n=== Empty DOCY ===");
    dump_docy(&binary);

    // Verify signature
    let sig_off = u32::from_le_bytes([binary[2], binary[3], binary[4], binary[5]]) as usize;
    assert_eq!(binary[sig_off], 0x00);
    assert_eq!(binary[sig_off + 1], 0x04);
}

#[test]
fn dump_simple_text_docy() {
    let engine = Engine::new();
    let doc = engine.open_as(b"Hello World\nSecond paragraph", Format::Txt).unwrap();
    let docy = s1_format_docy::write(doc.model());
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::general_purpose::STANDARD.decode(parts[3]).unwrap();

    println!("\n=== Simple text DOCY ===");
    dump_docy(&binary);
    println!("DOCY string length: {}", docy.len());
}

#[test]
fn dump_formatted_docy() {
    let engine = Engine::new();
    let doc = engine.open_as(b"# Heading\n\nNormal **bold** text", Format::Md).unwrap();
    let docy = s1_format_docy::write(doc.model());
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::general_purpose::STANDARD.decode(parts[3]).unwrap();

    println!("\n=== Formatted DOCY ===");
    dump_docy(&binary);
}

fn dump_docy(binary: &[u8]) {
    println!("Total bytes: {}", binary.len());
    println!("Table count: {}", binary[0]);

    let names = ["Sig","Info","Media","Num","HdrFtr","Style","Doc","Other","Comments","Settings","Foot","End"];
    for i in 0..binary[0] as usize {
        let pos = 1 + i * 5;
        let t = binary[pos] as usize;
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]);
        let name = if t < names.len() { names[t] } else { "?" };
        // Calculate table size
        let next_off = if i + 1 < binary[0] as usize {
            u32::from_le_bytes([binary[1+(i+1)*5+1], binary[1+(i+1)*5+2], binary[1+(i+1)*5+3], binary[1+(i+1)*5+4]])
        } else {
            binary.len() as u32
        };
        let size = next_off - off;
        println!("  Table[{}]: type={} ({}) offset={} size={}", i, t, name, off, size);

        let start = off as usize;
        let end = std::cmp::min(start + 40, binary.len());
        let hex: Vec<String> = binary[start..end].iter().map(|b| format!("{:02x}", b)).collect();
        println!("    bytes: {}", hex.join(" "));
    }
}
