use s1engine::Engine;

fn test_file(path: &str) -> (usize, usize) {
    let engine = Engine::new();
    let bytes = std::fs::read(path).expect(&format!("file not found: {}", path));
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let docy = s1_format_docy::write(model);

    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD, parts[3]
    ).unwrap();

    // Walk all tables checking for stream misalignment
    let count = binary[0] as usize;
    let mut errors = 0;
    for i in 0..count {
        let pos = 1 + i * 5;
        let t = binary[pos];
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize;
        let next_off = if i + 1 < count {
            u32::from_le_bytes([binary[1+(i+1)*5+1], binary[1+(i+1)*5+2], binary[1+(i+1)*5+3], binary[1+(i+1)*5+4]]) as usize
        } else { binary.len() };

        // Signature table (type=0) uses Read2, not Read1, and sdkjs skips it — skip validation
        if t == 0 { continue; }

        // Each table starts with a 4-byte length block
        if off + 4 > binary.len() { errors += 1; continue; }
        let tbl_len = u32::from_le_bytes([binary[off], binary[off+1], binary[off+2], binary[off+3]]) as usize;

        // Walk Read1 items
        let mut cur = off + 4;
        let end = off + 4 + tbl_len;
        let mut elems = 0;
        while cur + 5 <= end && cur + 5 <= binary.len() {
            let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
            if cur + 5 + len > binary.len() {
                let names = ["Sig","Info","Media","Num","HdrFtr","Style","Doc","Other","Cmts","Settings","Fn","En"];
                let name = names.get(t as usize).unwrap_or(&"?");
                println!("  TABLE {} ({}) @{}: type={} len={} OVERFLOW (binary={})", t, name, cur, binary[cur], len, binary.len());
                errors += 1;
                break;
            }
            elems += 1;
            cur = cur + 5 + len;
        }
        if cur != end {
            let names = ["Sig","Info","Media","Num","HdrFtr","Style","Doc","Other","Cmts","Settings","Fn","En"];
            let name = names.get(t as usize).unwrap_or(&"?");
            println!("  TABLE {} ({}) MISALIGN: cur={} end={} (elems={})", t, name, cur, end, elems);
            errors += 1;
        }
    }

    // Count doc table elements
    let doc_idx = (0..count).find(|i| binary[1 + i*5] == 6);
    let doc_elems = if let Some(di) = doc_idx {
        let off = u32::from_le_bytes([binary[1+di*5+1], binary[1+di*5+2], binary[1+di*5+3], binary[1+di*5+4]]) as usize;
        let len = u32::from_le_bytes([binary[off], binary[off+1], binary[off+2], binary[off+3]]) as usize;
        let mut cur = off + 4;
        let end = off + 4 + len;
        let mut n = 0;
        while cur + 5 <= end {
            let l = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
            n += 1;
            cur = cur + 5 + l;
        }
        n
    } else { 0 };

    println!("{}: {} chars, {} bytes, {} tables, {} doc elements, {} errors",
        path.rsplit('/').next().unwrap_or(path), docy.len(), binary.len(), count, doc_elems, errors);
    (doc_elems, errors)
}

#[test]
fn chat_reaction() {
    let (elems, errors) = test_file("/Users/sachin/Downloads/Chat Reaction.docx");
    assert_eq!(errors, 0, "stream errors");
    assert!(elems > 10, "should have many elements, got {}", elems);
}

#[test]
fn sds_anti_zh() {
    let (elems, errors) = test_file("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx");
    assert_eq!(errors, 0, "stream errors");
    assert!(elems > 10, "should have many elements, got {}", elems);
}

#[test]
fn aruljothi() {
    let (elems, errors) = test_file("/Users/sachin/Downloads/Aruljothi.docx");
    assert_eq!(errors, 0, "stream errors");
    assert!(elems > 10, "should have many elements, got {}", elems);
}

#[test]
fn nishtriya() {
    let (elems, errors) = test_file("/Users/sachin/Downloads/Nishtriya.docx");
    assert_eq!(errors, 0, "stream errors");
    assert!(elems > 10, "should have many elements, got {}", elems);
}
