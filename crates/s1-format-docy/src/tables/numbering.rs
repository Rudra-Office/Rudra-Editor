use crate::constants::num;
use crate::writer::DocyWriter;
use s1_model::DocumentModel;

pub fn has_content(model: &DocumentModel) -> bool {
    !model.numbering().abstract_nums.is_empty()
}

pub fn write(w: &mut DocyWriter, model: &DocumentModel) {
    let len_pos = w.begin_length_block();
    let numbering = model.numbering();

    // Abstract numbering definitions
    w.write_item(num::ABSTRACT_NUMS, |w| {
        for abs_num in &numbering.abstract_nums {
            w.write_item(num::ABSTRACT_NUM, |w| {
                // AbstractNum_Id: sdkjs uses WriteItem (Read1)
                w.write_item(num::ABSTRACT_NUM_ID, |w| {
                    w.write_long(abs_num.abstract_num_id);
                });

                // Levels: Read1 container → each Lvl uses Read2 internally
                w.write_item(num::ABSTRACT_NUM_LVLS, |w| {
                    for level in &abs_num.levels {
                        w.write_item(num::LVL, |w| {
                            // Inside Lvl, everything is Read2 (properties)
                            // Number format
                            let fmt = match level.num_format {
                                s1_model::ListFormat::Bullet => 23,
                                s1_model::ListFormat::Decimal => 0,
                                s1_model::ListFormat::LowerAlpha => 4,
                                s1_model::ListFormat::UpperAlpha => 3,
                                s1_model::ListFormat::LowerRoman => 2,
                                s1_model::ListFormat::UpperRoman => 1,
                                _ => 0,
                            };
                            w.write_prop_long(num::LVL_FORMAT, fmt);
                            w.write_prop_long(num::LVL_START, level.start);

                            // Level text: Read2 Variable → Read1 content inside
                            w.write_prop_item(num::LVL_TEXT, |w| {
                                write_level_text_items(w, &level.level_text);
                            });

                            // Level alignment: Read2 Byte
                            if let Some(a) = level.alignment {
                                let val = match a {
                                    s1_model::Alignment::Right => crate::constants::align::RIGHT,
                                    s1_model::Alignment::Left => crate::constants::align::LEFT,
                                    s1_model::Alignment::Center => crate::constants::align::CENTER,
                                    s1_model::Alignment::Justify => crate::constants::align::JUSTIFY,
                                    _ => crate::constants::align::LEFT,
                                };
                                w.write_prop_byte(37, val); // lvl_Jc
                            }

                            // Bullet font: Read2 Variable → Read2 rPr content
                            if let Some(ref font) = level.bullet_font {
                                w.write_prop_item(num::LVL_TEXT_PR, |w| {
                                    w.write_prop_string2(
                                        crate::constants::rpr::FONT_ASCII,
                                        font,
                                    );
                                    w.write_prop_string2(
                                        crate::constants::rpr::FONT_HANSI,
                                        font,
                                    );
                                });
                            }

                            // Level paragraph properties: Read2 Variable → Read2 pPr content
                            if let Some(indent) = level.indent_left {
                                w.write_prop_item(num::LVL_PARA_PR, |w| {
                                    let twips = crate::constants::pts_to_twips(indent);
                                    w.write_prop_long_signed(
                                        crate::constants::ppr::IND_LEFT_TWIPS,
                                        twips,
                                    );
                                    if let Some(hanging) = level.indent_hanging {
                                        let h_twips = crate::constants::pts_to_twips(hanging);
                                        w.write_prop_long_signed(
                                            crate::constants::ppr::IND_FIRST_LINE_TWIPS,
                                            -h_twips,
                                        );
                                    }
                                });
                            }
                        });
                    }
                });
            });
        }
    });

    // Numbering instances: Read1 container
    w.write_item(num::NUMS, |w| {
        for inst in &numbering.instances {
            w.write_item(num::NUM, |w| {
                // Num_ANumId / Num_NumId: sdkjs uses Read2 (WriteByte+lenType+WriteLong)
                w.write_prop_long(num::NUM_ANUM_ID, inst.abstract_num_id);
                w.write_prop_long(num::NUM_NUM_ID, inst.num_id);
            });
        }
    });

    w.end_length_block(len_pos);
}

/// Parse level text like "%1." into Read1 text/num items.
fn write_level_text_items(w: &mut DocyWriter, text: &str) {
    let mut chars = text.chars().peekable();
    let mut buf = String::new();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if !buf.is_empty() {
                // LvlTextItem (Read1) → LvlTextItemText (Read1 string)
                w.write_item(num::LVL_TEXT_ITEM, |w| {
                    w.write_string_item(num::LVL_TEXT_ITEM_TEXT, &buf);
                });
                buf.clear();
            }
            if let Some(&digit) = chars.peek() {
                if digit.is_ascii_digit() {
                    chars.next();
                    let lvl = (digit as u8 - b'1') as u8;
                    // LvlTextItem (Read1) → LvlTextItemNum (Read1 item containing a byte)
                    w.write_item(num::LVL_TEXT_ITEM, |w| {
                        w.write_item(num::LVL_TEXT_ITEM_NUM, |w| {
                            w.write_byte(lvl);
                        });
                    });
                    continue;
                }
            }
            buf.push('%');
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() {
        w.write_item(num::LVL_TEXT_ITEM, |w| {
            w.write_string_item(num::LVL_TEXT_ITEM_TEXT, &buf);
        });
    }
}
