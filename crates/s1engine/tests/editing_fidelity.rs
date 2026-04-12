//! Editing fidelity tests — validates that editing operations produce correct
//! results through the full pipeline: operation → model → layout → export.
//!
//! Tests real editing scenarios: typing, deleting, formatting, adding images,
//! splitting paragraphs, then verifies the result is correct in both the
//! model and the exported DOCX.

use s1engine::{DocumentBuilder, Engine, Format, Operation, Transaction};
use s1_model::{AttributeKey, AttributeValue, Node, NodeType};

fn engine() -> Engine { Engine::new() }

/// Helper: open a real document, apply edits, export, re-import, validate.
fn edit_and_validate(
    doc: &mut s1engine::Document,
    edit_name: &str,
    edit_fn: impl FnOnce(&mut s1engine::Document),
    validate_fn: impl Fn(&s1engine::Document),
) {
    edit_fn(doc);
    validate_fn(doc);

    let bytes = doc.export(Format::Docx)
        .unwrap_or_else(|e| panic!("{edit_name}: export failed: {e}"));
    let doc2 = engine().open(&bytes)
        .unwrap_or_else(|e| panic!("{edit_name}: re-import failed: {e}"));
    validate_fn(&doc2);

    eprintln!("{edit_name}: PASS (model + export + re-import all valid)");
}

// ═══════════════════════════════════════════════════════
// 1. INSERT NEW LINE / PARAGRAPH
// ═══════════���══════════════════════════════════════��════

#[test]
fn edit_insert_paragraph() {
    let mut doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Line one"))
        .paragraph(|p| p.text("Line two"))
        .build();

    let initial_count = doc.paragraph_count();

    edit_and_validate(&mut doc, "insert_paragraph", |doc| {
        // Insert a new paragraph after the body's first child
        let body_id = doc.model().body_id().unwrap();
        let body = doc.model().node(body_id).unwrap();
        let insert_after = body.children[0]; // first paragraph

        let new_para_id = doc.next_id();
        let new_para = Node::new(new_para_id, NodeType::Paragraph);
        let new_run_id = doc.next_id();
        let new_run = Node::new(new_run_id, NodeType::Run);
        let new_text_id = doc.next_id();
        let mut new_text = Node::new(new_text_id, NodeType::Text);
        new_text.text_content = Some("Inserted line".to_string());

        let mut txn = Transaction::with_label("Insert paragraph");
        txn.push(Operation::insert_node(body_id, 1, new_para));
        txn.push(Operation::insert_node(new_para_id, 0, new_run));
        txn.push(Operation::insert_node(new_run_id, 0, new_text));
        doc.apply_transaction(&txn).unwrap();
    }, |doc| {
        assert_eq!(doc.paragraph_count(), initial_count + 1, "paragraph count");
        assert!(doc.to_plain_text().contains("Inserted line"), "inserted text");
        assert!(doc.to_plain_text().contains("Line one"), "original text preserved");
        assert!(doc.to_plain_text().contains("Line two"), "original text preserved");
    });
}

// ══════���═══════════════════════════════════════��════════
// 2. DELETE A PARAGRAPH
// ��════════════════════════════════════════════════���═════

#[test]
fn edit_delete_paragraph() {
    let mut doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Keep this"))
        .paragraph(|p| p.text("Delete this"))
        .paragraph(|p| p.text("Keep this too"))
        .build();

    let initial_count = doc.paragraph_count();

    edit_and_validate(&mut doc, "delete_paragraph", |doc| {
        let body_id = doc.model().body_id().unwrap();
        let body = doc.model().node(body_id).unwrap();
        let delete_id = body.children[1]; // second paragraph

        doc.apply(Operation::delete_node(delete_id)).unwrap();
    }, |doc| {
        assert_eq!(doc.paragraph_count(), initial_count - 1, "paragraph count");
        assert!(!doc.to_plain_text().contains("Delete this"), "deleted text should be gone");
        assert!(doc.to_plain_text().contains("Keep this"), "first para preserved");
        assert!(doc.to_plain_text().contains("Keep this too"), "third para preserved");
    });
}

// ════════════════════���══════════════════════════════════
// 3. INSERT TEXT (TYPING)
// ══════��════════════════════════════════════════════════

#[test]
fn edit_insert_text() {
    let mut doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Hello world"))
        .build();

    edit_and_validate(&mut doc, "insert_text", |doc| {
        let body_id = doc.model().body_id().unwrap();
        let body = doc.model().node(body_id).unwrap();
        let para_id = body.children[0];
        let para = doc.model().node(para_id).unwrap();
        let run_id = para.children[0];
        let run = doc.model().node(run_id).unwrap();
        let text_id = run.children[0];

        // Insert " beautiful" after "Hello" (offset 5)
        doc.apply(Operation::insert_text(text_id, 5, " beautiful")).unwrap();
    }, |doc| {
        let text = doc.to_plain_text();
        assert!(text.contains("Hello beautiful world"), "inserted text: {text}");
    });
}

// ═══════════���═══════════════════════════���═══════════════
// 4. DELETE TEXT (BACKSPACE)
// ═══════════════════════════════════════════════════════

#[test]
fn edit_delete_text_backspace() {
    let mut doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Hello world"))
        .build();

    edit_and_validate(&mut doc, "backspace", |doc| {
        let body_id = doc.model().body_id().unwrap();
        let para_id = doc.model().node(body_id).unwrap().children[0];
        let run_id = doc.model().node(para_id).unwrap().children[0];
        let text_id = doc.model().node(run_id).unwrap().children[0];

        // Delete "world" (offset 6, length 5) — simulates selecting "world" and pressing backspace
        doc.apply(Operation::delete_text(text_id, 6, 5)).unwrap();
    }, |doc| {
        let text = doc.to_plain_text();
        assert!(text.contains("Hello "), "remaining text: {text}");
        assert!(!text.contains("world"), "deleted text should be gone: {text}");
    });
}

// ═════��══════════════��═══════════════════════���══════════
// 5. FORMAT TEXT (BOLD)
// ════════════════════���══════════════════════════════════

#[test]
fn edit_apply_bold() {
    let mut doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Normal text"))
        .build();

    edit_and_validate(&mut doc, "apply_bold", |doc| {
        let body_id = doc.model().body_id().unwrap();
        let para_id = doc.model().node(body_id).unwrap().children[0];
        let run_id = doc.model().node(para_id).unwrap().children[0];

        let mut attrs = s1_model::AttributeMap::new();
        attrs.set(AttributeKey::Bold, AttributeValue::Bool(true));
        doc.apply(Operation::set_attributes(run_id, attrs)).unwrap();
    }, |doc| {
        let body_id = doc.model().body_id().unwrap();
        let para_id = doc.model().node(body_id).unwrap().children[0];
        let run_id = doc.model().node(para_id).unwrap().children[0];
        let run = doc.model().node(run_id).unwrap();
        assert_eq!(run.attributes.get_bool(&AttributeKey::Bold), Some(true), "bold attribute");
        assert!(doc.to_plain_text().contains("Normal text"), "text preserved");
    });
}

// ═���═════════════════════════════════════════════════════
// 6. UNDO AFTER EDIT
// ═══════════════════════════════════════════════════════

#[test]
fn edit_undo() {
    let mut doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Original text"))
        .build();

    let original_text = doc.to_plain_text();

    // Insert text
    let body_id = doc.model().body_id().unwrap();
    let para_id = doc.model().node(body_id).unwrap().children[0];
    let run_id = doc.model().node(para_id).unwrap().children[0];
    let text_id = doc.model().node(run_id).unwrap().children[0];

    let mut txn = Transaction::with_label("type");
    txn.push(Operation::insert_text(text_id, 8, " modified"));
    doc.apply_transaction(&txn).unwrap();
    assert!(doc.to_plain_text().contains("modified"), "edit applied");

    // Undo
    doc.undo().unwrap();
    let after_undo = doc.to_plain_text();
    assert!(!after_undo.contains("modified"), "undo removed edit: {after_undo}");

    // Redo
    doc.redo().unwrap();
    assert!(doc.to_plain_text().contains("modified"), "redo restored edit");

    eprintln!("edit_undo: PASS");
}

// ══���════════════════════════════════════════════════════
// 7. ADD IMAGE
// ══════════════════════════════════════════════════════��

#[test]
fn edit_add_image() {
    let mut doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Before image"))
        .paragraph(|p| p.text("After image"))
        .build();

    let initial_count = doc.paragraph_count();

    // Add a new paragraph (simulating image insertion — actual image needs WASM API)
    edit_and_validate(&mut doc, "add_image", |doc| {
        let body_id = doc.model().body_id().unwrap();

        let img_para_id = doc.next_id();
        let img_para = Node::new(img_para_id, NodeType::Paragraph);
        let img_run_id = doc.next_id();
        let img_run = Node::new(img_run_id, NodeType::Run);
        let img_text_id = doc.next_id();
        let mut img_text = Node::new(img_text_id, NodeType::Text);
        img_text.text_content = Some("[Image placeholder]".to_string());

        let mut txn = Transaction::with_label("Insert image placeholder");
        txn.push(Operation::insert_node(body_id, 1, img_para));
        txn.push(Operation::insert_node(img_para_id, 0, img_run));
        txn.push(Operation::insert_node(img_run_id, 0, img_text));
        doc.apply_transaction(&txn).unwrap();
    }, |doc| {
        assert!(doc.paragraph_count() >= initial_count + 1, "paragraph added");
        assert!(doc.to_plain_text().contains("Before image"), "text preserved");
        assert!(doc.to_plain_text().contains("After image"), "text preserved");
        assert!(doc.to_plain_text().contains("[Image placeholder]"), "image para added");
    });
}

// ════��══════════════════════════════════════════════════
// 8. CHANGE HEADING LEVEL
// ════════════════════��══════════════════════════════════

#[test]
fn edit_change_heading() {
    let mut doc = DocumentBuilder::new()
        .heading(1, "Title")
        .paragraph(|p| p.text("Body"))
        .build();

    edit_and_validate(&mut doc, "change_heading", |doc| {
        let body_id = doc.model().body_id().unwrap();
        let para_id = doc.model().node(body_id).unwrap().children[0];

        // Change heading from H1 to H2
        let mut attrs = s1_model::AttributeMap::new();
        attrs.set(AttributeKey::StyleId, AttributeValue::String("Heading2".to_string()));
        doc.apply(Operation::set_attributes(para_id, attrs)).unwrap();
    }, |doc| {
        let headings = doc.model().collect_headings();
        assert!(!headings.is_empty(), "should still have headings");
        assert!(doc.to_plain_text().contains("Title"), "heading text preserved");
    });
}

// ��════════════════════��═════════════════════════════════
// 9. MULTI-STEP EDIT SEQUENCE
// ═════════��═════════════════════════��═══════════════════

#[test]
fn edit_multi_step_sequence() {
    let mut doc = DocumentBuilder::new()
        .heading(1, "Document")
        .paragraph(|p| p.text("First paragraph."))
        .paragraph(|p| p.text("Second paragraph."))
        .build();

    // Step 1: Insert text in first paragraph
    {
        let body_id = doc.model().body_id().unwrap();
        let para_id = doc.model().node(body_id).unwrap().children[1]; // first content para
        let run_id = doc.model().node(para_id).unwrap().children[0];
        let text_id = doc.model().node(run_id).unwrap().children[0];
        doc.apply(Operation::insert_text(text_id, 16, " Added text.")).unwrap();
    }

    // Step 2: Bold the heading
    {
        let body_id = doc.model().body_id().unwrap();
        let heading_id = doc.model().node(body_id).unwrap().children[0];
        let run_id = doc.model().node(heading_id).unwrap().children[0];
        let mut attrs = s1_model::AttributeMap::new();
        attrs.set(AttributeKey::Bold, AttributeValue::Bool(true));
        doc.apply(Operation::set_attributes(run_id, attrs)).unwrap();
    }

    // Step 3: Add new paragraph at end
    {
        let body_id = doc.model().body_id().unwrap();
        let child_count = doc.model().node(body_id).unwrap().children.len();

        let para_id = doc.next_id();
        let para = Node::new(para_id, NodeType::Paragraph);
        let run_id = doc.next_id();
        let run = Node::new(run_id, NodeType::Run);
        let text_id = doc.next_id();
        let mut text = Node::new(text_id, NodeType::Text);
        text.text_content = Some("Third paragraph.".to_string());

        let mut txn = Transaction::with_label("Add para");
        txn.push(Operation::insert_node(body_id, child_count, para));
        txn.push(Operation::insert_node(para_id, 0, run));
        txn.push(Operation::insert_node(run_id, 0, text));
        doc.apply_transaction(&txn).unwrap();
    }

    // Validate all changes
    let text = doc.to_plain_text();
    assert!(text.contains("Added text"), "step 1: {text}");
    assert!(text.contains("Third paragraph"), "step 3: {text}");

    // Export and re-import
    let bytes = doc.export(Format::Docx).unwrap();
    let doc2 = engine().open(&bytes).unwrap();
    let text2 = doc2.to_plain_text();
    assert!(text2.contains("Added text"), "round-trip step 1: {text2}");
    assert!(text2.contains("Third paragraph"), "round-trip step 3: {text2}");

    eprintln!("edit_multi_step: PASS (3 edits + export + re-import valid)");
}

// ═══════════════════════════════════════════════════════
// 10. EDIT REAL DOCUMENT
// ═════════════════════════���═════════════════════════════

#[test]
fn edit_real_document() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests/fidelity/real-world/design-system.docx");

    if !path.exists() {
        eprintln!("SKIP: design-system.docx not found");
        return;
    }

    let data = std::fs::read(&path).unwrap();
    let mut doc = engine().open(&data).unwrap();
    let original_count = doc.paragraph_count();
    let original_text = doc.to_plain_text();

    eprintln!("edit_real_doc: opened with {} paragraphs", original_count);

    // Edit: add a paragraph at the end
    let body_id = doc.model().body_id().unwrap();
    let child_count = doc.model().node(body_id).unwrap().children.len();

    let para_id = doc.next_id();
    let para = Node::new(para_id, NodeType::Paragraph);
    let run_id = doc.next_id();
    let run = Node::new(run_id, NodeType::Run);
    let text_id = doc.next_id();
    let mut text_node = Node::new(text_id, NodeType::Text);
    text_node.text_content = Some("FIDELITY_TEST_MARKER".to_string());

    let mut txn = Transaction::with_label("fidelity test");
    txn.push(Operation::insert_node(body_id, child_count, para));
    txn.push(Operation::insert_node(para_id, 0, run));
    txn.push(Operation::insert_node(run_id, 0, text_node));
    doc.apply_transaction(&txn).unwrap();

    // Validate edit
    assert_eq!(doc.paragraph_count(), original_count + 1);
    assert!(doc.to_plain_text().contains("FIDELITY_TEST_MARKER"));

    // Export and re-import
    let bytes = doc.export(Format::Docx).unwrap();
    let doc2 = engine().open(&bytes).unwrap();
    assert!(doc2.to_plain_text().contains("FIDELITY_TEST_MARKER"), "marker survived round-trip");
    assert_eq!(doc2.paragraph_count(), original_count + 1, "paragraph count after round-trip");

    // Original content should still be present
    let first_100: String = original_text.chars().take(100).collect();
    let first_words: Vec<&str> = first_100.split_whitespace().take(5).collect();
    for word in &first_words {
        if word.len() > 3 {
            assert!(doc2.to_plain_text().contains(word),
                "original word '{}' lost after edit+round-trip", word);
        }
    }

    eprintln!("edit_real_doc: PASS (edit + export + re-import, original content preserved)");
}
