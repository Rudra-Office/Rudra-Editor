use s1engine::Engine;
use s1_model::{NodeType, AttributeKey, AttributeValue};

#[test]
fn check_table_widths() {
    let engine = Engine::new();
    let bytes = std::fs::read("/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx").unwrap();
    let doc = engine.open(&bytes).unwrap();
    let model = doc.model();
    let body_id = model.body_id().unwrap();
    let body = model.node(body_id).unwrap();

    for (i, cid) in body.children.iter().enumerate() {
        if let Some(n) = model.node(*cid) {
            if n.node_type == NodeType::Table {
                println!("\nTable at body[{}]:", i);
                // Table width
                if let Some(tw) = n.attributes.get(&AttributeKey::TableWidth) {
                    println!("  TableWidth: {:?}", tw);
                }
                if let Some(cw) = n.attributes.get_string(&AttributeKey::TableColumnWidths) {
                    println!("  ColumnWidths: {:?}", cw);
                }
                // Check rows and cells
                for (ri, rid) in n.children.iter().enumerate() {
                    if let Some(row) = model.node(*rid) {
                        if row.node_type != NodeType::TableRow { continue; }
                        if ri > 1 { continue; } // Just first 2 rows
                        print!("  Row[{}]: ", ri);
                        for (ci, cid) in row.children.iter().enumerate() {
                            if let Some(cell) = model.node(*cid) {
                                if cell.node_type != NodeType::TableCell { continue; }
                                let w = cell.attributes.get(&AttributeKey::CellWidth);
                                let span = cell.attributes.get(&AttributeKey::ColSpan);
                                print!("Cell{}({:?} span={:?}) ", ci, w, span);
                            }
                        }
                        println!();
                    }
                }
            }
        }
    }
}
