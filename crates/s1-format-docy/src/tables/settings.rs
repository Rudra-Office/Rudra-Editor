use crate::constants::settings;
use crate::writer::DocyWriter;
use s1_model::DocumentModel;

pub fn write(w: &mut DocyWriter, model: &DocumentModel) {
    let len_pos = w.begin_length_block();

    // Default tab stop — WriteItem (Read1)
    w.write_item(settings::DEFAULT_TAB_STOP_TWIPS, |w| {
        w.write_long(720); // 720 twips = 36pt = 0.5 inch
    });

    // Track revisions — WriteItem (Read1)
    if model.metadata().track_changes {
        w.write_item(settings::TRACK_REVISIONS, |w| {
            w.write_bool(true);
        });
    }

    w.end_length_block(len_pos);
}
