//! s1-format-docy — OnlyOffice DOCY binary format writer.
//!
//! Transforms s1engine DocumentModel into DOCY binary format that
//! sdkjs BinaryFileReader can parse natively.

mod constants;
mod writer;
pub mod tables;
mod props;
pub mod content;

use base64::engine::Engine as _;
use s1_model::DocumentModel;

/// Write a DocumentModel as a DOCY binary string.
pub fn write(model: &DocumentModel) -> String {
    let mut w = writer::DocyWriter::new();

    let has_numbering = tables::numbering::has_content(model);
    let has_hdrftr = tables::headers_footers::has_content(model);
    let has_comments = tables::comments::has_content(model);
    let has_footnotes = tables::footnotes::has_content(model);
    let has_endnotes = tables::endnotes::has_content(model);

    let mut table_count: u8 = 5; // Sig + Other + Settings + Style + Doc
    if has_numbering { table_count += 1; }
    if has_hdrftr { table_count += 1; }
    if has_comments { table_count += 1; }
    if has_footnotes { table_count += 1; }
    if has_endnotes { table_count += 1; }

    let mut mt = w.begin_main_table(table_count);

    // Signature
    w.register_table(&mut mt, constants::table_type::SIGNATURE);
    tables::signature::write(&mut w);

    // Settings
    w.register_table(&mut mt, constants::table_type::SETTINGS);
    tables::settings::write(&mut w, model);

    // Other (empty theme)
    w.register_table(&mut mt, constants::table_type::OTHER);
    tables::other::write(&mut w);

    // Numbering
    if has_numbering {
        w.register_table(&mut mt, constants::table_type::NUMBERING);
        tables::numbering::write(&mut w, model);
    }

    // Comments
    if has_comments {
        w.register_table(&mut mt, constants::table_type::COMMENTS);
        tables::comments::write(&mut w, model);
    }

    // Footnotes
    if has_footnotes {
        w.register_table(&mut mt, constants::table_type::FOOTNOTES);
        tables::footnotes::write(&mut w, model);
    }

    // Endnotes
    if has_endnotes {
        w.register_table(&mut mt, constants::table_type::ENDNOTES);
        tables::endnotes::write(&mut w, model);
    }

    // Styles
    w.register_table(&mut mt, constants::table_type::STYLE);
    tables::styles::write(&mut w, model);

    // Headers/Footers
    if has_hdrftr {
        w.register_table(&mut mt, constants::table_type::HDR_FTR);
        tables::headers_footers::write(&mut w, model);
    }

    // Document (main content)
    w.register_table(&mut mt, constants::table_type::DOCUMENT);
    tables::document::write(&mut w, model);

    w.end_main_table(&mt);

    let binary = w.into_bytes();
    let b64 = base64::engine::general_purpose::STANDARD.encode(&binary);
    format!(
        "{};v{};{};{}",
        constants::DOCY_SIGNATURE,
        constants::DOCY_VERSION,
        binary.len(),
        b64
    )
}

pub use writer::DocyWriter;
