import sys

def rewrite_file():
    with open('ffi/wasm/src/lib.rs', 'r') as f:
        content = f.read()

    old_str = """        // Compute new position: need to figure out the text node at the insertion point
        // For now, use the anchor position advanced by inserted text length
        let earlier_pos = if range.anchor.offset_utf16 <= range.focus.offset_utf16
            && range.anchor.node_id == range.focus.node_id
        {
            &range.anchor
        } else {
            // Multi-node: use whichever resolved to start_para/start_offset
            let (ap, _) = resolve_position_to_paragraph(&model, &range.anchor)
                .unwrap_or((start_para, start_offset));
            if ap == start_para {
                &range.anchor
            } else {
                &range.focus
            }
        };

        let inserted_utf16_len: u32 = text.chars().map(|c| c.len_utf16() as u32).sum();
        let new_pos = PositionRefParsed {
            node_id: earlier_pos.node_id,
            offset_utf16: earlier_pos.offset_utf16 + inserted_utf16_len,
            affinity: "downstream".to_string(),
        };

        let doc = self.doc()?;"""

    new_str = """        let doc = self.doc()?;
        
        let inserted_char_len = text.chars().count();
        let new_char_offset = start_offset + inserted_char_len;
        
        let new_pos = match find_text_node_at_char_offset(doc.model(), start_para, new_char_offset) {
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
