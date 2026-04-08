import sys

def rewrite_file():
    with open('ffi/wasm/src/lib.rs', 'r') as f:
        content = f.read()

    old_str = """        // Resolve the start position back to text-node coordinates for the result.
        // After deletion, the document has changed, so we use the anchor position
        // (or focus, whichever is earlier) as the new collapsed cursor.
        let new_pos = if range.anchor.node_id == range.focus.node_id {
            let min_offset = range.anchor.offset_utf16.min(range.focus.offset_utf16);
            PositionRefParsed {
                node_id: range.anchor.node_id,
                offset_utf16: min_offset,
                affinity: "downstream".to_string(),
            }
        } else {
            // Multi-paragraph: cursor collapses to the start (anchor or focus)
            let (anchor_para, _) = resolve_position_to_paragraph(&model, &range.anchor)?;
            let (focus_para, _) = resolve_position_to_paragraph(&model, &range.focus)?;
            if let Some(bid) = model.body_id() {
                if let Some(body) = model.node(bid) {
                    let ai = body.children.iter().position(|&c| c == anchor_para);
                    let fi = body.children.iter().position(|&c| c == focus_para);
                    match (ai, fi) {
                        (Some(a), Some(f)) if a <= f => range.anchor.to_parsed_clone(),
                        (Some(_), Some(_)) => range.focus.to_parsed_clone(),
                        _ => range.anchor.to_parsed_clone(),
                    }
                } else {
                    range.anchor.to_parsed_clone()
                }
            } else {
                range.anchor.to_parsed_clone()
            }
        };

        let doc = self.doc()?;"""

    new_str = """        let doc = self.doc()?;

        // Resolve the start position back to text-node coordinates using the mutated model.
        let new_pos = match find_text_node_at_char_offset(doc.model(), start_para, start_offset) {
            Ok((tid, local_off, _)) => {
                let text_node = doc.model().node(tid).unwrap();
                let text_content = text_node.text_content.as_deref().unwrap_or("");
                let utf16_off = char_offset_to_utf16_offset(text_content, local_off);
                PositionRefParsed {
                    node_id: tid,
                    offset_utf16: utf16_off,
                    affinity: "downstream".to_string(),
                }
            }
            Err(_) => {
                if let Ok((tid, _)) = find_first_text_node(doc.model(), start_para) {
                    PositionRefParsed {
                        node_id: tid,
                        offset_utf16: 0,
                        affinity: "downstream".to_string(),
                    }
                } else {
                    PositionRefParsed {
                        node_id: start_para,
                        offset_utf16: 0,
                        affinity: "downstream".to_string(),
                    }
                }
            }
        };"""

    if old_str not in content:
        print("Old string not found")
        sys.exit(1)
        
    new_content = content.replace(old_str, new_str)
    with open('ffi/wasm/src/lib.rs', 'w') as f:
        f.write(new_content)

rewrite_file()
