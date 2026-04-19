use s1engine::Engine;

#[test]
fn trace_table_at_32096() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let docy = s1_format_docy::write(doc.model());
    let parts: Vec<&str> = docy.splitn(4, ';').collect();
    let bin = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[3]).unwrap();

    // Table at element #66. Find its offset.
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
    // Skip to element 66
    while cur + 5 <= end && idx < 66 {
        let len = u32::from_le_bytes([bin[cur+1],bin[cur+2],bin[cur+3],bin[cur+4]]) as usize;
        cur += 5 + len;
        idx += 1;
    }
    let tbl_start = cur;
    let tbl_type = bin[cur];
    let tbl_len = u32::from_le_bytes([bin[cur+1],bin[cur+2],bin[cur+3],bin[cur+4]]) as usize;
    println!("Table @{}: type={} len={}", tbl_start, tbl_type, tbl_len);

    // Walk table Read1 items
    cur = tbl_start + 5;
    let tbl_end = tbl_start + 5 + tbl_len;
    while cur + 5 <= tbl_end {
        let t = bin[cur];
        let l = u32::from_le_bytes([bin[cur+1],bin[cur+2],bin[cur+3],bin[cur+4]]) as usize;
        let contains = cur <= 32096 && 32096 < cur + 5 + l;
        let tnames = ["tblPr","tblGrid","?","Content","Row","RowCont","Cell","CellPr","CellCont"];
        let name = tnames.get(t as usize).unwrap_or(&"?");
        if contains || (cur + 5 + l > 32090 && cur < 32110) {
            println!("  @{}: {}({}) len={} {}", cur, name, t, l, if contains {"<< CONTAINS 32096"} else {""});
        }
        // Recurse into Content/Row/RowContent/Cell that might contain 32096
        if contains && (t == 3 || t == 4 || t == 5 || t == 6) {
            let inner_start = cur + 5;
            let inner_end = cur + 5 + l;
            let mut icur = inner_start;
            while icur + 5 <= inner_end {
                let it = bin[icur];
                let il = u32::from_le_bytes([bin[icur+1],bin[icur+2],bin[icur+3],bin[icur+4]]) as usize;
                let ic = icur <= 32096 && 32096 < icur + 5 + il;
                let iname = tnames.get(it as usize).unwrap_or(&"?");
                if ic || (icur + 5 + il > 32090 && icur < 32110) {
                    println!("    @{}: {}({}) len={} {}", icur, iname, it, il, if ic {"<< HAS IT"} else {""});
                    // One more level deep
                    if ic && (it == 4 || it == 5 || it == 6 || it == 7 || it == 8) {
                        let jstart = icur + 5;
                        let jend = icur + 5 + il;
                        let mut jcur = jstart;
                        while jcur + 5 <= jend {
                            let jt = bin[jcur];
                            let jl = u32::from_le_bytes([bin[jcur+1],bin[jcur+2],bin[jcur+3],bin[jcur+4]]) as usize;
                            let jc = jcur <= 32096 && 32096 < jcur + 5 + jl;
                            let jname = tnames.get(jt as usize).unwrap_or(&"?");
                            if jc || (jcur >= 32090 && jcur <= 32110) {
                                println!("      @{}: {}({}) len={} {}", jcur, jname, jt, jl, if jc {"<< HERE"} else {""});
                                if jc {
                                    // Hex dump
                                    let from = (32090 as usize).min(bin.len());
                                    let to = (32110 as usize).min(bin.len());
                                    print!("      hex@32090: ");
                                    for k in from..to { print!("{:02x} ", bin[k]); }
                                    println!();
                                }
                            }
                            jcur += 5 + jl;
                        }
                    }
                }
                icur += 5 + il;
            }
        }
        cur += 5 + l;
    }
}
