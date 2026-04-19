use s1engine::Engine;

#[test]
fn hex_dump_para1() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let docy = s1_format_docy::write(model);
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[3]).unwrap();

    // Paragraph #1 at @3198, len=229. pPr at @3203, len=117, content at 3208-3325
    println!("Full pPr hex (3208 to 3325):");
    for i in 3208..3325 {
        if (i - 3208) % 20 == 0 { print!("\n  {:5}: ", i); }
        print!("{:02x} ", binary[i]);
    }
    println!("\n\nContent hex (3325 to 3432):");
    for i in 3325..3432 {
        if (i - 3325) % 20 == 0 { print!("\n  {:5}: ", i); }
        print!("{:02x} ", binary[i]);
    }
    
    // Also check: model paragraph #1 attributes
    let body_id = model.body_id().unwrap();
    let body = model.node(body_id).unwrap();
    let para_id = body.children[1]; // second paragraph (index 1)
    let para = model.node(para_id).unwrap();
    println!("\n\nPara #1 attrs:");
    for (k, v) in para.attributes.iter() {
        println!("  {:?} = {:?}", k, v);
    }
    println!("Para #1 children: {}", para.children.len());
    for (i, cid) in para.children.iter().enumerate() {
        if let Some(c) = model.node(*cid) {
            println!("  [{}] {:?} text={:?}", i, c.node_type, c.text_content.as_deref().unwrap_or(""));
        }
    }
    println!();
}
