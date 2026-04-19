use s1engine::Engine;

#[test]
fn find_byte_3310() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let docy = s1_format_docy::write(model);
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD, parts[3]
    ).unwrap();

    // Doc table
    let count = binary[0] as usize;
    let doc_off = (0..count).find_map(|i| {
        let pos = 1 + i * 5;
        if binary[pos] == 6 { Some(u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize) }
        else { None }
    }).unwrap();
    let doc_len = u32::from_le_bytes([binary[doc_off], binary[doc_off+1], binary[doc_off+2], binary[doc_off+3]]) as usize;
    println!("Doc: off={} len={}", doc_off, doc_len);

    // Walk first few paragraphs
    let mut cur = doc_off + 4;
    let end = doc_off + 4 + doc_len;
    let mut elem = 0;
    while cur + 5 <= end && elem < 5 {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        println!("\nDoc elem #{} @{}: type={} len={}", elem, cur, typ, len);

        if typ == 0 && len > 0 { // Paragraph
            // Walk pPr and Content
            let mut pcur = cur + 5;
            let pend = cur + 5 + len;
            while pcur + 5 <= pend {
                let pt = binary[pcur];
                let pl = u32::from_le_bytes([binary[pcur+1], binary[pcur+2], binary[pcur+3], binary[pcur+4]]) as usize;
                println!("  Para field @{}: type={} len={}", pcur, pt, pl);

                if pt == 1 && pl > 0 { // pPr - walk Read2
                    let mut r2cur = pcur + 5;
                    let r2end = pcur + 5 + pl;
                    while r2cur + 2 <= r2end {
                        let r2type = binary[r2cur];
                        let r2lt = binary[r2cur + 1];
                        let val_len = match r2lt {
                            0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 8,
                            6 => {
                                if r2cur + 6 <= r2end {
                                    u32::from_le_bytes([binary[r2cur+2], binary[r2cur+3], binary[r2cur+4], binary[r2cur+5]]) as usize
                                } else { println!("    TRUNCATED Variable at @{}", r2cur); break; }
                            }
                            _ => { println!("    INVALID lenType={} at @{} type={}", r2lt, r2cur, r2type); break; }
                        };
                        let shift = if r2lt == 6 { 6 + val_len } else { 2 + val_len };
                        println!("    pPr @{}: type={} lenType={} valLen={}", r2cur, r2type, r2lt, val_len);
                        r2cur += shift;
                    }
                }
                pcur = pcur + 5 + pl;
            }
        }
        elem += 1;
        cur = cur + 5 + len;
    }

    // Hex around 3310
    println!("\n--- Hex around 3310 ---");
    let s = 3290.min(binary.len());
    let e = 3330.min(binary.len());
    for i in s..e {
        if (i - s) % 20 == 0 { print!("\n  {:5}: ", i); }
        print!("{:02x} ", binary[i]);
    }
    println!();
}
