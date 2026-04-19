use crate::constants::*;
use crate::writer::DocyWriter;
use s1_model::{BorderSide, BorderStyle, Borders};

/// Write a Borders struct (Read1 format: each side is an item with border properties inside).
pub fn write_borders(w: &mut DocyWriter, borders: &Borders) {
    if let Some(ref side) = borders.left {
        w.write_item(borders_type::LEFT, |w| write_border(w, side));
    }
    if let Some(ref side) = borders.top {
        w.write_item(borders_type::TOP, |w| write_border(w, side));
    }
    if let Some(ref side) = borders.right {
        w.write_item(borders_type::RIGHT, |w| write_border(w, side));
    }
    if let Some(ref side) = borders.bottom {
        w.write_item(borders_type::BOTTOM, |w| write_border(w, side));
    }
}

/// Write a single border side's properties (Read2 format: type+lenType+value).
fn write_border(w: &mut DocyWriter, side: &BorderSide) {
    // Color (3 raw RGB bytes in a Variable-length property)
    w.write_prop_item(border_type::COLOR, |w| {
        w.write_byte(side.color.r);
        w.write_byte(side.color.g);
        w.write_byte(side.color.b);
    });

    // Space in points (as u32 twips)
    let space_pts = (side.spacing * 8.0).round() as u32;
    w.write_prop_long(border_type::SPACE_POINT, space_pts);

    // Size in 1/8 points
    let size_8pt = (side.width * 8.0).round() as u32;
    w.write_prop_long(border_type::SIZE_8POINT, size_8pt);

    // Border value (line style byte for legacy)
    let val = border_style_to_byte(side.style);
    w.write_prop_byte(border_type::VALUE, val);

    // Border value type (u32 for extended types)
    w.write_prop_long(border_type::VALUE_TYPE, border_style_to_value_type(side.style));
}

/// Map BorderStyle to the legacy byte value (c_oSerBorderType.Value).
fn border_style_to_byte(style: BorderStyle) -> u8 {
    match style {
        BorderStyle::None => 0,    // border_None
        BorderStyle::Single => 1,  // border_Single
        BorderStyle::Double => 3,  // border_Double
        BorderStyle::Dashed => 2,  // border_Dashed — mapped to dashSmallGap
        BorderStyle::Dotted => 2,  // border_Dotted — mapped close
        BorderStyle::Thick => 1,   // border_Thick — single with larger width
        _ => 0,
    }
}

/// Map BorderStyle to the extended value type (c_oSerBorderType.ValueType).
/// These map to OOXML ST_Border values.
fn border_style_to_value_type(style: BorderStyle) -> u32 {
    match style {
        BorderStyle::None => 0,     // none
        BorderStyle::Single => 1,   // single
        BorderStyle::Double => 7,   // double
        BorderStyle::Dashed => 3,   // dashed
        BorderStyle::Dotted => 2,   // dotted
        BorderStyle::Thick => 12,   // thick
        _ => 0,
    }
}
