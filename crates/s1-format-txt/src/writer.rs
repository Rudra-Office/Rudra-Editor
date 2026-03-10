//! Plain text writer.
//!
//! Converts a [`DocumentModel`] into a plain text string. All formatting
//! is stripped. Paragraphs are separated by newlines. Tables are rendered
//! as tab-separated columns.

use s1_model::{DocumentModel, NodeId, NodeType};

/// Write a document model to plain text bytes (UTF-8).
pub fn write(doc: &DocumentModel) -> Vec<u8> {
    write_string(doc).into_bytes()
}

/// Write a document model to a plain text string.
pub fn write_string(doc: &DocumentModel) -> String {
    let body_id = match doc.body_id() {
        Some(id) => id,
        None => return String::new(),
    };

    let mut blocks = Vec::new();
    collect_blocks(doc, body_id, &mut blocks);
    blocks.join("\n")
}

/// Collect block-level text outputs from a container node.
/// Each paragraph becomes one entry, tables produce multiple entries (one per row).
fn collect_blocks(doc: &DocumentModel, container_id: NodeId, blocks: &mut Vec<String>) {
    let node = match doc.node(container_id) {
        Some(n) => n,
        None => return,
    };

    let children: Vec<NodeId> = node.children.clone();
    for child_id in children {
        let child = match doc.node(child_id) {
            Some(n) => n,
            None => continue,
        };

        match child.node_type {
            NodeType::Paragraph => {
                let mut text = String::new();
                let para_children: Vec<NodeId> = child.children.clone();
                for inline_id in para_children {
                    write_inline(doc, inline_id, &mut text);
                }
                blocks.push(text);
            }

            NodeType::Table => {
                write_table(doc, child_id, blocks);
            }

            // Sections and other containers: recurse
            NodeType::Section | NodeType::Body | NodeType::Document => {
                collect_blocks(doc, child_id, blocks);
            }

            // Skip headers, footers, comments in plain text
            NodeType::Header
            | NodeType::Footer
            | NodeType::CommentBody
            | NodeType::BookmarkStart
            | NodeType::BookmarkEnd
            | NodeType::CommentStart
            | NodeType::CommentEnd => {}

            _ => {
                // Other node types: try to extract inline content
                let mut text = String::new();
                write_inline(doc, child_id, &mut text);
                if !text.is_empty() {
                    blocks.push(text);
                }
            }
        }
    }
}

/// Write inline content (runs, text, breaks) into a string.
fn write_inline(doc: &DocumentModel, node_id: NodeId, out: &mut String) {
    let node = match doc.node(node_id) {
        Some(n) => n,
        None => return,
    };

    match node.node_type {
        NodeType::Text => {
            if let Some(text) = &node.text_content {
                out.push_str(text);
            }
        }
        NodeType::LineBreak => {
            out.push('\n');
        }
        NodeType::Tab => {
            out.push('\t');
        }
        NodeType::PageBreak | NodeType::ColumnBreak => {
            out.push('\n');
        }
        // Runs and other inline containers: recurse into children
        _ => {
            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                write_inline(doc, child_id, out);
            }
        }
    }
}

/// Write a table as tab-separated rows.
fn write_table(doc: &DocumentModel, table_id: NodeId, blocks: &mut Vec<String>) {
    let table = match doc.node(table_id) {
        Some(n) => n,
        None => return,
    };

    let rows: Vec<NodeId> = table.children.clone();
    for row_id in rows {
        let row = match doc.node(row_id) {
            Some(n) => n,
            None => continue,
        };

        let cells: Vec<NodeId> = row.children.clone();
        let mut cell_texts = Vec::new();

        for cell_id in cells {
            let mut cell_text = String::new();
            write_cell_content(doc, cell_id, &mut cell_text);
            cell_texts.push(cell_text.trim().to_string());
        }

        blocks.push(cell_texts.join("\t"));
    }
}

/// Write cell content, using spaces instead of newlines between paragraphs.
fn write_cell_content(doc: &DocumentModel, node_id: NodeId, out: &mut String) {
    let node = match doc.node(node_id) {
        Some(n) => n,
        None => return,
    };

    match node.node_type {
        NodeType::Text => {
            if let Some(text) = &node.text_content {
                out.push_str(text);
            }
        }
        NodeType::Paragraph => {
            if !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                write_cell_content(doc, child_id, out);
            }
        }
        NodeType::Tab | NodeType::LineBreak => {
            out.push(' ');
        }
        _ => {
            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                write_cell_content(doc, child_id, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1_model::Node;

    fn make_para_doc(lines: &[&str]) -> DocumentModel {
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

    #[test]
    fn write_empty_document() {
        let doc = DocumentModel::new();
        assert_eq!(write_string(&doc), "");
    }

    #[test]
    fn write_single_paragraph() {
        let doc = make_para_doc(&["Hello World"]);
        assert_eq!(write_string(&doc), "Hello World");
    }

    #[test]
    fn write_multiple_paragraphs() {
        let doc = make_para_doc(&["Line 1", "Line 2", "Line 3"]);
        assert_eq!(write_string(&doc), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn write_empty_paragraph() {
        let doc = make_para_doc(&["Line 1", "", "Line 3"]);
        assert_eq!(write_string(&doc), "Line 1\n\nLine 3");
    }

    #[test]
    fn write_strips_formatting() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let para_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(para_id, NodeType::Paragraph))
            .unwrap();

        let run_id = doc.next_id();
        let mut run = Node::new(run_id, NodeType::Run);
        run.attributes.set(
            s1_model::AttributeKey::Bold,
            s1_model::AttributeValue::Bool(true),
        );
        doc.insert_node(para_id, 0, run).unwrap();

        let text_id = doc.next_id();
        doc.insert_node(run_id, 0, Node::text(text_id, "Bold text"))
            .unwrap();

        assert_eq!(write_string(&doc), "Bold text");
    }

    #[test]
    fn write_table_tab_separated() {
        let mut doc = DocumentModel::new();
        let body_id = doc.body_id().unwrap();

        let table_id = doc.next_id();
        doc.insert_node(body_id, 0, Node::new(table_id, NodeType::Table))
            .unwrap();

        let row_id = doc.next_id();
        doc.insert_node(table_id, 0, Node::new(row_id, NodeType::TableRow))
            .unwrap();

        // Cell 1: "A"
        let cell1_id = doc.next_id();
        doc.insert_node(row_id, 0, Node::new(cell1_id, NodeType::TableCell))
            .unwrap();
        let p1 = doc.next_id();
        doc.insert_node(cell1_id, 0, Node::new(p1, NodeType::Paragraph))
            .unwrap();
        let r1 = doc.next_id();
        doc.insert_node(p1, 0, Node::new(r1, NodeType::Run))
            .unwrap();
        let t1 = doc.next_id();
        doc.insert_node(r1, 0, Node::text(t1, "A")).unwrap();

        // Cell 2: "B"
        let cell2_id = doc.next_id();
        doc.insert_node(row_id, 1, Node::new(cell2_id, NodeType::TableCell))
            .unwrap();
        let p2 = doc.next_id();
        doc.insert_node(cell2_id, 0, Node::new(p2, NodeType::Paragraph))
            .unwrap();
        let r2 = doc.next_id();
        doc.insert_node(p2, 0, Node::new(r2, NodeType::Run))
            .unwrap();
        let t2 = doc.next_id();
        doc.insert_node(r2, 0, Node::text(t2, "B")).unwrap();

        assert_eq!(write_string(&doc), "A\tB");
    }

    #[test]
    fn write_returns_utf8_bytes() {
        let doc = make_para_doc(&["Hello"]);
        let bytes = write(&doc);
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn write_unicode() {
        let doc = make_para_doc(&["こんにちは", "café"]);
        let text = write_string(&doc);
        assert_eq!(text, "こんにちは\ncafé");
    }
}
