use crate::constants::*;
use crate::writer::DocyWriter;
use s1_model::{PageOrientation, SectionBreakType, SectionProperties};
use crate::tables::headers_footers::HdrFtrEntry;

/// Write section properties with optional header/footer index references.
pub fn write_with_hdr_ftr(
    w: &mut DocyWriter,
    sec: &SectionProperties,
    all_headers: &[HdrFtrEntry],
    all_footers: &[HdrFtrEntry],
) {
    w.write_item(sec_pr::PG_SZ, |w| write_page_size(w, sec));
    w.write_item(sec_pr::PG_MAR, |w| write_page_margins(w, sec));
    w.write_item(sec_pr::SETTINGS, |w| write_settings(w, sec));

    if sec.columns > 1 || !sec.equal_width {
        w.write_item(sec_pr::COLS, |w| write_columns(w, sec));
    }

    // Header references — indices into the flat headers array
    if !sec.headers.is_empty() {
        w.write_item(sec_pr::HEADERS, |w| {
            for hf_ref in &sec.headers {
                // Find index of this header in the flat array
                if let Some(idx) = all_headers.iter().position(|e| e.node_id == hf_ref.node_id) {
                    w.write_item(sec_pr::HDR_FTR_ELEM, |w| {
                        w.write_long(idx as u32);
                    });
                }
            }
        });
    }

    // Footer references — indices into the flat footers array
    if !sec.footers.is_empty() {
        w.write_item(sec_pr::FOOTERS, |w| {
            for hf_ref in &sec.footers {
                if let Some(idx) = all_footers.iter().position(|e| e.node_id == hf_ref.node_id) {
                    w.write_item(sec_pr::HDR_FTR_ELEM, |w| {
                        w.write_long(idx as u32);
                    });
                }
            }
        });
    }
}

/// Write section properties without header/footer references.
#[allow(dead_code)]
pub fn write(w: &mut DocyWriter, sec: &SectionProperties) {
    write_with_hdr_ftr(w, sec, &[], &[]);
}

fn write_page_size(w: &mut DocyWriter, sec: &SectionProperties) {
    w.write_prop_long(sec_pg_sz::W_TWIPS, pts_to_twips(sec.page_width) as u32);
    w.write_prop_long(sec_pg_sz::H_TWIPS, pts_to_twips(sec.page_height) as u32);

    let orient = match sec.orientation {
        PageOrientation::Portrait => 0,
        PageOrientation::Landscape => 1,
        _ => 0,
    };
    w.write_prop_byte(sec_pg_sz::ORIENTATION, orient);
}

fn write_page_margins(w: &mut DocyWriter, sec: &SectionProperties) {
    w.write_prop_long(sec_pg_mar::LEFT_TWIPS, pts_to_twips(sec.margin_left) as u32);
    w.write_prop_long(sec_pg_mar::TOP_TWIPS, pts_to_twips(sec.margin_top) as u32);
    w.write_prop_long(sec_pg_mar::RIGHT_TWIPS, pts_to_twips(sec.margin_right) as u32);
    w.write_prop_long(sec_pg_mar::BOTTOM_TWIPS, pts_to_twips(sec.margin_bottom) as u32);
    w.write_prop_long(sec_pg_mar::HEADER_TWIPS, pts_to_twips(sec.header_distance) as u32);
    w.write_prop_long(sec_pg_mar::FOOTER_TWIPS, pts_to_twips(sec.footer_distance) as u32);
    w.write_prop_long(sec_pg_mar::GUTTER_TWIPS, 0);
}

fn write_settings(w: &mut DocyWriter, sec: &SectionProperties) {
    if sec.title_page {
        w.write_prop_bool(sec_settings::TITLE_PG, true);
    }
    if sec.even_and_odd_headers {
        w.write_prop_bool(sec_settings::EVEN_AND_ODD_HEADERS, true);
    }
    if let Some(section_type) = section_type_byte(sec.break_type) {
        w.write_prop_byte(sec_settings::SECTION_TYPE, section_type);
    }
}

fn write_columns(w: &mut DocyWriter, sec: &SectionProperties) {
    w.write_item(sec_columns::EQUAL_WIDTH, |w| w.write_bool(sec.equal_width));
    w.write_item(sec_columns::NUM, |w| w.write_long(sec.columns));
    w.write_item(sec_columns::SPACE, |w| {
        w.write_long(pts_to_twips(sec.column_spacing) as u32);
    });
}

fn section_type_byte(break_type: Option<SectionBreakType>) -> Option<u8> {
    match break_type {
        Some(SectionBreakType::Continuous) => Some(0),
        Some(SectionBreakType::EvenPage) => Some(1),
        Some(SectionBreakType::NextPage) => Some(3),
        Some(SectionBreakType::OddPage) => Some(4),
        Some(_) | None => None,
    }
}
