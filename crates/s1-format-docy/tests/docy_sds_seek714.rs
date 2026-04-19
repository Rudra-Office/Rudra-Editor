use s1engine::Engine;

#[test]
fn check_seek714() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let docy = s1_format_docy::write(model);

    // Parse header like sdkjs does
    let chars: Vec<char> = docy.chars().collect();
    let mut semicolons = 0;
    let mut version_str = String::new();
    let mut size_str = String::new();
    let mut b64_start = 0;
    let mut section = 0; // 0=sig, 1=version, 2=size
    for (i, &c) in chars.iter().enumerate() {
        if c == ';' {
            semicolons += 1;
            section = semicolons;
            if semicolons == 3 {
                b64_start = i + 1;
                break;
            }
            continue;
        }
        match section {
            1 => version_str.push(c),
            2 => size_str.push(c),
            _ => {}
        }
    }
    println!("Version: {:?}, Size: {:?}, b64_start: {}", version_str, size_str, b64_start);

    let b64_data = &docy[b64_start..];
    let binary = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD, b64_data
    ).unwrap();
    println!("Binary length: {}", binary.len());

    // Check main table header
    let count = binary[0];
    println!("Table count: {}", count);
    for i in 0..count as usize {
        let pos = 1 + i * 5;
        let t = binary[pos];
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]);
        println!("  Entry {}: type={} offset={}", i, t, off);
    }

    // Simulate sdkjs Seek(714) + ReadTable
    let seek_pos = 714usize;
    println!("\nSimulate Seek(714):");
    println!("  binary[714..718] = {:02x} {:02x} {:02x} {:02x}", binary[714], binary[715], binary[716], binary[717]);
    let table_len = u32::from_le_bytes([binary[714], binary[715], binary[716], binary[717]]);
    println!("  ReadTable length = {}", table_len);
    println!("  EnterFrame({}) check: pos=718, size={}, fits={}", table_len, binary.len(), 718 + table_len as usize <= binary.len());

    // Check the Numbering table too since its end should be at 714
    println!("\nNumbering table end check:");
    let num_off = 59usize;
    let num_len = u32::from_le_bytes([binary[num_off], binary[num_off+1], binary[num_off+2], binary[num_off+3]]) as usize;
    println!("  Num offset={}, len={}, end={}", num_off, num_len, num_off + 4 + num_len);
    println!("  binary[{}..{}] = {:02x} {:02x} {:02x} {:02x}", num_off+4+num_len, num_off+4+num_len+4,
        binary[num_off+4+num_len], binary[num_off+4+num_len+1], binary[num_off+4+num_len+2], binary[num_off+4+num_len+3]);

    // After Numbering pre-read, where does stream.cur end up?
    // ReadTable: EnterFrame(4) → cur=59, pos=63. GetULongLE reads 4 bytes from cur=59.
    // Read1 processes num_len bytes. Then cur should be at 59 + 4 + num_len.
    println!("\n  After Numbering ReadTable, expected cur = {}", num_off + 4 + num_len);
    println!("  This matches Style offset? {}", num_off + 4 + num_len == 714);
    
    // But Seek was called between Numbering and Style!
    // Seek(36) for Signature in aSeekTable - this sets pos=36 but NOT cur
    // Signature: break (no read)
    // Then Seek(714) for Style - pos=714
    // EnterFrame(4) → cur=pos=714
    // So Style reading SHOULD start from 714
    
    // Let me check: what if there's an issue with how EnterFrame works?
    // After Numbering, cur = 59 + 4 + num_len = 714, pos = ???
    // After Numbering's ReadTable: EnterFrame(4) set pos = 63. Then EnterFrame(num_len) set pos = 63 + num_len.
    // Then Read1 processes. At the end Read1 does... hmm Read1 doesn't Seek.
    // So pos = 63 + num_len = 63 + 651 = 714 after ReadTable's second EnterFrame.
    // But Read1 reads stuff, advancing cur. At the end, stCurPos should equal stLen.
    // So cur = 63 + 4 (from GetULongLE) + num_len (from Read1 consuming stLen bytes)
    // Wait, EnterFrame(stLen) sets cur=pos=63+4... no.
    
    // Let me trace more carefully:
    // Seek(59): pos=59
    // ReadTable:
    //   EnterFrame(4): cur=59, pos=63
    //   GetULongLE: reads cur[59..63]=num_len, cur=63
    //   EnterFrame(num_len): cur=63, pos=63+num_len=714
    //   Read1(num_len): reads from cur=63 for num_len bytes, cur advances to 63+num_len=714
    // After ReadTable: cur=714, pos=714

    // Then aSeekTable loop:
    // Seek(36): pos=36, cur=714 (unchanged)
    // Signature case: break (no EnterFrame, no read)
    // Seek(714): pos=714, cur=714 (unchanged)
    // ReadTable for Style:
    //   EnterFrame(4): cur=pos=714, pos=718
    //   GetULongLE: reads binary[714..718], cur=718
    //   This should give style_len = 2220
    //   EnterFrame(2220): cur=718, pos=718+2220=2938
    //   Read1(2220): processes from cur=718

    // Hmm, but wait. The Seek for Signature at 36: Seek only sets pos=36. Then for the next table:
    // Seek(714): pos=714.
    // EnterFrame(4): cur=pos=714.
    // So cur IS correctly set to 714. Style reading should work.

    // BUT: what if the Signature "break" case somehow messes up? Let me check if there's
    // any other code that runs for Signature. case Signature: break; — that's it.
    
    // What if the issue is that `length` variable in the aSeekTable loop gets reused?
    // Looking at the loop code:
    // for(var i = 0, length = aSeekTable.length; i < length; ++i)
    // The loop variable is named `length`! And inside ReadTable, there might also be a
    // variable named `length` for the table length. Let me check if this causes shadowing.
}
