use crate::constants::*;
use crate::writer::DocyWriter;
use crate::content;
use crate::tables::headers_footers;
use s1_model::{DocumentModel, NodeType, NodeId};

pub fn write(w: &mut DocyWriter, model: &DocumentModel) {
    let len_pos = w.begin_length_block();

    let body_id = match model.body_id() {
        Some(id) => id,
        None => { w.end_length_block(len_pos); return; }
    };
    let body = match model.node(body_id) {
        Some(n) => n,
        None => { w.end_length_block(len_pos); return; }
    };

    let (all_headers, all_footers) = headers_footers::collect(model);
    let sections = model.sections();
    let last_sec_idx = if sections.is_empty() { None } else { Some(sections.len() - 1) };

    // Build a map: child_id → section index (for paragraphs that end a section)
    // A paragraph with SectionIndex X that differs from the previous means
    // it's the LAST paragraph in section X, and section props go in its pPr.
    let children: Vec<NodeId> = body.children.clone();
    let mut para_section: Vec<Option<usize>> = vec![None; children.len()];
    let mut last_seen_idx: Option<usize> = None;
    for (i, child_id) in children.iter().enumerate() {
        if let Some(child) = model.node(*child_id) {
            if child.node_type == NodeType::Paragraph {
                if let Some(s1_model::AttributeValue::Int(idx)) = child.attributes.get(&s1_model::AttributeKey::SectionIndex) {
                    let idx = *idx as usize;
                    if Some(idx) != last_seen_idx && Some(idx) != last_sec_idx {
                        // This paragraph ends a mid-document section
                        para_section[i] = Some(idx);
                    }
                    last_seen_idx = Some(idx);
                }
            }
        }
    }

    // Write each body child
    for (i, child_id) in children.iter().enumerate() {
        let child = match model.node(*child_id) {
            Some(n) => n,
            None => continue,
        };

        match child.node_type {
            NodeType::Paragraph => {
                let sect = para_section[i].and_then(|idx| sections.get(idx));
                w.write_item(par::PAR, |w| {
                    content::paragraph::write_with_section(
                        w, model, *child_id, sect, &all_headers, &all_footers,
                    );
                });
            }
            NodeType::Table => {
                w.write_item(par::TABLE, |w| {
                    content::table::write(w, model, *child_id);
                });
            }
            NodeType::TableOfContents => {
                w.write_item(par::SDT, |w| {
                    content::toc::write(w, model, *child_id);
                });
            }
            _ => {}
        }
    }

    // Write LAST section as top-level sectPr (document-level section)
    // If the last section has no headers/footers, inherit from the first section that does
    if let Some(last_sec) = sections.last() {
        let mut effective_sec = last_sec.clone();
        if effective_sec.headers.is_empty() {
            if let Some(donor) = sections.iter().find(|s| !s.headers.is_empty()) {
                effective_sec.headers = donor.headers.clone();
            }
        }
        if effective_sec.footers.is_empty() {
            if let Some(donor) = sections.iter().find(|s| !s.footers.is_empty()) {
                effective_sec.footers = donor.footers.clone();
            }
        }
        w.write_item(par::SECT_PR, |w| {
            content::section::write_with_hdr_ftr(w, &effective_sec, &all_headers, &all_footers);
        });
    }

    w.end_length_block(len_pos);
}
