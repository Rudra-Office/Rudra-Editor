use s1engine::Engine;

#[test]
fn find_byte_1773() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let docy = s1_format_docy::write(model);

    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD, parts[3]
    ).unwrap();

    // Print table layout
    let count = binary[0] as usize;
    for i in 0..count {
        let pos = 1 + i * 5;
        let t = binary[pos];
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize;
        let names = ["Sig","Info","Media","Num","HdrFtr","Style","Doc","Other","Cmts","Settings","Fn","En"];
        let name = names.get(t as usize).unwrap_or(&"?");
        if t != 0 {
            let tlen = u32::from_le_bytes([binary[off], binary[off+1], binary[off+2], binary[off+3]]) as usize;
            println!("Table {}: {} ({}) offset={} len={} end={}", i, t, name, off, tlen, off+4+tlen);
        } else {
            println!("Table {}: {} ({}) offset={}", i, t, name, off);
        }
    }

    // Dump bytes around 1773
    println!("\n--- Bytes around 1773 ---");
    let start = 1750;
    let end = 1800.min(binary.len());
    for i in start..end {
        if (i - start) % 20 == 0 { print!("\n  {:5}: ", i); }
        print!("{:02x} ", binary[i]);
    }
    println!();

    // Walk Style table to find which style item contains byte 1773
    let style_off = (0..count).find_map(|i| {
        let pos = 1 + i * 5;
        if binary[pos] == 5 { Some(u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize) }
        else { None }
    }).unwrap();

    let style_len = u32::from_le_bytes([binary[style_off], binary[style_off+1], binary[style_off+2], binary[style_off+3]]) as usize;
    println!("\nStyle table: offset={} len={}", style_off, style_len);

    // Read1 walk of style table
    let mut cur = style_off + 4;
    let end = style_off + 4 + style_len;
    let mut elem = 0;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        let contains_1773 = cur <= 1773 && 1773 < cur + 5 + len;
        println!("  Style top @{}: type={} len={} end={} {}", cur, typ, len, cur+5+len,
            if contains_1773 { "<-- CONTAINS 1773" } else { "" });

        // If this is the STYLES list (type=2), walk into it
        if typ == 2 && len > 0 {
            let mut scur = cur + 5;
            let send = cur + 5 + len;
            let mut sidx = 0;
            while scur + 5 <= send {
                let styp = binary[scur];
                let slen = u32::from_le_bytes([binary[scur+1], binary[scur+2], binary[scur+3], binary[scur+4]]) as usize;
                let contains = scur <= 1773 && 1773 < scur + 5 + slen;
                if contains || sidx < 3 {
                    println!("    Style#{} @{}: type={} len={} end={} {}",
                        sidx, scur, styp, slen, scur+5+slen,
                        if contains { "<-- CONTAINS 1773" } else { "" });

                    // Walk into the style that contains 1773
                    if contains {
                        let mut icur = scur + 5;
                        let iend = scur + 5 + slen;
                        while icur + 5 <= iend {
                            let ityp = binary[icur];
                            let ilen = u32::from_le_bytes([binary[icur+1], binary[icur+2], binary[icur+3], binary[icur+4]]) as usize;
                            let ic = icur <= 1773 && 1773 < icur + 5 + ilen;
                            println!("      field @{}: type={} len={} end={} {}",
                                icur, ityp, ilen, icur+5+ilen,
                                if ic { "<-- 1773 IS HERE" } else { "" });
                            icur = icur + 5 + ilen;
                        }
                    }
                }
                sidx += 1;
                scur = scur + 5 + slen;
            }
            println!("    ({} styles total)", sidx);
        }

        elem += 1;
        cur = cur + 5 + len;
    }
}
