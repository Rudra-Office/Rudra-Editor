use crate::constants::*;
use crate::writer::DocyWriter;
use crate::props::borders;
use s1_model::{AttributeKey, AttributeMap, AttributeValue, Alignment, LineSpacing, DocumentDefaults, TabAlignment, TabLeader};

/// Write paragraph properties (pPr) from an attribute map.
pub fn write(w: &mut DocyWriter, attrs: &AttributeMap) {
    // Alignment
    if let Some(AttributeValue::Alignment(a)) = attrs.get(&AttributeKey::Alignment) {
        let val = match a {
            Alignment::Right => align::RIGHT,
            Alignment::Left => align::LEFT,
            Alignment::Center => align::CENTER,
            Alignment::Justify => align::JUSTIFY,
            _ => align::LEFT, 
        };
        w.write_prop_byte(ppr::JC, val);
    }

    // Indentation — must be wrapped in Ind container (pPrType.Ind=1)
    let has_indent = attrs.get_f64(&AttributeKey::IndentLeft).is_some()
        || attrs.get_f64(&AttributeKey::IndentRight).is_some()
        || attrs.get_f64(&AttributeKey::IndentFirstLine).is_some();
    if has_indent {
        w.write_prop_item(ppr::IND, |w| {
            if let Some(v) = attrs.get_f64(&AttributeKey::IndentLeft) {
                w.write_prop_long_signed(ppr::IND_LEFT_TWIPS, pts_to_twips(v));
            }
            if let Some(v) = attrs.get_f64(&AttributeKey::IndentRight) {
                w.write_prop_long_signed(ppr::IND_RIGHT_TWIPS, pts_to_twips(v));
            }
            if let Some(v) = attrs.get_f64(&AttributeKey::IndentFirstLine) {
                w.write_prop_long_signed(ppr::IND_FIRST_LINE_TWIPS, pts_to_twips(v));
            }
        });
    }

    // Spacing
    let has_spacing = attrs.get_f64(&AttributeKey::SpacingBefore).is_some()
        || attrs.get_f64(&AttributeKey::SpacingAfter).is_some()
        || attrs.get(&AttributeKey::LineSpacing).is_some();

    if has_spacing {
        w.write_prop_item(ppr::SPACING, |w| {
            if let Some(v) = attrs.get_f64(&AttributeKey::SpacingBefore) {
                w.write_prop_long_signed(spacing::BEFORE, pts_to_twips(v));
            }
            if let Some(v) = attrs.get_f64(&AttributeKey::SpacingAfter) {
                w.write_prop_long_signed(spacing::AFTER, pts_to_twips(v));
            }
            if let Some(ls) = attrs.get_line_spacing(&AttributeKey::LineSpacing) {
                match ls {
                    LineSpacing::Single => {
                        w.write_prop_long(spacing::LINE, 240);
                        w.write_prop_byte(spacing::LINE_RULE, 0); // auto
                    }
                    LineSpacing::OnePointFive => {
                        w.write_prop_long(spacing::LINE, 360);
                        w.write_prop_byte(spacing::LINE_RULE, 0);
                    }
                    LineSpacing::Double => {
                        w.write_prop_long(spacing::LINE, 480);
                        w.write_prop_byte(spacing::LINE_RULE, 0);
                    }
                    LineSpacing::Multiple(v) => {
                        w.write_prop_long(spacing::LINE, (v * 240.0) as u32);
                        w.write_prop_byte(spacing::LINE_RULE, 0);
                    }
                    LineSpacing::Exact(v) => {
                        w.write_prop_long(spacing::LINE, pts_to_twips(v) as u32);
                        w.write_prop_byte(spacing::LINE_RULE, 1); // exact
                    }
                    LineSpacing::AtLeast(v) => {
                        w.write_prop_long(spacing::LINE, pts_to_twips(v) as u32);
                        w.write_prop_byte(spacing::LINE_RULE, 2); // atLeast
                    }
                    _ => { w.write_prop_long(spacing::LINE, 240); w.write_prop_byte(spacing::LINE_RULE, 0); }
                }
            }
        });
    }

    // Keep lines / keep next / page break before / widow control
    if let Some(true) = attrs.get_bool(&AttributeKey::KeepLinesTogether) {
        w.write_prop_bool(ppr::KEEP_LINES, true);
    }
    if let Some(true) = attrs.get_bool(&AttributeKey::KeepWithNext) {
        w.write_prop_bool(ppr::KEEP_NEXT, true);
    }
    if let Some(true) = attrs.get_bool(&AttributeKey::PageBreakBefore) {
        w.write_prop_bool(ppr::PAGE_BREAK_BEFORE, true);
    }
    if let Some(true) = attrs.get_bool(&AttributeKey::WidowControl) {
        w.write_prop_bool(ppr::WIDOW_CONTROL, true);
    }

    // Paragraph style
    if let Some(style) = attrs.get_string(&AttributeKey::StyleId) {
        w.write_prop_string2(ppr::PARA_STYLE, style);
    }

    // List/numbering
    if let Some(AttributeValue::ListInfo(li)) = attrs.get(&AttributeKey::ListInfo) {
        w.write_prop_item(ppr::NUM_PR, |w| {
            w.write_prop_long(ppr::NUM_PR_LVL, li.level as u32);
            w.write_prop_long(ppr::NUM_PR_ID, li.num_id);
        });
    }

    // Outline level — sdkjs uses Long (GetLongLE), not Byte
    if let Some(AttributeValue::Int(lvl)) = attrs.get(&AttributeKey::OutlineLevel) {
        if *lvl >= 0 && *lvl <= 8 {
            w.write_prop_long(ppr::OUTLINE_LVL, *lvl as u32);
        }
    }

    // Bidi
    if let Some(true) = attrs.get_bool(&AttributeKey::Bidi) {
        w.write_prop_bool(ppr::BIDI, true);
    }

    // Contextual spacing
    if let Some(true) = attrs.get_bool(&AttributeKey::ContextualSpacing) {
        w.write_prop_bool(ppr::CONTEXTUAL_SPACING, true);
    }

    // Background/shading — Read2 format: ShdType.Color(1) + lenType=Three(3) + RGB
    if let Some(AttributeValue::Color(c)) = attrs.get(&AttributeKey::Background) {
        w.write_prop_item(ppr::SHD, |w| {
            // c_oSerShdType.Value = 0, Byte
            w.write_prop_byte(0, 1); // ShdClear
            // c_oSerShdType.Color = 1, Three bytes (RGB)
            w.write_byte(1);  // type
            w.write_byte(3);  // lenType = Three
            w.write_color_rgb(c.r, c.g, c.b);
        });
    }

    // Tab stops
    if let Some(tabs) = attrs.get_tab_stops(&AttributeKey::TabStops) {
        if !tabs.is_empty() {
            w.write_prop_item(ppr::TAB, |w| {
                for tab in tabs {
                    w.write_prop_item(ppr::TAB_ITEM, |w| {
                        // Position in twips
                        w.write_prop_long(ppr::TAB_ITEM_POS_TWIPS, pts_to_twips(tab.position) as u32);
                        // Value (tab type): Left=8, Center=1, Right=7, Decimal=3
                        let val = match tab.alignment {
                            TabAlignment::Left => 8u8,
                            TabAlignment::Center => 1,
                            TabAlignment::Right => 7,
                            TabAlignment::Decimal => 3,
                            _ => 8,
                        };
                        w.write_prop_byte(ppr::TAB_ITEM_VAL, val);
                        // Leader: Dot=0, Heavy=1, Hyphen=2, MiddleDot=3, None=4, Underscore=5
                        let leader = match tab.leader {
                            TabLeader::Dot => 0u8,
                            TabLeader::Dash => 2,
                            TabLeader::Underscore => 5,
                            TabLeader::None => 4,
                            _ => 4,
                        };
                        w.write_prop_byte(ppr::TAB_ITEM_LEADER, leader);
                    });
                }
            });
        }
    }

    // Paragraph borders
    if let Some(brd) = attrs.get_borders(&AttributeKey::ParagraphBorders) {
        w.write_prop_item(ppr::PBDR, |w| {
            borders::write_borders(w, brd);
        });
    }
}

/// Write default paragraph properties from document defaults.
pub fn write_defaults(w: &mut DocyWriter, defaults: &DocumentDefaults) {
    if let Some(v) = defaults.space_after {
        w.write_prop_item(ppr::SPACING, |w| {
            w.write_prop_long_signed(spacing::AFTER, pts_to_twips(v));
        });
    }
    if let Some(v) = defaults.line_spacing_multiple {
        w.write_prop_item(ppr::SPACING, |w| {
            w.write_prop_long(spacing::LINE, (v * 240.0) as u32);
            w.write_prop_byte(spacing::LINE_RULE, 0);
        });
    }
}
