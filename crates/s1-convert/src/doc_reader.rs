//! DOC (OLE2/CFB) reader with FIB, piece table, CHPx, table detection,
//! and metadata extraction.
//!
//! Extracts text and character formatting from legacy .doc files by parsing the
//! FileInformationBlock (FIB), piece table, and character property bin table
//! (PlcfBteChpx) from the OLE2 compound file container. Falls back to heuristic
//! byte scanning when proper parsing fails.
//!
//! This is a **partial** reader — the DOC binary format is extremely complex
//! (thousands of pages of spec). We extract:
//! - Plain text content via FIB → piece table → text extraction
//! - Basic paragraph breaks (0x0D separator)
//! - Character formatting (bold, italic, font size, color, underline,
//!   strikethrough, superscript/subscript) via PlcfBteChpx → CHPX FKP → SPRMs
//! - Table structure via cell mark (0x07) detection and restructuring
//! - Document metadata via SummaryInformation OLE2 stream
//!
//! Images, headers/footers are NOT extracted.
//! For full DOC support, consumers should use external conversion tools.

use std::io::{Cursor, Read};

use s1_model::attributes::{AttributeKey, AttributeValue, Color, UnderlineStyle};
use s1_model::{DocumentModel, Node, NodeId, NodeType};

use crate::chpx::{parse_chpx_bin_table, CharProperties, CharacterRun};
use crate::error::ConvertError;
use crate::fib::Fib;
use crate::piece_table::PieceTable;

/// Read a DOC file and extract what we can into a DocumentModel.
///
/// This extracts plain text organized into paragraphs with character
/// formatting. Table structures are detected from cell mark characters
/// (0x07) and rebuilt as proper Table/TableRow/TableCell nodes. Metadata
/// is extracted from the SummaryInformation OLE2 stream when available.
///
/// The reader first attempts proper FIB/piece table parsing. If that fails,
/// it falls back to heuristic text extraction for graceful degradation.
///
/// # Errors
///
/// Returns `ConvertError::InvalidDoc` if the file is not a valid OLE2 container
/// or does not contain a WordDocument stream.
pub fn read_doc(data: &[u8]) -> Result<DocumentModel, ConvertError> {
    let cursor = Cursor::new(data);
    let mut comp = cfb::CompoundFile::open(cursor)
        .map_err(|e| ConvertError::InvalidDoc(format!("not a valid OLE2 file: {e}")))?;

    // Check for WordDocument stream (required for .doc files)
    let has_word_doc = comp.walk().any(|entry| entry.name() == "WordDocument");

    if !has_word_doc {
        return Err(ConvertError::InvalidDoc(
            "missing WordDocument stream — not a Word document".into(),
        ));
    }

    // Try to extract metadata from SummaryInformation stream
    let metadata = extract_metadata(&mut comp);

    // Try to read text from the document
    let (text, char_runs) = extract_text_from_doc(&mut comp)?;

    // Build a DocumentModel from the extracted text with formatting
    let mut doc = if char_runs.is_empty() {
        build_model_from_text(&text)?
    } else {
        build_model_from_text_with_formatting(&text, &char_runs)?
    };

    // Apply metadata if we extracted any
    if let Some(meta) = metadata {
        *doc.metadata_mut() = meta;
    }

    // Post-process: detect table structures from cell marks
    detect_and_build_tables(&mut doc)?;

    Ok(doc)
}

/// Extract metadata from the SummaryInformation OLE2 stream.
///
/// Returns `None` if the stream is missing or cannot be parsed.
fn extract_metadata(
    comp: &mut cfb::CompoundFile<Cursor<&[u8]>>,
) -> Option<s1_model::metadata::DocumentMetadata> {
    let mut summary_data = Vec::new();
    let mut stream = comp.open_stream("/\x05SummaryInformation").ok()?;
    stream.read_to_end(&mut summary_data).ok()?;
    crate::summary_info::parse_summary_info(&summary_data).ok()
}

/// Build a DocumentModel from extracted plain text.
///
/// Splits text on newlines into paragraphs, each containing a single run.
fn build_model_from_text(text: &str) -> Result<DocumentModel, ConvertError> {
    let mut doc = DocumentModel::new();

    // DocumentModel::new() already creates a Body node under root
    let body_id = doc
        .body_id()
        .ok_or_else(|| ConvertError::InvalidDoc("document model missing body node".into()))?;

    // Split text into paragraphs
    let paragraphs: Vec<&str> = text.split('\n').collect();

    for (i, para_text) in paragraphs.iter().enumerate() {
        let trimmed = para_text.trim_matches('\r');
        if trimmed.is_empty() && i == paragraphs.len() - 1 {
            // Skip trailing empty paragraph
            continue;
        }

        let para_id = doc.next_id();
        doc.insert_node(body_id, i, Node::new(para_id, NodeType::Paragraph))
            .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

        if !trimmed.is_empty() {
            add_text_run(&mut doc, para_id, trimmed)?;
        }
    }

    Ok(doc)
}

/// Extract readable text and optional character runs from a DOC compound file.
///
/// Returns a tuple of `(text, character_runs)`. Character runs may be empty
/// if formatting could not be extracted.
///
/// Tries two strategies in order:
/// 1. Proper FIB + piece table parsing (structured extraction with formatting)
/// 2. Heuristic byte scanning (fallback for older/corrupted files, no formatting)
fn extract_text_from_doc(
    comp: &mut cfb::CompoundFile<Cursor<&[u8]>>,
) -> Result<(String, Vec<CharacterRun>), ConvertError> {
    // Read the WordDocument stream (required)
    let mut word_doc_data = Vec::new();
    if let Ok(mut stream) = comp.open_stream("/WordDocument") {
        stream
            .read_to_end(&mut word_doc_data)
            .map_err(|e| ConvertError::InvalidDoc(format!("failed to read WordDocument: {e}")))?;
    }

    // Strategy 1: Try proper FIB + piece table parsing
    if let Ok((text, char_runs)) = extract_text_and_formatting_via_piece_table(comp, &word_doc_data)
    {
        if !text.trim().is_empty() {
            return Ok((text, char_runs));
        }
    }

    // Strategy 2: Heuristic fallback (no formatting)
    let text = extract_text_heuristic(&word_doc_data);

    if text.trim().is_empty() {
        Ok((String::new(), Vec::new()))
    } else {
        Ok((text, Vec::new()))
    }
}

/// Extract text and character formatting using proper FIB and piece table parsing.
///
/// 1. Parse FIB from WordDocument stream
/// 2. Determine which table stream to use (0Table or 1Table)
/// 3. Read Clx from table stream at fcClx..fcClx+lcbClx
/// 4. Parse piece table from Clx
/// 5. Extract text from each piece (ANSI or Unicode)
/// 6. Optionally parse PlcfBteChpx for character formatting
/// 7. Use 0x0D as paragraph separator
///
/// Returns `(text, character_runs)`.
fn extract_text_and_formatting_via_piece_table(
    comp: &mut cfb::CompoundFile<Cursor<&[u8]>>,
    word_doc_data: &[u8],
) -> Result<(String, Vec<CharacterRun>), ConvertError> {
    // Parse FIB
    let fib = Fib::parse(word_doc_data)?;

    // Read the correct table stream
    let table_stream_path = format!("/{}", fib.table_stream_name());
    let mut table_data = Vec::new();
    let mut stream = comp.open_stream(&table_stream_path).map_err(|e| {
        ConvertError::InvalidDoc(format!("failed to open {}: {e}", fib.table_stream_name()))
    })?;
    stream.read_to_end(&mut table_data).map_err(|e| {
        ConvertError::InvalidDoc(format!("failed to read {}: {e}", fib.table_stream_name()))
    })?;

    // Extract Clx from table stream
    let fc_clx = fib.fc_clx as usize;
    let lcb_clx = fib.lcb_clx as usize;

    if lcb_clx == 0 {
        return Err(ConvertError::InvalidDoc("Clx length is zero".into()));
    }

    if fc_clx + lcb_clx > table_data.len() {
        return Err(ConvertError::InvalidDoc(format!(
            "Clx range {}..{} exceeds table stream size {}",
            fc_clx,
            fc_clx + lcb_clx,
            table_data.len()
        )));
    }

    let clx_data = &table_data[fc_clx..fc_clx + lcb_clx];

    // Parse piece table
    let piece_table = PieceTable::parse(clx_data)?;

    // Extract text from each piece, limiting to ccp_text characters
    let mut full_text = String::new();
    let mut chars_extracted: u32 = 0;
    let max_chars = if fib.ccp_text > 0 {
        fib.ccp_text
    } else {
        u32::MAX
    };

    for piece in &piece_table.pieces {
        if chars_extracted >= max_chars {
            break;
        }

        let piece_text = piece.text_from_stream(word_doc_data)?;

        // Limit extraction to ccp_text total characters
        let remaining = (max_chars - chars_extracted) as usize;
        let to_take = piece_text.chars().count().min(remaining);

        let taken: String = piece_text.chars().take(to_take).collect();
        chars_extracted += to_take as u32;

        full_text.push_str(&taken);
    }

    // Try to extract character formatting runs (best-effort)
    let char_runs = parse_chpx_bin_table(&table_data, word_doc_data, &fib).unwrap_or_default();

    // Convert DOC paragraph markers (0x0D / carriage return) to newlines
    let result = full_text.replace('\r', "\n");

    Ok((result, char_runs))
}

// ─── Table Detection ────────────────────────────────────────────────────────

/// Cell mark character used by the DOC format to terminate table cells.
const CELL_MARK: char = '\x07';

/// Detect table structure in extracted DOC text and restructure the document model.
///
/// In DOC format, table cells are terminated by 0x07 (cell mark) characters,
/// and table rows are terminated by a paragraph containing only 0x07.
/// This function scans paragraphs for these markers and restructures the
/// document model to use proper Table/TableRow/TableCell nodes.
///
/// The algorithm:
/// 1. Walk body children looking for consecutive paragraphs with 0x07
/// 2. Group them: paragraphs ending in 0x07 (with other text) = cell content,
///    paragraphs that are just 0x07 = row terminator
/// 3. Build Table > TableRow > TableCell > Paragraph structure
/// 4. Remove 0x07 characters from text content
///
/// # Errors
///
/// Returns `ConvertError::InvalidDoc` if the document model is malformed.
fn detect_and_build_tables(doc: &mut DocumentModel) -> Result<(), ConvertError> {
    let body_id = match doc.body_id() {
        Some(id) => id,
        None => return Ok(()),
    };

    // Collect body children info: (node_id, has_cell_mark, is_row_terminator)
    let body_children: Vec<NodeId> = doc
        .node(body_id)
        .map(|n| n.children.clone())
        .unwrap_or_default();

    if body_children.is_empty() {
        return Ok(());
    }

    // Analyze each paragraph to detect cell marks
    let mut para_info: Vec<(NodeId, bool)> = Vec::new();
    for &child_id in &body_children {
        let has_mark = paragraph_contains_cell_mark(doc, child_id);
        para_info.push((child_id, has_mark));
    }

    // Find contiguous groups of paragraphs with cell marks
    let groups = find_table_groups(&para_info);
    if groups.is_empty() {
        return Ok(());
    }

    // Process groups in reverse order (so indices remain valid)
    for group in groups.into_iter().rev() {
        let table_para_ids: Vec<NodeId> =
            para_info[group.clone()].iter().map(|(id, _)| *id).collect();

        build_table_from_paragraphs(doc, body_id, group.start, &table_para_ids)?;
    }

    Ok(())
}

/// Check if a paragraph (or its text descendants) contains the cell mark character.
fn paragraph_contains_cell_mark(doc: &DocumentModel, para_id: NodeId) -> bool {
    let node = match doc.node(para_id) {
        Some(n) => n,
        None => return false,
    };

    // Only process paragraphs
    if node.node_type != NodeType::Paragraph {
        return false;
    }

    // Check all descendant text nodes
    let descendants = doc.descendants(para_id);
    for desc in descendants {
        if desc.node_type == NodeType::Text {
            if let Some(ref text) = desc.text_content {
                if text.contains(CELL_MARK) {
                    return true;
                }
            }
        }
    }
    false
}

/// Collect the full text content of a paragraph from its text node descendants.
fn collect_paragraph_text(doc: &DocumentModel, para_id: NodeId) -> String {
    let mut text = String::new();
    let descendants = doc.descendants(para_id);
    for desc in descendants {
        if desc.node_type == NodeType::Text {
            if let Some(ref t) = desc.text_content {
                text.push_str(t);
            }
        }
    }
    text
}

/// Find contiguous groups of paragraphs that contain cell marks.
///
/// Returns a vector of range indices into the `para_info` slice,
/// each representing a group of consecutive table paragraphs.
fn find_table_groups(para_info: &[(NodeId, bool)]) -> Vec<std::ops::Range<usize>> {
    let mut groups = Vec::new();
    let mut i = 0;
    while i < para_info.len() {
        if para_info[i].1 {
            let start = i;
            while i < para_info.len() && para_info[i].1 {
                i += 1;
            }
            groups.push(start..i);
        } else {
            i += 1;
        }
    }
    groups
}

/// Build a table structure from a group of paragraphs with cell marks.
///
/// Removes the original paragraphs from the body and replaces them with
/// a single Table node containing TableRow > TableCell > Paragraph nodes.
///
/// Cell marks (0x07) are stripped from text content.
fn build_table_from_paragraphs(
    doc: &mut DocumentModel,
    body_id: NodeId,
    insert_index: usize,
    para_ids: &[NodeId],
) -> Result<(), ConvertError> {
    // Parse paragraphs into rows and cells:
    // - Paragraphs with text + 0x07 at the end → cell content
    // - Paragraphs that are solely 0x07 → row terminator
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();

    for &para_id in para_ids {
        let text = collect_paragraph_text(doc, para_id);
        let trimmed = text.trim_end_matches(CELL_MARK);

        if trimmed.is_empty() && text.contains(CELL_MARK) {
            // Row terminator — a paragraph that is just cell mark(s)
            if !current_row.is_empty() {
                rows.push(std::mem::take(&mut current_row));
            }
        } else {
            // Cell content — paragraph text before the cell mark
            current_row.push(trimmed.to_string());
        }
    }

    // Flush any remaining cells as a final row
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    if rows.is_empty() {
        return Ok(());
    }

    // Remove original paragraphs from body (in reverse to preserve indices)
    for &para_id in para_ids.iter().rev() {
        let _ = doc.remove_node(para_id);
    }

    // Create the table node
    let table_id = doc.next_id();
    doc.insert_node(body_id, insert_index, Node::new(table_id, NodeType::Table))
        .map_err(|e| ConvertError::InvalidDoc(format!("failed to create table: {e}")))?;

    for (row_idx, row_cells) in rows.iter().enumerate() {
        let row_id = doc.next_id();
        doc.insert_node(table_id, row_idx, Node::new(row_id, NodeType::TableRow))
            .map_err(|e| ConvertError::InvalidDoc(format!("failed to create table row: {e}")))?;

        for (cell_idx, cell_text) in row_cells.iter().enumerate() {
            let cell_id = doc.next_id();
            doc.insert_node(row_id, cell_idx, Node::new(cell_id, NodeType::TableCell))
                .map_err(|e| {
                    ConvertError::InvalidDoc(format!("failed to create table cell: {e}"))
                })?;

            // Create a paragraph inside the cell with the cleaned text
            let para_id = doc.next_id();
            doc.insert_node(cell_id, 0, Node::new(para_id, NodeType::Paragraph))
                .map_err(|e| {
                    ConvertError::InvalidDoc(format!("failed to create cell paragraph: {e}"))
                })?;

            if !cell_text.is_empty() {
                add_text_run(doc, para_id, cell_text)?;
            }
        }
    }

    Ok(())
}

// ─── Heuristic Extraction ───────────────────────────────────────────────────

/// Heuristic text extraction from a binary DOC stream.
///
/// Scans for contiguous runs of printable characters, treating 0x0D as
/// paragraph breaks. Filters out binary noise and control characters.
///
/// This is used as a fallback when FIB/piece table parsing fails.
pub(crate) fn extract_text_heuristic(data: &[u8]) -> String {
    let mut result = String::new();
    let mut current_run = String::new();
    let min_run_length = 4; // Minimum chars to consider a text run valid

    let mut i = 0;
    while i < data.len() {
        let byte = data[i];

        match byte {
            // Paragraph break
            0x0D => {
                if current_run.len() >= min_run_length {
                    result.push_str(&current_run);
                    result.push('\n');
                }
                current_run.clear();
            }
            // Tab
            0x09 => {
                current_run.push('\t');
            }
            // Printable ASCII
            0x20..=0x7E => {
                current_run.push(byte as char);
            }
            // Common Latin-1 printable chars (accented letters, etc.)
            0xC0..=0xFF => {
                // Try to interpret as Latin-1
                current_run.push(byte as char);
            }
            // Anything else breaks the current text run
            _ => {
                if current_run.len() >= min_run_length {
                    // Keep valid text runs
                } else {
                    current_run.clear();
                }
            }
        }

        i += 1;
    }

    // Flush last run
    if current_run.len() >= min_run_length {
        result.push_str(&current_run);
    }

    result
}

/// Add a text run to a paragraph.
fn add_text_run(doc: &mut DocumentModel, para_id: NodeId, text: &str) -> Result<(), ConvertError> {
    let run_id = doc.next_id();
    doc.insert_node(para_id, 0, Node::new(run_id, NodeType::Run))
        .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

    let text_id = doc.next_id();
    doc.insert_node(run_id, 0, Node::text(text_id, text))
        .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

    Ok(())
}

/// Add a formatted text run to a paragraph.
fn add_formatted_text_run(
    doc: &mut DocumentModel,
    para_id: NodeId,
    child_index: usize,
    text: &str,
    props: &CharProperties,
) -> Result<(), ConvertError> {
    let run_id = doc.next_id();
    doc.insert_node(para_id, child_index, Node::new(run_id, NodeType::Run))
        .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

    // Apply character properties to the run node
    apply_char_properties(doc, run_id, props);

    let text_id = doc.next_id();
    doc.insert_node(run_id, 0, Node::text(text_id, text))
        .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

    Ok(())
}

/// Apply character properties to a run node's attributes.
fn apply_char_properties(doc: &mut DocumentModel, run_id: NodeId, props: &CharProperties) {
    if let Some(node) = doc.node_mut(run_id) {
        if let Some(bold) = props.bold {
            node.attributes
                .set(AttributeKey::Bold, AttributeValue::Bool(bold));
        }
        if let Some(italic) = props.italic {
            node.attributes
                .set(AttributeKey::Italic, AttributeValue::Bool(italic));
        }
        if let Some(half_pts) = props.font_size_half_pts {
            let points = half_pts as f64 / 2.0;
            node.attributes
                .set(AttributeKey::FontSize, AttributeValue::Float(points));
        }
        if let Some(color_idx) = props.color_index {
            if let Some(color) = doc_color_index_to_color(color_idx) {
                node.attributes
                    .set(AttributeKey::Color, AttributeValue::Color(color));
            }
        }
        if let Some(underline) = props.underline {
            let style = match underline {
                0 => UnderlineStyle::None,
                1 => UnderlineStyle::Single,
                2 => UnderlineStyle::Single, // Words only → treat as Single
                3 => UnderlineStyle::Double,
                4 => UnderlineStyle::Dotted,
                6 => UnderlineStyle::Thick,
                7 => UnderlineStyle::Dashed,
                11 => UnderlineStyle::Wave,
                _ => UnderlineStyle::Single,
            };
            node.attributes.set(
                AttributeKey::Underline,
                AttributeValue::UnderlineStyle(style),
            );
        }
        if let Some(strike) = props.strikethrough {
            node.attributes
                .set(AttributeKey::Strikethrough, AttributeValue::Bool(strike));
        }
        if let Some(sup) = props.superscript {
            if sup {
                node.attributes
                    .set(AttributeKey::Superscript, AttributeValue::Bool(true));
            }
        }
        if let Some(sub) = props.subscript {
            if sub {
                node.attributes
                    .set(AttributeKey::Subscript, AttributeValue::Bool(true));
            }
        }
    }
}

/// Standard DOC color table mapping color index to RGB values.
///
/// Index 0 is "auto" which defaults to black in most contexts.
pub fn doc_color_index_to_color(index: u8) -> Option<Color> {
    match index {
        0 => Some(Color::BLACK),               // auto
        1 => Some(Color::BLACK),               // black
        2 => Some(Color::new(0, 0, 255)),      // blue
        3 => Some(Color::new(0, 255, 255)),    // cyan
        4 => Some(Color::new(0, 128, 0)),      // green
        5 => Some(Color::new(255, 0, 255)),    // magenta
        6 => Some(Color::new(255, 0, 0)),      // red
        7 => Some(Color::new(255, 255, 0)),    // yellow
        8 => Some(Color::WHITE),               // white
        9 => Some(Color::new(0, 0, 128)),      // dark blue
        10 => Some(Color::new(0, 128, 128)),   // dark cyan
        11 => Some(Color::new(0, 128, 0)),     // dark green
        12 => Some(Color::new(128, 0, 128)),   // dark magenta
        13 => Some(Color::new(128, 0, 0)),     // dark red
        14 => Some(Color::new(128, 128, 0)),   // dark yellow
        15 => Some(Color::new(128, 128, 128)), // dark gray
        16 => Some(Color::new(192, 192, 192)), // light gray
        _ => None,
    }
}

/// Build a DocumentModel from extracted text with character formatting applied.
///
/// Uses the character runs to split text into differently-formatted segments
/// within each paragraph. Character runs are sorted by `fc_start` and applied
/// to the text by character position mapping.
///
/// The character runs use file character positions (FCs) which are correlated
/// with character positions in the extracted text. This function attempts a
/// best-effort mapping by treating FC values as approximate character offsets
/// and applying formatting to matching text ranges.
fn build_model_from_text_with_formatting(
    text: &str,
    char_runs: &[CharacterRun],
) -> Result<DocumentModel, ConvertError> {
    let mut doc = DocumentModel::new();

    let body_id = doc
        .body_id()
        .ok_or_else(|| ConvertError::InvalidDoc("document model missing body node".into()))?;

    let paragraphs: Vec<&str> = text.split('\n').collect();

    // Track the global character position across paragraphs
    let mut global_cp: u32 = 0;

    for (i, para_text) in paragraphs.iter().enumerate() {
        let trimmed = para_text.trim_matches('\r');
        if trimmed.is_empty() && i == paragraphs.len() - 1 {
            continue;
        }

        let para_id = doc.next_id();
        doc.insert_node(body_id, i, Node::new(para_id, NodeType::Paragraph))
            .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

        if !trimmed.is_empty() {
            // Find character runs that overlap this paragraph's character range
            let para_start = global_cp;
            let para_end = global_cp + trimmed.len() as u32;

            // Collect runs that overlap with this paragraph
            let overlapping: Vec<&CharacterRun> = char_runs
                .iter()
                .filter(|r| r.fc_start < para_end && r.fc_end > para_start)
                .collect();

            if overlapping.is_empty() || !has_any_formatting(&overlapping) {
                // No formatting — use plain text run
                add_text_run(&mut doc, para_id, trimmed)?;
            } else {
                // Apply formatted runs
                let mut run_index = 0;
                let mut offset_in_para: usize = 0;

                for char_run in &overlapping {
                    // Calculate the overlap region within this paragraph
                    let run_start_in_para = if char_run.fc_start > para_start {
                        (char_run.fc_start - para_start) as usize
                    } else {
                        0
                    };
                    let run_end_in_para =
                        ((char_run.fc_end - para_start) as usize).min(trimmed.len());

                    // Add any unformatted text before this run
                    if run_start_in_para > offset_in_para {
                        let gap_text = &trimmed[offset_in_para..run_start_in_para];
                        if !gap_text.is_empty() {
                            add_text_run_at(&mut doc, para_id, run_index, gap_text)?;
                            run_index += 1;
                        }
                    }

                    // Add the formatted run
                    if run_start_in_para < run_end_in_para {
                        let seg = &trimmed[run_start_in_para..run_end_in_para];
                        if !seg.is_empty() {
                            add_formatted_text_run(
                                &mut doc,
                                para_id,
                                run_index,
                                seg,
                                &char_run.properties,
                            )?;
                            run_index += 1;
                        }
                    }

                    offset_in_para = run_end_in_para;
                }

                // Add any remaining text after the last run
                if offset_in_para < trimmed.len() {
                    let remaining_text = &trimmed[offset_in_para..];
                    if !remaining_text.is_empty() {
                        add_text_run_at(&mut doc, para_id, run_index, remaining_text)?;
                    }
                }
            }
        }

        // Advance global position past this paragraph + the newline separator
        global_cp += trimmed.len() as u32 + 1;
    }

    Ok(doc)
}

/// Check if any of the overlapping character runs have non-default formatting.
fn has_any_formatting(runs: &[&CharacterRun]) -> bool {
    runs.iter().any(|r| {
        let p = &r.properties;
        p.bold.is_some()
            || p.italic.is_some()
            || p.font_size_half_pts.is_some()
            || p.color_index.is_some()
            || p.font_index.is_some()
            || p.underline.is_some()
            || p.strikethrough.is_some()
            || p.superscript.is_some()
            || p.subscript.is_some()
    })
}

/// Add a text run at a specific child index.
fn add_text_run_at(
    doc: &mut DocumentModel,
    para_id: NodeId,
    child_index: usize,
    text: &str,
) -> Result<(), ConvertError> {
    let run_id = doc.next_id();
    doc.insert_node(para_id, child_index, Node::new(run_id, NodeType::Run))
        .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

    let text_id = doc.next_id();
    doc.insert_node(run_id, 0, Node::text(text_id, text))
        .map_err(|e| ConvertError::InvalidDoc(format!("{e}")))?;

    Ok(())
}

/// Check if the given bytes look like a DOC (OLE2/CFB) file.
///
/// Checks for the OLE2 magic bytes: `D0 CF 11 E0 A1 B1 1A E1`.
pub fn is_doc_file(data: &[u8]) -> bool {
    data.len() >= 8
        && data[0] == 0xD0
        && data[1] == 0xCF
        && data[2] == 0x11
        && data[3] == 0xE0
        && data[4] == 0xA1
        && data[5] == 0xB1
        && data[6] == 0x1A
        && data[7] == 0xE1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_doc_file_magic_bytes() {
        let magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        assert!(is_doc_file(&magic));
    }

    #[test]
    fn is_doc_file_too_short() {
        assert!(!is_doc_file(&[0xD0, 0xCF]));
    }

    #[test]
    fn is_doc_file_wrong_magic() {
        assert!(!is_doc_file(&[0x50, 0x4B, 0x03, 0x04, 0, 0, 0, 0])); // ZIP
    }

    #[test]
    fn read_doc_invalid_data() {
        let result = read_doc(b"not a doc file");
        assert!(result.is_err());
    }

    #[test]
    fn extract_text_heuristic_basic() {
        // Simulate a binary stream with embedded text
        let mut data = Vec::new();
        data.extend_from_slice(b"\x00\x00\x00"); // binary noise
        data.extend_from_slice(b"Hello World"); // text
        data.push(0x0D); // paragraph break
        data.extend_from_slice(b"Second paragraph here");
        data.push(0x0D);

        let text = extract_text_heuristic(&data);
        assert!(text.contains("Hello World"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn extract_text_heuristic_filters_short_runs() {
        let mut data = Vec::new();
        data.extend_from_slice(b"AB"); // too short (< 4 chars)
        data.push(0x00);
        data.extend_from_slice(b"A valid text run here"); // long enough
        data.push(0x0D);

        let text = extract_text_heuristic(&data);
        assert!(!text.contains("AB")); // short run filtered
        assert!(text.contains("A valid text run here"));
    }

    #[test]
    fn extract_text_heuristic_empty() {
        let text = extract_text_heuristic(&[0x00, 0x01, 0x02]);
        assert!(text.is_empty());
    }

    #[test]
    fn extract_text_heuristic_tabs() {
        let mut data = Vec::new();
        data.extend_from_slice(b"Col1\tCol2\tCol3");
        data.push(0x0D);

        let text = extract_text_heuristic(&data);
        assert!(text.contains("Col1\tCol2\tCol3"));
    }

    #[test]
    fn read_doc_still_works_for_invalid_fib() {
        // Create a valid OLE2 file with a WordDocument stream that has
        // garbage FIB data — should fall back to heuristic extraction.
        // We can't easily create a real OLE2 file in a unit test without
        // cfb's write support, so we test that the function handles
        // non-OLE2 data gracefully by returning an error.
        let result = read_doc(b"not an OLE2 file");
        assert!(result.is_err());
        // The error should be about invalid OLE2, not about FIB
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("OLE2") || err.contains("not a valid"),
            "got: {err}"
        );
    }

    #[test]
    fn build_model_from_text_basic() {
        let text = "Hello World\nSecond paragraph";
        let doc = build_model_from_text(text).unwrap();
        let root = doc.root_id();
        let root_node = doc.node(root).unwrap();
        assert!(!root_node.children.is_empty());

        // Should have a body node
        let body_id = root_node.children[0];
        let body = doc.node(body_id).unwrap();
        // Should have 2 paragraphs
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn build_model_from_text_empty() {
        let doc = build_model_from_text("").unwrap();
        let root = doc.root_id();
        let root_node = doc.node(root).unwrap();
        // Body should exist
        assert_eq!(root_node.children.len(), 1);
        let body_id = root_node.children[0];
        let body = doc.node(body_id).unwrap();
        // Empty string splits into [""], which is a single empty paragraph
        // but it's the trailing empty one, so it gets skipped
        // However, split("") returns [""] (len 1), and the trailing check
        // only skips when i == paragraphs.len() - 1 AND trimmed is empty.
        // Here i=0 and paragraphs.len()-1=0, so it IS the trailing one. Skipped.
        assert!(body.children.is_empty());
    }

    #[test]
    fn build_model_from_text_with_carriage_returns() {
        let text = "Line one\r\nLine two\r\n";
        let doc = build_model_from_text(text).unwrap();
        let root = doc.root_id();
        let body_id = doc.node(root).unwrap().children[0];
        let body = doc.node(body_id).unwrap();
        // "Line one" + "Line two" + trailing empty (skipped) = 2 paragraphs
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn doc_color_index_to_color_table() {
        use s1_model::attributes::Color;

        // Index 0 = auto (black)
        assert_eq!(doc_color_index_to_color(0), Some(Color::BLACK));
        // Index 1 = black
        assert_eq!(doc_color_index_to_color(1), Some(Color::BLACK));
        // Index 2 = blue
        assert_eq!(doc_color_index_to_color(2), Some(Color::new(0, 0, 255)));
        // Index 6 = red
        assert_eq!(doc_color_index_to_color(6), Some(Color::new(255, 0, 0)));
        // Index 8 = white
        assert_eq!(doc_color_index_to_color(8), Some(Color::WHITE));
        // Index 16 = light gray
        assert_eq!(
            doc_color_index_to_color(16),
            Some(Color::new(192, 192, 192))
        );
        // Out of range
        assert_eq!(doc_color_index_to_color(17), None);
        assert_eq!(doc_color_index_to_color(255), None);
    }

    #[test]
    fn build_model_applies_formatting_infrastructure() {
        // Test that the formatting infrastructure works by building a model
        // with character properties applied to a run node directly.
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let props = CharProperties {
            bold: Some(true),
            italic: Some(true),
            font_size_half_pts: Some(28), // 14pt
            color_index: Some(6),         // red
            underline: Some(1),           // single underline
            strikethrough: Some(false),
            superscript: None,
            subscript: None,
            all_caps: None,
            font_index: None,
        };

        add_formatted_text_run(&mut doc, para_id, 0, "Hello", &props).unwrap();

        // Find the run node
        let para = doc.node(para_id).unwrap();
        assert_eq!(para.children.len(), 1);
        let run_id = para.children[0];
        let run = doc.node(run_id).unwrap();

        // Check attributes
        assert_eq!(run.attributes.get_bool(&AttributeKey::Bold), Some(true));
        assert_eq!(run.attributes.get_bool(&AttributeKey::Italic), Some(true));
        assert_eq!(run.attributes.get_f64(&AttributeKey::FontSize), Some(14.0));
        assert_eq!(
            run.attributes.get_color(&AttributeKey::Color),
            Some(Color::new(255, 0, 0))
        );

        // Check text content
        let text_id = run.children[0];
        let text_node = doc.node(text_id).unwrap();
        assert_eq!(text_node.text_content.as_deref(), Some("Hello"));
    }

    // ─── Table Detection Tests ──────────────────────────────────────────

    /// Helper: create a document with paragraphs from text lines.
    /// Cell marks (0x07) in text are preserved as-is.
    fn build_doc_with_paragraphs(lines: &[&str]) -> DocumentModel {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        for (i, line) in lines.iter().enumerate() {
            let para_id = doc.next_id();
            doc.insert_node(body_id, i, Node::new(para_id, NodeType::Paragraph))
                .unwrap();

            if !line.is_empty() {
                let run_id = doc.next_id();
                doc.insert_node(para_id, 0, Node::new(run_id, NodeType::Run))
                    .unwrap();

                let text_id = doc.next_id();
                doc.insert_node(run_id, 0, Node::text(text_id, *line))
                    .unwrap();
            }
        }

        doc
    }

    /// Helper: collect text from a paragraph's text-node descendants.
    fn get_para_text(doc: &DocumentModel, para_id: NodeId) -> String {
        collect_paragraph_text(doc, para_id)
    }

    #[test]
    fn detect_tables_simple() {
        // A simple 1-row, 2-cell table:
        // "Cell1\x07", "Cell2\x07", "\x07" (row terminator)
        let mut doc = build_doc_with_paragraphs(&["Cell1\x07", "Cell2\x07", "\x07"]);

        detect_and_build_tables(&mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1, "should have 1 table");

        let table_id = body.children[0];
        let table = doc.node(table_id).unwrap();
        assert_eq!(table.node_type, NodeType::Table);
        assert_eq!(table.children.len(), 1, "should have 1 row");

        let row_id = table.children[0];
        let row = doc.node(row_id).unwrap();
        assert_eq!(row.node_type, NodeType::TableRow);
        assert_eq!(row.children.len(), 2, "should have 2 cells");

        // Check cell content
        let cell0 = doc.node(row.children[0]).unwrap();
        assert_eq!(cell0.node_type, NodeType::TableCell);
        let cell0_para = cell0.children[0];
        assert_eq!(get_para_text(&doc, cell0_para), "Cell1");

        let cell1 = doc.node(row.children[1]).unwrap();
        let cell1_para = cell1.children[0];
        assert_eq!(get_para_text(&doc, cell1_para), "Cell2");
    }

    #[test]
    fn detect_tables_multi_row() {
        // 2 rows, 2 cells each
        let mut doc = build_doc_with_paragraphs(&[
            "A1\x07", "B1\x07", "\x07", // end of row 1
            "A2\x07", "B2\x07", "\x07", // end of row 2
        ]);

        detect_and_build_tables(&mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 1);

        let table_id = body.children[0];
        let table = doc.node(table_id).unwrap();
        assert_eq!(table.children.len(), 2, "should have 2 rows");

        // Row 1
        let row0 = doc.node(table.children[0]).unwrap();
        assert_eq!(row0.children.len(), 2);
        let r0c0_para = doc.node(row0.children[0]).unwrap().children[0];
        assert_eq!(get_para_text(&doc, r0c0_para), "A1");
        let r0c1_para = doc.node(row0.children[1]).unwrap().children[0];
        assert_eq!(get_para_text(&doc, r0c1_para), "B1");

        // Row 2
        let row1 = doc.node(table.children[1]).unwrap();
        assert_eq!(row1.children.len(), 2);
        let r1c0_para = doc.node(row1.children[0]).unwrap().children[0];
        assert_eq!(get_para_text(&doc, r1c0_para), "A2");
        let r1c1_para = doc.node(row1.children[1]).unwrap().children[0];
        assert_eq!(get_para_text(&doc, r1c1_para), "B2");
    }

    #[test]
    fn detect_tables_mixed() {
        // Regular paragraph, then a table, then another regular paragraph
        let mut doc =
            build_doc_with_paragraphs(&["Normal paragraph", "Cell1\x07", "\x07", "After table"]);

        detect_and_build_tables(&mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), 3, "normal + table + normal");

        // First child: regular paragraph
        let first = doc.node(body.children[0]).unwrap();
        assert_eq!(first.node_type, NodeType::Paragraph);

        // Second child: table
        let table = doc.node(body.children[1]).unwrap();
        assert_eq!(table.node_type, NodeType::Table);

        // Third child: regular paragraph
        let last = doc.node(body.children[2]).unwrap();
        assert_eq!(last.node_type, NodeType::Paragraph);
    }

    #[test]
    fn no_tables_in_regular_text() {
        // No cell marks — document should be unchanged
        let mut doc =
            build_doc_with_paragraphs(&["First paragraph", "Second paragraph", "Third paragraph"]);

        let body_id = doc.body_id().unwrap();
        let orig_count = doc.node(body_id).unwrap().children.len();

        detect_and_build_tables(&mut doc).unwrap();

        let body = doc.node(body_id).unwrap();
        assert_eq!(body.children.len(), orig_count, "no changes expected");

        // All children should still be paragraphs
        for &child_id in &body.children {
            let child = doc.node(child_id).unwrap();
            assert_eq!(child.node_type, NodeType::Paragraph);
        }
    }

    #[test]
    fn detect_tables_preserves_content() {
        // Verify that 0x07 characters are stripped from cell text
        let mut doc = build_doc_with_paragraphs(&["Hello World\x07", "Rust Engine\x07", "\x07"]);

        detect_and_build_tables(&mut doc).unwrap();

        let body_id = doc.body_id().unwrap();
        let table_id = doc.node(body_id).unwrap().children[0];
        let row_id = doc.node(table_id).unwrap().children[0];

        // Cell 0
        let cell0_id = doc.node(row_id).unwrap().children[0];
        let cell0_para = doc.node(cell0_id).unwrap().children[0];
        let text = get_para_text(&doc, cell0_para);
        assert_eq!(text, "Hello World");
        assert!(!text.contains('\x07'), "cell mark should be stripped");

        // Cell 1
        let cell1_id = doc.node(row_id).unwrap().children[1];
        let cell1_para = doc.node(cell1_id).unwrap().children[0];
        let text = get_para_text(&doc, cell1_para);
        assert_eq!(text, "Rust Engine");
        assert!(!text.contains('\x07'), "cell mark should be stripped");
    }
}
