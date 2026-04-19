use crate::constants::hdr_ftr;
use crate::writer::DocyWriter;
use crate::content;
use s1_model::{DocumentModel, NodeType, NodeId, HeaderFooterType};

/// Collected header/footer entries with their DOCY type and global index.
pub struct HdrFtrEntry {
    pub node_id: NodeId,
    pub docy_type: u8, // HdrFtr_First=2, HdrFtr_Even=3, HdrFtr_Odd=4
}

/// Collect all headers and footers from all sections into flat lists.
/// Returns (headers, footers) — the index in each list is what section properties reference.
pub fn collect(model: &DocumentModel) -> (Vec<HdrFtrEntry>, Vec<HdrFtrEntry>) {
    let mut headers = Vec::new();
    let mut footers = Vec::new();

    for sec in model.sections() {
        for hf_ref in &sec.headers {
            let dt = match hf_ref.hf_type {
                HeaderFooterType::First => hdr_ftr::FIRST,
                HeaderFooterType::Even => hdr_ftr::EVEN,
                HeaderFooterType::Default => hdr_ftr::ODD,
                _ => hdr_ftr::ODD,
            };
            headers.push(HdrFtrEntry { node_id: hf_ref.node_id, docy_type: dt });
        }
        for hf_ref in &sec.footers {
            let dt = match hf_ref.hf_type {
                HeaderFooterType::First => hdr_ftr::FIRST,
                HeaderFooterType::Even => hdr_ftr::EVEN,
                HeaderFooterType::Default => hdr_ftr::ODD,
                _ => hdr_ftr::ODD,
            };
            footers.push(HdrFtrEntry { node_id: hf_ref.node_id, docy_type: dt });
        }
    }

    (headers, footers)
}

pub fn has_content(model: &DocumentModel) -> bool {
    let (h, f) = collect(model);
    !h.is_empty() || !f.is_empty()
}

pub fn write(w: &mut DocyWriter, model: &DocumentModel) {
    let len_pos = w.begin_length_block();
    let (headers, footers) = collect(model);

    // Write headers
    if !headers.is_empty() {
        w.write_item(hdr_ftr::HEADER, |w| {
            for entry in &headers {
                w.write_item(entry.docy_type, |w| {
                    w.write_item(hdr_ftr::CONTENT, |w| {
                        write_hdr_ftr_content(w, model, entry.node_id);
                    });
                });
            }
        });
    }

    // Write footers
    if !footers.is_empty() {
        w.write_item(hdr_ftr::FOOTER, |w| {
            for entry in &footers {
                w.write_item(entry.docy_type, |w| {
                    w.write_item(hdr_ftr::CONTENT, |w| {
                        write_hdr_ftr_content(w, model, entry.node_id);
                    });
                });
            }
        });
    }

    w.end_length_block(len_pos);
}

fn write_hdr_ftr_content(w: &mut DocyWriter, model: &DocumentModel, node_id: NodeId) {
    let node = match model.node(node_id) {
        Some(n) => n,
        None => return,
    };
    for child_id in &node.children {
        if let Some(child) = model.node(*child_id) {
            match child.node_type {
                NodeType::Paragraph => {
                    w.write_item(crate::constants::par::PAR, |w| {
                        content::paragraph::write(w, model, *child_id);
                    });
                }
                NodeType::Table => {
                    w.write_item(crate::constants::par::TABLE, |w| {
                        content::table::write(w, model, *child_id);
                    });
                }
                _ => {}
            }
        }
    }
}
