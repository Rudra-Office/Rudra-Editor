use crate::constants::*;
use crate::writer::DocyWriter;
use crate::props;
use s1_model::DocumentModel;

pub fn write(w: &mut DocyWriter, model: &DocumentModel) {
    let len_pos = w.begin_length_block();

    // Default paragraph properties
    w.write_item(style_table::DEF_PPR, |w| {
        let defaults = model.doc_defaults();
        props::para_props::write_defaults(w, defaults);
    });

    // Default run properties
    w.write_item(style_table::DEF_RPR, |w| {
        let defaults = model.doc_defaults();
        props::run_props::write_defaults(w, defaults);
    });

    // All styles — include model styles + required defaults
    w.write_item(style_table::STYLES, |w| {
        // Write model styles first
        for s in model.styles() {
            write_style(w, s);
        }

        // If model has no styles, generate minimum defaults sdkjs needs
        if model.styles().is_empty() {
            write_default_normal_style(w, model);
            write_heading_styles(w);
        }
    });

    w.end_length_block(len_pos);
}

/// Write "Normal" default paragraph style — sdkjs requires this.
fn write_default_normal_style(w: &mut DocyWriter, model: &DocumentModel) {
    w.write_item(style::STYLE, |w| {
        w.write_string_item(style::STYLE_ID, "Normal");
        w.write_string_item(style::STYLE_NAME, "Normal");
        // Style fields use Read1 (WriteItem), NOT Read2 (WriteProp)
        w.write_item(style::STYLE_TYPE, |w| w.write_byte(3)); // paragraph
        w.write_item(style::STYLE_DEFAULT, |w| w.write_bool(true));
        w.write_item(style::STYLE_Q_FORMAT, |w| w.write_bool(true));
        w.write_item(style::STYLE_UI_PRIORITY, |w| w.write_long(0));

        // Default paragraph props
        w.write_item(style::STYLE_PARA_PR, |w| {
            let defaults = model.doc_defaults();
            props::para_props::write_defaults(w, defaults);
        });

        // Default run props (font, size)
        w.write_item(style::STYLE_TEXT_PR, |w| {
            let defaults = model.doc_defaults();
            props::run_props::write_defaults(w, defaults);
        });
    });
}

/// Write heading styles (Heading 1-6).
fn write_heading_styles(w: &mut DocyWriter) {
    for level in 1..=6u8 {
        w.write_item(style::STYLE, |w| {
            let id = format!("Heading{}", level);
            let name = format!("heading {}", level);
            w.write_string_item(style::STYLE_ID, &id);
            w.write_string_item(style::STYLE_NAME, &name);
            w.write_item(style::STYLE_TYPE, |w| w.write_byte(3)); // paragraph
            w.write_string_item(style::STYLE_BASED_ON, "Normal");
            w.write_string_item(style::STYLE_NEXT, "Normal");
            w.write_item(style::STYLE_Q_FORMAT, |w| w.write_bool(true));
            w.write_item(style::STYLE_UI_PRIORITY, |w| w.write_long(9));

            // Paragraph props: outline level
            w.write_item(style::STYLE_PARA_PR, |w| {
                w.write_prop_long(ppr::OUTLINE_LVL, (level - 1) as u32);
                w.write_prop_bool(ppr::KEEP_NEXT, true);
                w.write_prop_bool(ppr::KEEP_LINES, true);
            });

            // Run props: bold + larger font
            w.write_item(style::STYLE_TEXT_PR, |w| {
                w.write_prop_bool(rpr::BOLD, true);
                let size = match level {
                    1 => 32, 2 => 26, 3 => 24, 4 => 22, 5 => 20, _ => 20,
                };
                w.write_prop_long(rpr::FONT_SIZE, size); // half-points
            });
        });
    }
}

fn write_style(w: &mut DocyWriter, s: &s1_model::Style) {
    w.write_item(style::STYLE, |w| {
        // Style ID (WriteString2)
        w.write_string_item(style::STYLE_ID, &s.id);

        // Style name (WriteString2)
        w.write_string_item(style::STYLE_NAME, &s.name);

        // Type — WriteItem (Read1), NOT WriteProp (Read2)
        let type_byte = match s.style_type {
            s1_model::StyleType::Character => 1,
            s1_model::StyleType::List => 2,
            s1_model::StyleType::Paragraph => 3,
            s1_model::StyleType::Table => 4, _ => 3,
        };
        w.write_item(style::STYLE_TYPE, |w| w.write_byte(type_byte));

        // Based on (WriteString2)
        if let Some(ref parent) = s.parent_id {
            w.write_string_item(style::STYLE_BASED_ON, parent);
        }

        // Next style (WriteString2)
        if let Some(ref next) = s.next_style_id {
            w.write_string_item(style::STYLE_NEXT, next);
        }

        // Default flag — WriteItem (Read1)
        if s.is_default {
            w.write_item(style::STYLE_DEFAULT, |w| w.write_bool(true));
        }

        // qFormat — WriteItem (Read1)
        w.write_item(style::STYLE_Q_FORMAT, |w| w.write_bool(true));

        // Paragraph properties (WriteItem → content uses Read2 for props)
        w.write_item(style::STYLE_PARA_PR, |w| {
            props::para_props::write(w, &s.attributes);
        });

        // Run/text properties (WriteItem → content uses Read2 for props)
        w.write_item(style::STYLE_TEXT_PR, |w| {
            props::run_props::write(w, &s.attributes);
        });
    });
}
