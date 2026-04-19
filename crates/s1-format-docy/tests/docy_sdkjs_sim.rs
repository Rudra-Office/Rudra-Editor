use s1engine::Engine;

fn read2_sim(binary: &[u8], start: usize, len: usize, label: &str, in_rpr: bool) -> Result<(), String> {
    let mut pos = 0;
    while pos < len {
        if start + pos + 2 > binary.len() {
            return Err(format!("{} @{}: truncated", label, start+pos));
        }
        let typ = binary[start + pos];
        let lt = binary[start + pos + 1];
        let val_len = match lt {
            0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 8,
            6 => {
                if start + pos + 6 > binary.len() { return Err(format!("{} @{}: trunc var", label, start+pos)); }
                u32::from_le_bytes([binary[start+pos+2], binary[start+pos+3], binary[start+pos+4], binary[start+pos+5]]) as usize
            }
            _ => return Err(format!("{} @{}: BAD lenType=0x{:02x} type={}", label, start+pos, lt, typ)),
        };
        let shift = if lt == 6 { 6 + val_len } else { 2 + val_len };
        if start + pos + shift > start + len {
            return Err(format!("{} @{}: overflow type={} shift={}", label, start+pos, typ, shift));
        }
        if lt == 6 && val_len > 0 {
            let inner = start + pos + 6;
            // In rPr context, COLOR(9) and HIGHLIGHT(11) contain raw bytes, not Read2
            if in_rpr && (typ == 9 || typ == 11) {
                // Skip - raw RGB bytes
            } else {
                match typ {
                    27 | 31 => { read1_sim(binary, inner, val_len, &format!("{}->{}", label, typ))?; }
                    1 | 9 | 14 | 17 | 18 | 22 => { read2_sim(binary, inner, val_len, &format!("{}->{}", label, typ), false)?; }
                    _ => {}
                }
            }
        }
        pos += shift;
    }
    Ok(())
}

fn read1_sim(binary: &[u8], start: usize, len: usize, label: &str) -> Result<(), String> {
    let mut pos = 0;
    while pos + 5 <= len {
        let typ = binary[start + pos];
        let il = u32::from_le_bytes([binary[start+pos+1], binary[start+pos+2], binary[start+pos+3], binary[start+pos+4]]) as usize;
        if pos + 5 + il > len {
            return Err(format!("{} @{}: Read1 overflow type={} len={}", label, start+pos, typ, il));
        }
        if typ == 18 && il > 0 { read2_sim(binary, start+pos+5, il, &format!("{}->tab", label), false)?; }
        pos += 5 + il;
    }
    if pos != len { return Err(format!("{}: misalign pos={} len={}", label, pos, len)); }
    Ok(())
}

fn validate_doc(binary: &[u8], off: usize) -> Vec<String> {
    let mut errors = Vec::new();
    let tlen = u32::from_le_bytes([binary[off], binary[off+1], binary[off+2], binary[off+3]]) as usize;
    let mut cur = off + 4;
    let end = off + 4 + tlen;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end { errors.push(format!("Doc @{}: overflow", cur)); break; }
        if typ == 0 && len > 0 { // Para
            let mut pcur = cur + 5;
            let pend = cur + 5 + len;
            while pcur + 5 <= pend {
                let pt = binary[pcur];
                let pl = u32::from_le_bytes([binary[pcur+1], binary[pcur+2], binary[pcur+3], binary[pcur+4]]) as usize;
                if pt == 1 && pl > 0 {
                    if let Err(e) = read2_sim(binary, pcur+5, pl, &format!("P@{}->pPr", cur), false) { errors.push(e); }
                }
                if pt == 2 && pl > 0 {
                    // Content - check runs
                    let mut rcur = pcur + 5;
                    let rend = pcur + 5 + pl;
                    while rcur + 5 <= rend {
                        let rt = binary[rcur];
                        let rl = u32::from_le_bytes([binary[rcur+1], binary[rcur+2], binary[rcur+3], binary[rcur+4]]) as usize;
                        if rt == 5 && rl > 0 { // Run
                            let mut rrcur = rcur + 5;
                            let rrend = rcur + 5 + rl;
                            while rrcur + 5 <= rrend {
                                let rrt = binary[rrcur];
                                let rrl = u32::from_le_bytes([binary[rrcur+1], binary[rrcur+2], binary[rrcur+3], binary[rrcur+4]]) as usize;
                                if rrt == 1 && rrl > 0 {
                                    if let Err(e) = read2_sim(binary, rrcur+5, rrl, &format!("P@{}->rPr", cur), true) { errors.push(e); }
                                }
                                rrcur += 5 + rrl;
                            }
                        }
                        rcur += 5 + rl;
                    }
                }
                pcur += 5 + pl;
            }
        }
        if typ == 4 && len > 0 { // SectPr
            if let Err(e) = read1_sim(binary, cur+5, len, "SectPr") { errors.push(e); }
        }
        cur += 5 + len;
    }
    errors
}

fn validate_styles(binary: &[u8], off: usize) -> Vec<String> {
    let mut errors = Vec::new();
    let tlen = u32::from_le_bytes([binary[off], binary[off+1], binary[off+2], binary[off+3]]) as usize;
    let mut cur = off + 4;
    let end = off + 4 + tlen;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if typ == 0 && len > 0 { if let Err(e) = read2_sim(binary, cur+5, len, "DefPPr", false) { errors.push(e); } }
        if typ == 1 && len > 0 { if let Err(e) = read2_sim(binary, cur+5, len, "DefRPr", true) { errors.push(e); } }
        if typ == 2 { // Styles list
            let mut scur = cur + 5;
            let send = cur + 5 + len;
            let mut si = 0;
            while scur + 5 <= send {
                let st = binary[scur];
                let sl = u32::from_le_bytes([binary[scur+1], binary[scur+2], binary[scur+3], binary[scur+4]]) as usize;
                if st == 0 && sl > 0 {
                    let mut fcur = scur + 5;
                    let fend = scur + 5 + sl;
                    while fcur + 5 <= fend {
                        let ft = binary[fcur];
                        let fl = u32::from_le_bytes([binary[fcur+1], binary[fcur+2], binary[fcur+3], binary[fcur+4]]) as usize;
                        if ft == 5 && fl > 0 { if let Err(e) = read2_sim(binary, fcur+5, fl, &format!("S{}->rPr", si), true) { errors.push(e); } }
                        if ft == 6 && fl > 0 { if let Err(e) = read2_sim(binary, fcur+5, fl, &format!("S{}->pPr", si), false) { errors.push(e); } }
                        fcur += 5 + fl;
                    }
                }
                si += 1;
                scur += 5 + sl;
            }
        }
        cur += 5 + len;
    }
    errors
}

#[test]
fn full_sdkjs_sim_all_files() {
    let files: Vec<(String, &str)> = vec![
        ("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx".into(), "SDS"),
        ("/Users/sachin/Downloads/Chat Reaction.docx".into(), "Chat"),
        ("/Users/sachin/Downloads/Aruljothi.docx".into(), "Arulj"),
        ("/Users/sachin/Downloads/Nishtriya.docx".into(), "Nisht"),
    ];
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let files: Vec<(String, &str)> = files.into_iter().chain(vec![
        (format!("{}/../s1engine/tests/fixtures/text-only.docx", manifest), "text"),
        (format!("{}/../s1engine/tests/fixtures/formatted.docx", manifest), "fmt"),
    ]).collect();

    let mut all_ok = true;
    for (path, name) in &files {
        let engine = Engine::new();
        let bytes = std::fs::read(path).unwrap();
        let doc = engine.open(&bytes).unwrap();
        let docy = s1_format_docy::write(doc.model());
        let parts: Vec<&str> = docy.splitn(4, ';').collect();
        let bin = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[3]).unwrap();

        let count = bin[0] as usize;
        let mut errors = Vec::new();
        for i in 0..count {
            let p = 1 + i * 5;
            let t = bin[p];
            let o = u32::from_le_bytes([bin[p+1], bin[p+2], bin[p+3], bin[p+4]]) as usize;
            if t == 5 { errors.extend(validate_styles(&bin, o)); }
            if t == 6 { errors.extend(validate_doc(&bin, o)); }
        }
        if errors.is_empty() {
            println!("{}: PASS", name);
        } else {
            println!("{}: FAIL ({})", name, errors.len());
            for e in &errors[..errors.len().min(3)] { println!("  {}", e); }
            all_ok = false;
        }
    }
    assert!(all_ok);
}
