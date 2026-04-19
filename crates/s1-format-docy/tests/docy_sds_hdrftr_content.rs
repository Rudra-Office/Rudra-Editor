use s1engine::Engine;
use s1_model::{NodeType, AttributeKey};

#[test]
fn check_header_footer_content() {
    let files = [
        "/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx",
        "/Users/sachin/Downloads/Aruljothi.docx",
    ];
    for path in &files {
        let engine = Engine::new();
        let bytes = std::fs::read(path).unwrap();
        let doc = engine.open(&bytes).unwrap();
        let model = doc.model();
        let name = path.rsplit('/').next().unwrap();
        println!("\n=== {} ===", name);
        
        // Check section header/footer refs
        for (si, sec) in model.sections().iter().enumerate() {
            if !sec.headers.is_empty() || !sec.footers.is_empty() {
                println!("Section {}: {} headers, {} footers", si, sec.headers.len(), sec.footers.len());
                for hf in &sec.headers {
                    if let Some(node) = model.node(hf.node_id) {
                        println!("  Header({:?}): {} children", hf.hf_type, node.children.len());
                        for cid in &node.children {
                            if let Some(c) = model.node(*cid) {
                                let text: String = c.children.iter()
                                    .filter_map(|id| model.node(*id))
                                    .filter_map(|n| {
                                        if n.node_type == NodeType::Run {
                                            n.children.iter()
                                                .filter_map(|id| model.node(*id))
                                                .filter_map(|t| t.text_content.clone())
                                                .collect::<Vec<_>>()
                                                .join("")
                                                .into()
                                        } else { None }
                                    })
                                    .collect::<Vec<_>>()
                                    .join("");
                                println!("    {:?}: {:?}", c.node_type, &text[..text.len().min(60)]);
                            }
                        }
                    }
                }
                for hf in &sec.footers {
                    if let Some(node) = model.node(hf.node_id) {
                        println!("  Footer({:?}): {} children", hf.hf_type, node.children.len());
                        for cid in &node.children {
                            if let Some(c) = model.node(*cid) {
                                let has_field = c.children.iter().any(|id| 
                                    model.node(*id).map_or(false, |n| n.node_type == NodeType::Field)
                                );
                                println!("    {:?} children={} hasField={}", c.node_type, c.children.len(), has_field);
                            }
                        }
                    }
                }
            }
        }
    }
}
