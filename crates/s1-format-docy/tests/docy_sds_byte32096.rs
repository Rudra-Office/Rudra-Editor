use s1engine::Engine;

#[test]
fn find_32096() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let docy = s1_format_docy::write(doc.model());
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let bin = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[3]).unwrap();

    // Find doc table
    let count = bin[0] as usize;
    let doc_off = (0..count).find_map(|i| {
        let p = 1 + i * 5;
        if bin[p] == 6 { Some(u32::from_le_bytes([bin[p+1],bin[p+2],bin[p+3],bin[p+4]]) as usize) } else { None }
    }).unwrap();
    let doc_len = u32::from_le_bytes([bin[doc_off],bin[doc_off+1],bin[doc_off+2],bin[doc_off+3]]) as usize;
    println!("Doc: off={} len={}", doc_off, doc_len);

    // Walk to find what element contains byte 32096
    let mut cur = doc_off + 4;
    let end = doc_off + 4 + doc_len;
    let mut idx = 0;
    while cur + 5 <= end {
        let typ = bin[cur];
        let len = u32::from_le_bytes([bin[cur+1],bin[cur+2],bin[cur+3],bin[cur+4]]) as usize;
        if cur <= 32096 && 32096 < cur + 5 + len {
            let names = ["Par","pPr","Content","Table","sectPr"];
            let name = names.get(typ as usize).unwrap_or(&"?");
            println!("** Element #{} @{}: type={}({}) len={} contains 32096", idx, cur, typ, name, len);

            // Walk inside
            if typ == 0 { // Paragraph
                let mut pcur = cur + 5;
                let pend = cur + 5 + len;
                while pcur + 5 <= pend {
                    let pt = bin[pcur];
                    let pl = u32::from_le_bytes([bin[pcur+1],bin[pcur+2],bin[pcur+3],bin[pcur+4]]) as usize;
                    if pcur <= 32096 && 32096 < pcur + 5 + pl {
                        println!("  pField @{}: type={} len={} CONTAINS 32096", pcur, pt, pl);
                        // Dump Read2 around 32096
                        let offset_in_field = 32096 - (pcur + 5);
                        println!("  offset in field content: {}", offset_in_field);
                        let from = 32090.min(bin.len());
                        let to = 32110.min(bin.len());
                        print!("  hex: ");
                        for j in from..to { print!("{:02x} ", bin[j]); }
                        println!();
                    }
                    pcur += 5 + pl;
                }
            }
            if typ == 3 { // Table
                println!("  (table — checking last bytes)");
                let tend = cur + 5 + len;
                let from = (tend - 20).max(cur + 5);
                print!("  tail hex @{}: ", from);
                for j in from..tend { print!("{:02x} ", bin[j]); }
                println!();
                println!("  next bytes after table @{}: {:02x} {:02x} {:02x} {:02x} {:02x}",
                    tend, bin[tend], bin[tend+1], bin[tend+2], bin[tend+3], bin[tend+4]);
            }
        }
        idx += 1;
        cur += 5 + len;
    }
}
