import sys

def rewrite_file():
    with open('ffi/wasm/src/lib.rs', 'r') as f:
        content = f.read()

    old_str = """        if new_para_child_idx == 0 {
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
        }"""

    new_str = """        if new_para_child_idx == 0 {
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
        
        if split_child_idx == 0 && local_offset == 0 {
            let empty_run_id = doc.next_id();
            let mut empty_run = Node::new(empty_run_id, NodeType::Run);
            if let Some(&first_run_id) = para_children.iter().find(|&&c| {
                doc.node(c).map(|n| n.node_type == NodeType::Run).unwrap_or(false)
            }) {
                if let Some(first_run) = doc.node(first_run_id) {
                    empty_run.attributes = first_run.attributes.clone();
                }
            }
            txn.push(Operation::insert_node(para_id, 0, empty_run));
            txn.push(Operation::insert_node(empty_run_id, 0, Node::text(doc.next_id(), "")));
        }"""

    if old_str not in content:
        print("Old string not found")
        sys.exit(1)
        
    new_content = content.replace(old_str, new_str)
    with open('ffi/wasm/src/lib.rs', 'w') as f:
        f.write(new_content)

rewrite_file()
