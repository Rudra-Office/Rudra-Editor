import sys

def rewrite_file():
    with open('ffi/wasm/src/lib.rs', 'r') as f:
        content = f.read()

    start_str = """    pub fn split_paragraph(
        &mut self,
        node_id_str: &str,
        char_offset: usize,
    ) -> Result<String, JsError> {"""

    end_str = """        doc.apply_transaction(&txn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(format!("{}:{}", new_para_id.replica, new_para_id.counter))
    }"""

    new_code = """    pub fn split_paragraph(
        &mut self,
        node_id_str: &str,
        char_offset: usize,
    ) -> Result<String, JsError> {
        let doc = self.doc_mut()?;
        let para_id = parse_node_id(node_id_str)?;

        let para = doc
            .node(para_id)
            .ok_or_else(|| JsError::new("Paragraph not found"))?;
        let parent_id = para.parent.ok_or_else(|| JsError::new("No parent"))?;
        let parent = doc.node(parent_id).ok_or_else(|| JsError::new("Parent not found"))?;
        let index = parent.children.iter().position(|&c| c == para_id).unwrap();
        let para_children = para.children.clone();
        let para_attributes = para.attributes.clone();

        let mut accumulated = 0usize;
        let mut split_child_idx = para_children.len();
        let mut local_offset = 0usize;
        for (i, &child_id) in para_children.iter().enumerate() {
            if let Some(child) = doc.node(child_id) {
                let clen = match child.node_type {
                    NodeType::Run => run_char_len(doc.model(), child_id),
                    NodeType::LineBreak | NodeType::Tab => 1,
                    _ => 0,
                };
                if char_offset <= accumulated + clen {
                    split_child_idx = i;
                    local_offset = char_offset - accumulated;
                    break;
                }
                accumulated += clen;
            }
        }

        let new_para_id = doc.next_id();
        let mut txn = Transaction::with_label("Split paragraph");
        let mut new_para = Node::new(new_para_id, NodeType::Paragraph);
        new_para.attributes = para_attributes;
        txn.push(Operation::insert_node(parent_id, index + 1, new_para));
        
        let mut new_para_child_idx = 0;
        
        if split_child_idx < para_children.len() {
            let split_child_id = para_children[split_child_idx];
            let split_child_node = doc.node(split_child_id).unwrap();
            let split_child_type = split_child_node.node_type;
            
            if local_offset == 0 {
                for &child_id in para_children.iter().skip(split_child_idx) {
                    txn.push(Operation::move_node(child_id, new_para_id, new_para_child_idx));
                    new_para_child_idx += 1;
                }
            } else if split_child_type == NodeType::Run {
                let mut new_run = Node::new(doc.next_id(), NodeType::Run);
                new_run.attributes = split_child_node.attributes.clone();
                let new_run_id = new_run.id;
                txn.push(Operation::insert_node(new_para_id, new_para_child_idx, new_run));
                new_para_child_idx += 1;
                
                let (text_node_id, local_off, text_len) =
                    find_text_node_at_char_offset_in_run(doc.model(), split_child_id, local_offset)?;
                
                let mut new_run_child_idx = 0;
                if local_off < text_len {
                    let text_node = doc.node(text_node_id).unwrap();
                    let full_text = text_node.text_content.as_deref().unwrap_or("");
                    let tail_text: String = full_text.chars().skip(local_off).collect();
                    
                    txn.push(Operation::delete_text(text_node_id, local_off, text_len - local_off));
                    
                    let new_text_id = doc.next_id();
                    txn.push(Operation::insert_node(new_run_id, new_run_child_idx, Node::text(new_text_id, &tail_text)));
                    new_run_child_idx += 1;
                }
                
                let text_idx = split_child_node.children.iter().position(|&c| c == text_node_id).unwrap();
                for &sub_id in split_child_node.children.iter().skip(text_idx + 1) {
                    txn.push(Operation::move_node(sub_id, new_run_id, new_run_child_idx));
                    new_run_child_idx += 1;
                }
                
                for &child_id in para_children.iter().skip(split_child_idx + 1) {
                    txn.push(Operation::move_node(child_id, new_para_id, new_para_child_idx));
                    new_para_child_idx += 1;
                }
            } else {
                for &child_id in para_children.iter().skip(split_child_idx + 1) {
                    txn.push(Operation::move_node(child_id, new_para_id, new_para_child_idx));
                    new_para_child_idx += 1;
                }
            }
        }
        
        if new_para_child_idx == 0 {
            let new_run_id = doc.next_id();
            let mut new_run = Node::new(new_run_id, NodeType::Run);
            if let Some(&last_run_id) = para_children.iter().rev().find(|&&c| {
                doc.node(c).map(|n| n.node_type == NodeType::Run).unwrap_or(false)
            }) {
                if let Some(last_run) = doc.node(last_run_id) {
                    new_run.attributes = last_run.attributes.clone();
                }
            }
            txn.push(Operation::insert_node(new_para_id, 0, new_run));
            txn.push(Operation::insert_node(new_run_id, 0, Node::text(doc.next_id(), "")));
        }

        doc.apply_transaction(&txn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(format!("{}:{}", new_para_id.replica, new_para_id.counter))
    }"""

    start_idx = content.find(start_str)
    if start_idx == -1:
        print("Start string not found")
        sys.exit(1)
        
    end_idx = content.find(end_str, start_idx)
    if end_idx == -1:
        print("End string not found")
        sys.exit(1)
        
    end_idx += len(end_str)
    
    new_content = content[:start_idx] + new_code + content[end_idx:]
    with open('ffi/wasm/src/lib.rs', 'w') as f:
        f.write(new_content)

rewrite_file()
