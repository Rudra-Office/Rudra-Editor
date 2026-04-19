use s1engine::Engine;
use s1_model::NodeType;

#[test]
fn find_sds_tables_and_element66() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let body_id = model.body_id().unwrap();
    let body = model.node(body_id).unwrap();

    // Find tables and special elements
    for (i, cid) in body.children.iter().enumerate() {
        if let Some(n) = model.node(*cid) {
            if n.node_type == NodeType::Table {
                let rows = n.children.iter().filter(|id| 
                    model.node(**id).map_or(false, |n| n.node_type == NodeType::TableRow)
                ).count();
                println!("TABLE at body[{}]: {} rows", i, rows);
            }
            // Also print around element 60-70
            if i >= 60 && i <= 72 {
                let has_sec = n.attributes.get(&s1_model::AttributeKey::SectionIndex).is_some();
                let has_list = n.attributes.get(&s1_model::AttributeKey::ListInfo).is_some();
                let has_tabs = n.attributes.get_tab_stops(&s1_model::AttributeKey::TabStops).map_or(false, |t| !t.is_empty());
                println!("body[{}] {:?} children={} sec={} list={} tabs={}", 
                    i, n.node_type, n.children.len(), has_sec, has_list, has_tabs);
            }
        }
    }
}
