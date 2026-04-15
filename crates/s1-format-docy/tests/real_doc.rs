use s1_format_docy;
use s1engine::{Engine, Format};
use base64::engine::Engine as _;

#[test]
fn check_docy_from_txt() {
    let engine = Engine::new();
    let doc = engine.open_as(b"Hello World\nSecond para\nThird para", Format::Txt).unwrap();
    let model = doc.model();

    // Check body
    let body_id = model.body_id().expect("should have body");
    let body = model.node(body_id).unwrap();
    println!("Body children: {}", body.children.len());
    for (i, cid) in body.children.iter().enumerate() {
        let child = model.node(*cid).unwrap();
        println!("  [{}] {:?}", i, child.node_type);
    }

    let docy = s1_format_docy::write(model);
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::general_purpose::STANDARD.decode(parts[3]).unwrap();

    // Find doc table
    for i in 0..binary[0] as usize {
        let pos = 1 + i * 5;
        let t = binary[pos];
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize;
        if t == 6 {
            let len = u32::from_le_bytes([binary[off], binary[off+1], binary[off+2], binary[off+3]]);
            println!("\nDoc table at {}: content length = {}", off, len);
            // Count paragraphs (type 0) in content
            let mut p = off + 4;
            let end = off + 4 + len as usize;
            let mut para_count = 0;
            while p < end {
                let item_type = binary[p];
                let item_len = u32::from_le_bytes([binary[p+1], binary[p+2], binary[p+3], binary[p+4]]) as usize;
                if item_type == 0 { para_count += 1; }
                println!("  item type={} len={} at offset {}", item_type, item_len, p);
                p += 5 + item_len;
            }
            println!("Paragraph count in DOCY: {}", para_count);
            break;
        }
    }
}
