use s1engine::Engine;

#[test]
fn check_sds_sectpr_in_ppr() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();

    // Check which paragraphs have SectionIndex and what section they reference
    let body_id = model.body_id().unwrap();
    let body = model.node(body_id).unwrap();
    let sections = model.sections();

    println!("Total sections: {}", sections.len());
    println!("Total body children: {}", body.children.len());

    let mut last_idx: Option<usize> = None;
    let last_sec_idx = if sections.is_empty() { None } else { Some(sections.len() - 1) };

    for (i, cid) in body.children.iter().enumerate() {
        if let Some(n) = model.node(*cid) {
            if n.node_type == s1_model::NodeType::Paragraph {
                if let Some(s1_model::AttributeValue::Int(idx)) = n.attributes.get(&s1_model::AttributeKey::SectionIndex) {
                    let idx = *idx as usize;
                    if Some(idx) != last_idx && Some(idx) != last_sec_idx {
                        println!("  Para #{}: SectionIndex={} (mid-doc section, will write PPR_SECT_PR)", i, idx);
                        // Check section properties
                        if let Some(sec) = sections.get(idx) {
                            println!("    break={:?} hdrs={} ftrs={} title={}", sec.break_type, sec.headers.len(), sec.footers.len(), sec.title_page);
                        }
                    }
                    last_idx = Some(idx);
                }
            }
        }
    }

    // Generate DOCY and find first paragraph with PPR_SECT_PR
    let docy = s1_format_docy::write(model);
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[3]).unwrap();

    // Find doc table
    let count = binary[0] as usize;
    let doc_off = (0..count).find_map(|i| {
        let pos = 1 + i * 5;
        if binary[pos] == 6 { Some(u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize) }
        else { None }
    }).unwrap();
    let doc_len = u32::from_le_bytes([binary[doc_off], binary[doc_off+1], binary[doc_off+2], binary[doc_off+3]]) as usize;

    // Scan for pPr containing type=31 (PPR_SECT_PR)
    let mut cur = doc_off + 4;
    let end = doc_off + 4 + doc_len;
    let mut pidx = 0;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;

        if typ == 0 && len > 0 { // Paragraph
            // Check pPr for type=31
            let mut pcur = cur + 5;
            let pend = cur + 5 + len;
            if pcur + 5 <= pend {
                let pt = binary[pcur];
                let pl = u32::from_le_bytes([binary[pcur+1], binary[pcur+2], binary[pcur+3], binary[pcur+4]]) as usize;
                if pt == 1 && pl > 0 { // pPr
                    // Scan Read2 for type=31
                    let mut r2 = pcur + 5;
                    let r2end = pcur + 5 + pl;
                    while r2 + 2 <= r2end {
                        let r2t = binary[r2];
                        let r2lt = binary[r2 + 1];
                        if r2t == 31 { // PPR_SECT_PR!
                            println!("\nPara #{} @{}: pPr has PPR_SECT_PR at @{}", pidx, cur, r2);
                            if r2lt == 6 && r2 + 6 <= r2end {
                                let vlen = u32::from_le_bytes([binary[r2+2], binary[r2+3], binary[r2+4], binary[r2+5]]) as usize;
                                println!("  varLen={}, content at @{}", vlen, r2+6);
                                // Dump hex of section content
                                let sstart = r2 + 6;
                                let send = (sstart + vlen).min(binary.len());
                                print!("  hex: ");
                                for j in sstart..send.min(sstart+40) {
                                    print!("{:02x} ", binary[j]);
                                }
                                println!();
                                // Try Read1 walk
                                let mut scur = sstart;
                                while scur + 5 <= send {
                                    let st = binary[scur];
                                    let sl = u32::from_le_bytes([binary[scur+1], binary[scur+2], binary[scur+3], binary[scur+4]]) as usize;
                                    let sec_names = ["pgSz","pgMar","settings","headers","footers","hdrftrelem","pageNumType","sectPrChange","cols"];
                                    let sname = sec_names.get(st as usize).unwrap_or(&"?");
                                    println!("    SectPr @{}: type={}({}) len={}", scur, st, sname, sl);
                                    scur += 5 + sl;
                                }
                            }
                            break;
                        }
                        let shift = match r2lt {
                            0 => 2, 1 => 3, 2 => 4, 3 => 5, 4 => 6, 5 => 10,
                            6 => {
                                if r2 + 6 <= r2end {
                                    6 + u32::from_le_bytes([binary[r2+2], binary[r2+3], binary[r2+4], binary[r2+5]]) as usize
                                } else { break; }
                            }
                            _ => { println!("  BAD lenType at @{}: type={} lt={}", r2, r2t, r2lt); break; }
                        };
                        r2 += shift;
                    }
                }
            }
            pidx += 1;
        }
        cur += 5 + len;
    }
}
