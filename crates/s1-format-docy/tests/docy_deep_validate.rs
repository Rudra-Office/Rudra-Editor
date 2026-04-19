use s1engine::Engine;

/// Recursively validate DOCY binary structure matching sdkjs Read1/Read2 patterns.
/// Returns error count.
fn validate_read1(binary: &[u8], start: usize, length: usize, path: &str, errors: &mut Vec<String>) {
    let mut cur = start;
    let end = start + length;
    let mut elem = 0;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > binary.len() {
            errors.push(format!("{} @{}: type={} len={} OVERFLOW (max={})", path, cur, typ, len, binary.len()));
            return;
        }
        if cur + 5 + len > end {
            errors.push(format!("{} @{}: type={} len={} EXCEEDS BLOCK END (end={})", path, cur, typ, len, end));
            return;
        }
        // Recurse into known Read1 containers
        let content_start = cur + 5;
        let inner_path = format!("{}[{}]", path, typ);
        if len > 0 {
            validate_read1_item(binary, content_start, len, typ, &inner_path, path, errors);
        }
        elem += 1;
        cur = cur + 5 + len;
    }
    if cur != end {
        errors.push(format!("{}: MISALIGN cur={} end={} after {} items", path, cur, end, elem));
    }
}

fn validate_read2(binary: &[u8], start: usize, length: usize, path: &str, errors: &mut Vec<String>) {
    let mut cur = start;
    let end = start + length;
    while cur < end {
        if cur >= binary.len() {
            errors.push(format!("{} @{}: READ2 past end of binary", path, cur));
            return;
        }
        let typ = binary[cur];
        cur += 1;
        if cur >= end { break; }
        let len_type = binary[cur];
        cur += 1;
        match len_type {
            0 => {} // Null - no value
            1 => { cur += 1; } // Byte
            2 => { cur += 2; } // Short
            3 => { cur += 3; } // Three
            4 => { cur += 4; } // Long
            5 => { cur += 8; } // Double
            6 => { // Variable
                if cur + 4 > binary.len() {
                    errors.push(format!("{} @{}: READ2 Variable len past end", path, cur));
                    return;
                }
                let vlen = u32::from_le_bytes([binary[cur], binary[cur+1], binary[cur+2], binary[cur+3]]) as usize;
                cur += 4;
                if cur + vlen > binary.len() {
                    errors.push(format!("{} @{}: READ2 type={} Variable len={} OVERFLOW", path, cur, typ, vlen));
                    return;
                }
                cur += vlen;
            }
            _ => {
                errors.push(format!("{} @{}: READ2 type={} INVALID lenType={}", path, cur-2, typ, len_type));
                return;
            }
        }
    }
}

fn validate_read1_item(binary: &[u8], start: usize, length: usize, typ: u8, path: &str, parent: &str, errors: &mut Vec<String>) {
    // Document table (parent="Doc") top-level items
    if parent == "Doc" {
        match typ {
            0 => validate_paragraph(binary, start, length, path, errors), // Par
            3 => validate_read1(binary, start, length, path, errors), // Table (has tblPr, grid, content)
            4 => validate_read1(binary, start, length, path, errors), // SectPr → Read1 with Read2 children
            15 => validate_read1(binary, start, length, path, errors), // Sdt
            6 | 7 => {} // CommentStart/End
            23 | 24 => {} // BookmarkStart/End
            10 => validate_read1(binary, start, length, path, errors), // Hyperlink
            _ => {}
        }
    }
    // Inside paragraph
    else if parent.ends_with("[0]") && parent.contains("Doc") {
        // Par children: pPr(1)=Read2, Content(2)=Read1
        match typ {
            1 => validate_read2(binary, start, length, path, errors), // pPr
            2 => validate_read1(binary, start, length, path, errors), // Content
            _ => {}
        }
    }
    // Style table items
    else if parent == "Style" {
        match typ {
            0 => validate_read2(binary, start, length, path, errors), // DEF_PPR (Read2)
            1 => validate_read2(binary, start, length, path, errors), // DEF_RPR (Read2)
            2 => validate_styles_list(binary, start, length, path, errors), // STYLES (Read1)
            _ => {}
        }
    }
}

fn validate_paragraph(binary: &[u8], start: usize, length: usize, path: &str, errors: &mut Vec<String>) {
    let mut cur = start;
    let end = start + length;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end {
            errors.push(format!("{} @{}: Para child type={} len={} EXCEEDS (end={})", path, cur, typ, len, end));
            return;
        }
        match typ {
            1 => validate_read2(binary, cur+5, len, &format!("{}->pPr", path), errors), // pPr
            2 => validate_para_content(binary, cur+5, len, &format!("{}->Content", path), errors), // Content
            _ => {}
        }
        cur = cur + 5 + len;
    }
    if cur != end {
        errors.push(format!("{}: Para MISALIGN cur={} end={}", path, cur, end));
    }
}

fn validate_para_content(binary: &[u8], start: usize, length: usize, path: &str, errors: &mut Vec<String>) {
    let mut cur = start;
    let end = start + length;
    let mut elem = 0;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end {
            errors.push(format!("{} @{}: Content item type={} len={} EXCEEDS (end={}) elem={}", path, cur, typ, len, end, elem));
            return;
        }
        // Validate runs
        if typ == 5 { // Run
            validate_run(binary, cur+5, len, &format!("{}->[Run#{}]", path, elem), errors);
        }
        // Validate hyperlinks
        if typ == 10 { // Hyperlink
            validate_read1(binary, cur+5, len, &format!("{}->[Hyp#{}]", path, elem), errors);
        }
        // Validate bookmark/comment markers
        if typ == 6 || typ == 7 || typ == 23 || typ == 24 {
            validate_read1(binary, cur+5, len, &format!("{}->[Marker#{}:{}]", path, elem, typ), errors);
        }
        elem += 1;
        cur = cur + 5 + len;
    }
    if cur != end {
        errors.push(format!("{}: Content MISALIGN cur={} end={} after {} items", path, cur, end, elem));
    }
}

fn validate_run(binary: &[u8], start: usize, length: usize, path: &str, errors: &mut Vec<String>) {
    let mut cur = start;
    let end = start + length;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end {
            errors.push(format!("{} @{}: Run child type={} len={} EXCEEDS (end={})", path, cur, typ, len, end));
            return;
        }
        match typ {
            1 => validate_read2(binary, cur+5, len, &format!("{}->rPr", path), errors), // rPr
            8 => {} // Content (text) - raw bytes
            _ => {}
        }
        cur = cur + 5 + len;
    }
    if cur != end {
        errors.push(format!("{}: Run MISALIGN cur={} end={}", path, cur, end));
    }
}

fn validate_styles_list(binary: &[u8], start: usize, length: usize, path: &str, errors: &mut Vec<String>) {
    let mut cur = start;
    let end = start + length;
    let mut elem = 0;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end {
            errors.push(format!("{} @{}: Style#{} type={} len={} EXCEEDS (end={})", path, cur, elem, typ, len, end));
            return;
        }
        if typ == 0 { // Individual style
            validate_single_style(binary, cur+5, len, &format!("{}->Style#{}", path, elem), errors);
        }
        elem += 1;
        cur = cur + 5 + len;
    }
    if cur != end {
        errors.push(format!("{}: Styles MISALIGN cur={} end={} after {} styles", path, cur, end, elem));
    }
}

fn validate_single_style(binary: &[u8], start: usize, length: usize, path: &str, errors: &mut Vec<String>) {
    // Style items are Read1
    let mut cur = start;
    let end = start + length;
    while cur + 5 <= end {
        let typ = binary[cur];
        let len = u32::from_le_bytes([binary[cur+1], binary[cur+2], binary[cur+3], binary[cur+4]]) as usize;
        if cur + 5 + len > end {
            errors.push(format!("{} @{}: StyleField type={} len={} EXCEEDS (end={})", path, cur, typ, len, end));
            return;
        }
        // ParaPr(6) and TextPr(5) contain Read2 properties
        match typ {
            5 => validate_read2(binary, cur+5, len, &format!("{}->TextPr", path), errors),
            6 => validate_read2(binary, cur+5, len, &format!("{}->ParaPr", path), errors),
            _ => {} // Other style fields are simple Read1 values
        }
        cur = cur + 5 + len;
    }
    if cur != end {
        errors.push(format!("{}: Style MISALIGN cur={} end={}", path, cur, end));
    }
}

fn test_file(name: &str, path: &str) {
    let engine = Engine::new();
    let bytes = std::fs::read(path).expect(&format!("file not found: {}", path));
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let docy = s1_format_docy::write(model);

    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let binary = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD, parts[3]
    ).unwrap();

    let count = binary[0] as usize;
    let mut all_errors = Vec::new();

    for i in 0..count {
        let pos = 1 + i * 5;
        let t = binary[pos];
        let off = u32::from_le_bytes([binary[pos+1], binary[pos+2], binary[pos+3], binary[pos+4]]) as usize;

        // Skip Signature (sdkjs ignores it)
        if t == 0 { continue; }

        if off + 4 > binary.len() {
            all_errors.push(format!("Table {} offset {} past binary end", t, off));
            continue;
        }
        let tbl_len = u32::from_le_bytes([binary[off], binary[off+1], binary[off+2], binary[off+3]]) as usize;

        let tbl_name = match t {
            3 => "Num", 4 => "HdrFtr", 5 => "Style", 6 => "Doc",
            7 => "Other", 8 => "Comments", 9 => "Settings",
            10 => "Footnotes", 11 => "Endnotes", _ => "Unknown"
        };

        validate_read1(&binary, off + 4, tbl_len, tbl_name, &mut all_errors);
    }

    if all_errors.is_empty() {
        println!("{}: PASS ({} bytes, {} tables)", name, binary.len(), count);
    } else {
        println!("{}: FAIL ({} errors)", name, all_errors.len());
        for e in &all_errors {
            println!("  {}", e);
        }
        panic!("{}: {} encoding errors found", name, all_errors.len());
    }
}

#[test]
fn deep_validate_chat_reaction() {
    test_file("Chat Reaction", "/Users/sachin/Downloads/Chat Reaction.docx");
}

#[test]
fn deep_validate_sds() {
    test_file("SDS_ANTI-T_ZH", "/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx");
}

#[test]
fn deep_validate_aruljothi() {
    test_file("Aruljothi", "/Users/sachin/Downloads/Aruljothi.docx");
}

#[test]
fn deep_validate_nishtriya() {
    test_file("Nishtriya", "/Users/sachin/Downloads/Nishtriya.docx");
}
