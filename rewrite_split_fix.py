import sys

def rewrite_file():
    with open('ffi/wasm/src/lib.rs', 'r') as f:
        content = f.read()

    old_snippet = """        if split_child_idx < para_children.len() {
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
                }"""

    new_snippet = """        if split_child_idx < para_children.len() {
            let split_child_id = para_children[split_child_idx];
            let split_child_node = doc.node(split_child_id).unwrap();
            let split_child_type = split_child_node.node_type;
            let split_child_attrs = split_child_node.attributes.clone();
            let split_child_children = split_child_node.children.clone();
            
            if local_offset == 0 {
                for &child_id in para_children.iter().skip(split_child_idx) {
                    txn.push(Operation::move_node(child_id, new_para_id, new_para_child_idx));
                    new_para_child_idx += 1;
                }
            } else if split_child_type == NodeType::Run {
                let mut new_run = Node::new(doc.next_id(), NodeType::Run);
                new_run.attributes = split_child_attrs;
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
                
                let text_idx = split_child_children.iter().position(|&c| c == text_node_id).unwrap();
                for &sub_id in split_child_children.iter().skip(text_idx + 1) {
                    txn.push(Operation::move_node(sub_id, new_run_id, new_run_child_idx));
                    new_run_child_idx += 1;
                }"""

    content = content.replace(old_snippet, new_snippet)
    with open('ffi/wasm/src/lib.rs', 'w') as f:
        f.write(content)

rewrite_file()
