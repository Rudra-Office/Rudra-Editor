use s1engine::Engine;

#[test]
fn check_grid_values() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let body_id = model.body_id().unwrap();
    let body = model.node(body_id).unwrap();

    for (i, cid) in body.children.iter().enumerate() {
        if let Some(n) = model.node(*cid) {
            if n.node_type == s1_model::NodeType::Table {
                let cw = n.attributes.get_string(&s1_model::AttributeKey::TableColumnWidths);
                println!("Table body[{}] ColumnWidths={:?}", i, cw);
                if let Some(s) = cw {
                    let widths: Vec<f64> = s.split(',').filter_map(|v| v.trim().parse().ok()).collect();
                    let total_pts: f64 = widths.iter().sum();
                    let total_twips: f64 = widths.iter().map(|w| (w * 20.0).round()).sum();
                    println!("  Parsed: {:?}", widths);
                    println!("  Total: {:.1} pts = {:.0} twips", total_pts, total_twips);
                    println!("  Page width: 595.5 pts = 11910 twips, margins: 64+49 = 113 pts = 2260 twips");
                    println!("  Available: {:.0} twips", 11910.0 - 2260.0);
                }
            }
        }
    }
    
    // Also check: what does the DOCY binary actually contain for the grid?
    let docy = s1_format_docy::write(model);
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let bin = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[3]).unwrap();
    
    let doc_off = {
        let count = bin[0] as usize;
        (0..count).find_map(|i| {
            let p = 1+i*5; if bin[p]==6 { Some(u32::from_le_bytes([bin[p+1],bin[p+2],bin[p+3],bin[p+4]]) as usize) } else { None }
        }).unwrap()
    };
    let mut cur = doc_off + 4;
    let dlen = u32::from_le_bytes([bin[doc_off],bin[doc_off+1],bin[doc_off+2],bin[doc_off+3]]) as usize;
    let end = doc_off + 4 + dlen;
    let mut idx = 0;
    while cur + 5 <= end {
        let t = bin[cur];
        let l = u32::from_le_bytes([bin[cur+1],bin[cur+2],bin[cur+3],bin[cur+4]]) as usize;
        if t == 3 { // Table
            println!("\nBinary Table @{} len={}:", cur, l);
            // Find tblGrid (type=1) inside
            let mut tcur = cur + 5;
            let tend = cur + 5 + l;
            while tcur + 5 <= tend {
                let tt = bin[tcur];
                let tl = u32::from_le_bytes([bin[tcur+1],bin[tcur+2],bin[tcur+3],bin[tcur+4]]) as usize;
                if tt == 1 { // tblGrid
                    println!("  tblGrid @{} len={}:", tcur, tl);
                    // Read2 items: type(1)+lenType(1)+value
                    let mut gcur = tcur + 5;
                    let gend = tcur + 5 + tl;
                    while gcur + 2 <= gend {
                        let gt = bin[gcur];
                        let glt = bin[gcur+1];
                        if glt == 4 { // Long
                            let v = u32::from_le_bytes([bin[gcur+2],bin[gcur+3],bin[gcur+4],bin[gcur+5]]);
                            println!("    type={} val={} twips ({:.1} pts, {:.1} mm)", gt, v, v as f64 / 20.0, v as f64 * 0.0176389);
                            gcur += 6;
                        } else {
                            break;
                        }
                    }
                }
                tcur += 5 + tl;
            }
        }
        idx += 1;
        cur += 5 + l;
    }
}
