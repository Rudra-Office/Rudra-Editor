# s1engine — AI Development Context

## What Is This Project?

s1engine is a modular document engine SDK built in Rust (with C/C++ FFI where necessary). It reads, writes, edits, and converts document formats (DOCX, ODT, PDF, TXT). The long-term goal is a CRDT-based collaborative editing engine.

This is a **library**, not an application. Consumers build editors/tools on top of it.

## Read These First

1. `docs/OVERVIEW.md` — Project vision, goals, non-goals
2. `docs/ARCHITECTURE.md` — System design, crate structure, core design decisions
3. `docs/SPECIFICATION.md` — Detailed technical spec for every module
4. `docs/ROADMAP.md` — Phased development plan with milestones
5. `docs/API_DESIGN.md` — Public API surface, feature flags, examples
6. `docs/DEPENDENCIES.md` — All external dependencies with rationale

## Architecture Rules (MUST Follow)

### 1. Document Model is Sacred
- `s1-model` has **ZERO external dependencies** — pure Rust data structures only
- Every node MUST have a globally unique `NodeId(replica_id, counter)`
- Never expose internal model representation in public API

### 2. All Mutations Via Operations
- NEVER modify the document tree directly
- ALL changes go through `Operation` → applied via `s1-ops`
- This is non-negotiable — it's the foundation for undo/redo and future CRDT support
- Every `Operation` must implement `invert()` for undo

### 3. Format Isolation
- Each format crate (`s1-format-docx`, `s1-format-odt`, etc.) ONLY depends on `s1-model`
- Format crates NEVER depend on each other
- Format crates NEVER depend on `s1-ops` or `s1-layout`

### 4. No Panics in Library Code
- ALL public functions return `Result<T, Error>`
- No `.unwrap()` or `.expect()` in library code (tests are fine)
- Be lenient in parsing (warn on unknown elements), strict in writing (valid output)

### 5. Error Types
- Use `thiserror` for error derivation
- Each crate has its own error type, convertible to top-level `s1engine::Error`
- Errors must be informative — include context (file position, node id, format element)

## Crate Structure

```
crates/
  s1-model/          Core document model (tree, nodes, attributes, styles)
  s1-ops/            Operations, transactions, undo/redo, cursor/selection
  s1-format-docx/    DOCX (OOXML) reader/writer
  s1-format-odt/     ODT (ODF) reader/writer
  s1-format-pdf/     PDF export only
  s1-format-txt/     Plain text reader/writer
  s1-convert/        Format conversion pipelines (incl. DOC→DOCX)
  s1-layout/         Page layout engine (pagination, line breaking)
  s1-text/           Text processing (HarfBuzz, FreeType, ICU via FFI)
  s1engine/          Facade crate — high-level public API
ffi/
  c/                 C FFI bindings (cbindgen)
  wasm/              WASM bindings (wasm-bindgen)
```

## Coding Conventions

### Rust Style
- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Use `snake_case` for functions/modules, `PascalCase` for types, `SCREAMING_SNAKE` for constants
- Prefer `&str` over `String` in function parameters
- Use `impl Into<String>` for builder methods that take ownership
- Derive `Debug, Clone, PartialEq` on all public types where reasonable
- Use `#[non_exhaustive]` on public enums that may gain variants

### Testing
- Every public function needs at least one test
- Format crates need round-trip tests (read → write → read → compare)
- Use `proptest` for property-based testing on `s1-model` and `s1-ops`
- Use `cargo-fuzz` for format readers
- Test fixtures go in `tests/fixtures/`

### Performance
- Profile before optimizing — use `criterion` benchmarks
- Layout engine MUST be incremental (don't re-layout unchanged content)
- Avoid unnecessary allocations in hot paths
- Use `&[u8]` / `impl Read` for I/O, not file paths in core logic

### Documentation
- All public items need `///` doc comments
- Include examples in doc comments for key APIs
- Use `# Errors` section to document when functions return errors
- Use `# Panics` section if a function can panic (shouldn't happen in lib code)

## Key Design Patterns

### Builder Pattern (for document construction)
```rust
doc.builder()
    .heading(1, "Title")
    .paragraph(|p| p.text("Hello ").bold("world"))
    .build();
```

### Transaction Pattern (for editing)
```rust
let mut txn = doc.begin_transaction("description");
txn.insert_text(node_id, offset, "text")?;
txn.commit()?; // Atomic undo unit
```

### Codec Pattern (for formats)
```rust
// Every format implements these traits
trait FormatReader {
    fn read(input: &[u8]) -> Result<Document, Error>;
}
trait FormatWriter {
    fn write(doc: &Document) -> Result<Vec<u8>, Error>;
}
```

## C++ Interop

C/C++ libraries are used ONLY in `s1-text` crate via FFI:
- **HarfBuzz** — text shaping (`harfbuzz-rs`)
- **FreeType** — font loading (`freetype-rs`)
- **ICU** — Unicode ops (consider `icu4x` pure-Rust alternative)

NEVER add C/C++ dependencies to any other crate.

## What NOT To Do

- Don't add GUI/rendering code — this is a headless engine
- Don't add networking code — collaboration transport is consumer's responsibility
- Don't add async — keep the API synchronous (consumers can wrap in async)
- Don't use `unsafe` unless absolutely necessary, and document why
- Don't break the `s1-model` zero-dependency rule
- Don't merge format crate dependencies
- Don't skip writing tests for "simple" code

---

## Project State (KEEP UPDATED)

> **This section MUST be updated after every significant change, milestone completion, or phase transition.**

### Current Phase: 1 COMPLETE — Phase 2 next (Rich Documents)
### Status: s1-model (52), s1-ops (37), s1-format-txt (25), s1-format-docx (64), s1engine (28). 206 total tests.

### Phase Completion Tracker

| Phase | Status | Started | Completed | Notes |
|---|---|---|---|---|
| Phase 0: Planning | COMPLETE | 2026-03-11 | 2026-03-11 | Specs, architecture, roadmap finalized |
| Phase 1: Foundation | COMPLETE | 2026-03-11 | 2026-03-11 | 7 milestones done; 206 tests |
| Phase 2: Rich Documents | NOT STARTED | — | — | Tables, images, lists, full DOCX, ODT |
| Phase 3: Layout & Export | NOT STARTED | — | — | Text shaping, layout, PDF export |
| Phase 4: Collaboration | NOT STARTED | — | — | CRDT integration |
| Phase 5: Production | NOT STARTED | — | — | WASM, C FFI, hardening |

### Milestone Tracker (Current Phase)

Phase 1 milestones (update when Phase 1 begins):
- [x] 1.1 Project Setup — Cargo workspace, CI/CD, license
- [x] 1.2 Document Model — NodeId, Node, tree, attributes, styles (52 tests)
- [x] 1.3 Operations — Operation enum, transactions, undo/redo, cursor (37 tests)
- [x] 1.4 TXT Format — Reader/writer with encoding detection (25 tests)
- [x] 1.5 Basic DOCX Reader — ZIP, XML parsing, paragraphs, runs, formatting, styles, metadata (37 tests)
- [x] 1.6 Basic DOCX Writer — ZIP packaging, content/styles/metadata writers, round-trip tests (27 new tests, 64 total)
- [x] 1.7 Facade — Engine, Document, Format, Error, DocumentBuilder; open/create/export/undo/redo (28 tests)

### Crate Implementation Status

| Crate | Status | Tests | Notes |
|---|---|---|---|
| `s1-model` | **COMPLETE** | 52 passing | Core types, zero deps, all modules implemented |
| `s1-ops` | **COMPLETE** | 37 passing | Operations, transactions, undo/redo, cursor/selection |
| `s1-format-docx` | **COMPLETE** | 64 passing | Reader + writer: paragraphs, runs, formatting, styles, metadata, round-trip |
| `s1-format-odt` | Not started | — | ODT reader/writer |
| `s1-format-pdf` | Not started | — | PDF export |
| `s1-format-txt` | **COMPLETE** | 25 passing | Reader (UTF-8/UTF-16/Latin-1 detection), writer, round-trip |
| `s1-convert` | Not started | — | Format conversion |
| `s1-layout` | Not started | — | Page layout |
| `s1-text` | Not started | — | Text shaping (C++ FFI) |
| `s1engine` | **COMPLETE** | 28 passing | Engine, Document, Format, Error, DocumentBuilder; open/create/export; undo/redo |

### Recent Changes Log

| Date | Change | Files Affected |
|---|---|---|
| 2026-03-11 | Initial project planning and specification | docs/* |
| 2026-03-11 | Workspace setup, all crate stubs created | Cargo.toml, crates/*/Cargo.toml |
| 2026-03-11 | s1-model fully implemented (52 tests) | crates/s1-model/src/* |
| 2026-03-11 | s1-ops fully implemented (37 tests) | crates/s1-ops/src/* |
| 2026-03-11 | s1-format-txt fully implemented (25 tests) | crates/s1-format-txt/src/* |
| 2026-03-11 | s1-format-docx reader implemented (37 tests) | crates/s1-format-docx/src/* |
| 2026-03-11 | s1-format-docx writer implemented (27 new tests, 64 total) | crates/s1-format-docx/src/writer.rs, content_writer.rs, style_writer.rs, metadata_writer.rs, xml_writer.rs |
| 2026-03-11 | s1engine facade implemented (28 tests) | crates/s1engine/src/lib.rs, engine.rs, document.rs, format.rs, error.rs, builder.rs |

---

## Test Case Registry (KEEP UPDATED)

> **Update this section as tests are added. Every crate should track its test coverage here.**

### Testing Strategy Summary
- **Unit tests**: Every public function, every operation type, every node type
- **Round-trip tests**: Read → Write → Read → Compare (for all format crates)
- **Property tests**: `proptest` for model and operations (random valid inputs)
- **Fuzz tests**: `cargo-fuzz` for all format readers (malformed input)
- **Integration tests**: Cross-crate workflows (open DOCX → edit → export PDF)
- **Fixture tests**: Real-world documents in `tests/fixtures/`

### Test Cases by Crate

#### s1-model (Phase 1)
- [ ] `node_create` — Create nodes of every NodeType
- [ ] `node_id_uniqueness` — NodeIds are unique within a replica
- [ ] `node_id_cross_replica` — NodeIds from different replicas don't collide
- [ ] `tree_insert_child` — Insert child at beginning, middle, end
- [ ] `tree_remove_node` — Remove node, verify children orphaned/removed
- [ ] `tree_move_node` — Move node between parents
- [ ] `tree_traversal_dfs` — Depth-first traversal visits all nodes
- [ ] `tree_traversal_ancestors` — Walk up from node to root
- [ ] `attribute_set_get` — Set and retrieve typed attributes
- [ ] `attribute_merge` — Merge attribute maps (later values override)
- [ ] `style_resolution` — Direct formatting > character style > paragraph style > default
- [ ] `style_inheritance` — Child style inherits from parent style
- [ ] `metadata_read_write` — Set/get all metadata fields
- [ ] `media_store_dedup` — Same content hashes to same MediaId
- [ ] `proptest_tree_operations` — Random tree operations never produce invalid state

#### s1-ops (Phase 1)
- [x] `op_insert_node` — Insert node, verify tree updated
- [x] `op_delete_node` — Delete node, verify removed with descendants
- [x] `op_move_node` — Move node, verify old parent updated, new parent updated
- [x] `op_insert_text` — Insert text at beginning, middle, end of Text node
- [x] `op_delete_text` — Delete text range, verify content updated
- [x] `op_set_attributes` — Set attributes, verify merged correctly
- [x] `op_remove_attributes` — Remove specific attribute keys
- [ ] `op_split_node` — Split paragraph at offset, verify two paragraphs created
- [ ] `op_merge_nodes` — Merge adjacent paragraphs, verify single paragraph
- [x] `op_invert_insert` — Invert of insert is delete (and vice versa)
- [x] `op_invert_text` — Invert of insert-text is delete-text with same range
- [x] `op_invert_attributes` — Invert of set-attributes restores old values
- [x] `transaction_commit` — Committed transaction adds to undo stack
- [x] `transaction_rollback` — Rolled-back transaction reverts all operations
- [x] `undo_single` — Undo reverses last transaction
- [x] `undo_multiple` — Undo multiple transactions in order
- [x] `redo_after_undo` — Redo restores undone transaction
- [x] `redo_cleared_on_new_edit` — New edit after undo clears redo stack
- [x] `cursor_collapsed` — Collapsed selection (cursor) at position
- [x] `cursor_range` — Selection spanning multiple nodes
- [x] `op_validation_invalid_parent` — Reject insert into non-existent parent
- [x] `op_validation_invalid_target` — Reject delete of non-existent node
- [x] `op_validation_out_of_bounds` — Reject text insert beyond text length
- [ ] `proptest_op_invert_roundtrip` — apply(op) then apply(invert(op)) = original state
- [ ] `fuzz_random_operations` — Random operation sequences never panic

#### s1-format-txt (Phase 1)
- [x] `read_utf8` — Read UTF-8 text file (+ multibyte)
- [x] `read_utf16_bom` — Read UTF-16 LE/BE with BOM
- [x] `read_latin1` — Read Latin-1 encoded file (fallback)
- [x] `read_empty` — Empty file produces empty document
- [x] `read_single_line` — Single line → single paragraph
- [x] `read_multiple_lines` — Multiple lines → multiple paragraphs
- [x] `read_blank_lines` — Blank lines → empty paragraphs
- [x] `read_crlf` — Handle \r\n and \r line endings
- [x] `read_utf8_bom` — UTF-8 BOM stripped correctly
- [x] `read_preserves_structure` — Paragraph → Run → Text structure
- [x] `read_trailing_newline` — Trailing newline creates empty paragraph
- [x] `write_basic` — Document with paragraphs → text with newlines
- [x] `write_table` — Table → tab-separated columns
- [x] `write_strips_formatting` — Bold/italic text outputs as plain
- [x] `write_unicode` — Unicode text round-trips correctly
- [x] `roundtrip_simple` — Read → write → compare (with blank lines)
- [x] `roundtrip_unicode` — Round-trip Unicode text
- [x] `roundtrip_empty` — Round-trip empty input

#### s1-format-docx (Phase 1-2)
- [x] `read_minimal` — Minimal valid DOCX (single paragraph)
- [x] `read_paragraphs` — Multiple paragraphs with text
- [x] `read_bold_italic` — Run properties: bold, italic
- [x] `read_font_size_color` — Run properties: font, size, color
- [x] `read_paragraph_alignment` — Paragraph alignment (left, center, right, justify)
- [x] `read_paragraph_spacing` — Spacing before/after, line spacing
- [x] `read_paragraph_indent` — Left, right, first-line indent
- [x] `read_styles` — Parse styles.xml, resolve style inheritance
- [x] `read_unknown_elements` — Unknown XML elements silently skipped
- [x] `read_line_break` — Line breaks within runs
- [x] `read_page_break` — Page breaks
- [x] `read_tab` — Tab characters
- [x] `read_invalid_zip` — Invalid input produces error, not panic
- [x] `read_missing_document_xml` — Missing required file produces error
- [x] `read_metadata` — Parse docProps/core.xml (title, creator, etc.)
- [x] `read_style_parent` — Style inheritance (basedOn)
- [x] `read_bold_false` — Toggle properties with val="false"
- [ ] `read_tables` — Basic table structure (Phase 2)
- [ ] `read_merged_cells` — Column span, row span (Phase 2)
- [ ] `read_images_inline` — Inline images from word/media/ (Phase 2)
- [ ] `read_images_floating` — Floating/anchored images (Phase 2)
- [ ] `read_lists_bulleted` — Bulleted lists from numbering.xml (Phase 2)
- [ ] `read_lists_numbered` — Numbered lists (Phase 2)
- [ ] `read_lists_multilevel` — Multi-level nested lists (Phase 2)
- [ ] `read_headers_footers` — Header/footer XML files (Phase 2)
- [ ] `read_sections` — Multiple sections with different page sizes (Phase 2)
- [ ] `read_hyperlinks` — Hyperlink elements (Phase 2)
- [ ] `read_bookmarks` — Bookmark start/end (Phase 2)
- [x] `write_simple_document` — Write minimal valid DOCX
- [x] `write_bold_run` — Bold + font size run properties
- [x] `write_paragraph_alignment` — Paragraph alignment serialization
- [x] `write_paragraph_spacing` — Spacing before/after in twips
- [x] `write_escapes_special_chars` — XML escaping in text
- [x] `write_empty_paragraph` — Empty paragraph element
- [x] `write_line_break` — Line break wrapped in run
- [x] `write_font_and_color` — Font family + color properties
- [x] `write_styles` — Write styles.xml with inheritance
- [x] `write_metadata` — Write docProps/core.xml
- [x] `write_produces_valid_zip` — Output is valid ZIP with required entries
- [x] `roundtrip_text` — Read → write → read text preserved
- [x] `roundtrip_formatting` — Round-trip bold + font size preserved
- [x] `roundtrip_styles` — Round-trip style definitions preserved
- [x] `roundtrip_metadata` — Round-trip title + creator preserved
- [x] `roundtrip_multiple_paragraphs` — Round-trip multiple paragraphs
- [ ] `write_opens_in_word` — Output opens without errors in Word
- [ ] `write_opens_in_libreoffice` — Output opens in LibreOffice
- [ ] `roundtrip_tables` — Round-trip tables (Phase 2)
- [ ] `roundtrip_images` — Round-trip images (Phase 2)
- [ ] `fuzz_reader` — Fuzz DOCX reader with random ZIP/XML input

#### s1-format-odt (Phase 2)
- [ ] `read_minimal` — Minimal valid ODT
- [ ] `read_paragraphs` — Paragraphs with text:p and text:span
- [ ] `read_formatting` — ODF style properties
- [ ] `read_tables` — ODF tables
- [ ] `read_images` — Images in draw:frame
- [ ] `read_lists` — ODF list structures
- [ ] `write_minimal` — Write valid ODT
- [ ] `write_opens_in_libreoffice` — Output opens in LibreOffice
- [ ] `roundtrip_odt` — Read → write → read → compare
- [ ] `cross_format_docx_to_odt` — DOCX → model → ODT → model → compare content
- [ ] `fuzz_reader` — Fuzz ODT reader

#### s1-format-pdf (Phase 3)
- [ ] `export_single_page` — Single page text document
- [ ] `export_multi_page` — Multi-page with correct pagination
- [ ] `export_fonts_embedded` — Fonts are embedded and subsetted
- [ ] `export_images` — Images rendered correctly
- [ ] `export_tables` — Tables with borders
- [ ] `export_hyperlinks` — Clickable hyperlinks in PDF
- [ ] `export_bookmarks` — PDF outline/bookmarks
- [ ] `export_valid_pdf` — Output passes PDF validation

#### s1-layout (Phase 3)
- [ ] `layout_single_paragraph` — Single paragraph fits in one page
- [ ] `layout_line_breaking` — Long paragraph wraps correctly
- [ ] `layout_pagination` — Content exceeding page height creates new page
- [ ] `layout_widow_orphan` — Widow/orphan control
- [ ] `layout_table` — Table column widths computed correctly
- [ ] `layout_incremental` — Edit one paragraph, only affected pages re-laid out
- [ ] `layout_performance` — 100-page layout under 500ms

#### s1engine (Facade — Phase 1+)
- [x] `create_empty_document` — Create empty document via Engine
- [x] `document_metadata` — Set/get metadata through Document
- [x] `document_apply_and_undo` — Apply transaction, undo, redo through Document
- [x] `document_paragraph_ids` — Query paragraph IDs
- [x] `open_and_export_docx` — Open DOCX bytes, export, round-trip verify
- [x] `open_and_export_txt` — Open TXT bytes, export string, verify
- [x] `format_detection` — Auto-detect format from bytes (ZIP/PDF/TXT)
- [x] `unsupported_format_error` — Unsupported format returns error
- [x] `document_clear_history` — Clear undo/redo history
- [x] `detect_from_extension` — Format from file extension
- [x] `detect_from_extension_case_insensitive` — Case-insensitive extension
- [x] `detect_unknown_extension` — Unknown extension returns error
- [x] `detect_from_path` — Format from file path
- [x] `detect_from_bytes_zip/pdf/txt` — Magic byte detection
- [x] `format_extension` — Format to extension string
- [x] `format_mime_type` — Format to MIME type
- [x] `build_empty_document` — Builder produces empty doc
- [x] `build_single_paragraph` — Builder .text() shorthand
- [x] `build_heading` — Heading with auto-created style
- [x] `build_mixed_content` — Headings + paragraphs + plain text
- [x] `build_with_formatting` — Bold, italic, bold_italic runs
- [x] `build_with_metadata` — Title + author via builder
- [x] `build_with_underline` — Underline run
- [x] `build_heading_levels` — H1/H2/H3 with distinct styles
- [x] `build_with_line_break` — Line break in paragraph
- [x] `build_and_export_docx` — Builder → DOCX → reopen round-trip

#### Integration Tests
- [ ] `open_real_world_docx` — Open 10+ real DOCX files without panic
- [ ] `open_real_world_odt` — Open 10+ real ODT files without panic
- [ ] `convert_docx_to_odt` — Full conversion pipeline
- [ ] `convert_docx_to_pdf` — DOCX → layout → PDF
- [ ] `convert_docx_to_txt` — DOCX → plain text
- [ ] `large_document_perf` — 100+ page document within performance targets

### Test Fixture Documents Needed

| Fixture | Description | Format | Phase |
|---|---|---|---|
| `simple.docx` | Single paragraph, no formatting | DOCX | 1 |
| `formatted.docx` | Bold, italic, fonts, colors, sizes | DOCX | 1 |
| `styles.docx` | Heading1-6, custom styles | DOCX | 1 |
| `tables_basic.docx` | Simple 3x3 table | DOCX | 2 |
| `tables_merged.docx` | Table with merged cells | DOCX | 2 |
| `tables_nested.docx` | Table inside a table cell | DOCX | 2 |
| `images_inline.docx` | Inline PNG and JPEG images | DOCX | 2 |
| `images_floating.docx` | Floating/anchored images | DOCX | 2 |
| `lists.docx` | Bulleted, numbered, multi-level lists | DOCX | 2 |
| `headers_footers.docx` | Headers, footers, page numbers | DOCX | 2 |
| `sections.docx` | Multiple sections, landscape + portrait | DOCX | 2 |
| `hyperlinks.docx` | Internal and external hyperlinks | DOCX | 2 |
| `comments.docx` | Document with comments | DOCX | 2 |
| `bidi.docx` | Arabic/Hebrew bidirectional text | DOCX | 3 |
| `cjk.docx` | Chinese/Japanese/Korean text | DOCX | 3 |
| `large_100p.docx` | 100+ page document (performance) | DOCX | 3 |
| `simple.odt` | Basic ODT document | ODT | 2 |
| `formatted.odt` | ODT with formatting | ODT | 2 |
| `legacy.doc` | Legacy DOC binary format | DOC | 3 |

---

## Maintenance Instructions

### After Every Code Change
1. Run `cargo test` — all tests must pass
2. Run `cargo clippy -- -D warnings` — no warnings
3. Run `cargo fmt --check` — formatting correct
4. Update the **Crate Implementation Status** table above if a crate's status changed
5. Update the **Test Case Registry** — mark completed tests with [x]

### After Every Milestone Completion
1. Mark milestone as complete in **Milestone Tracker**
2. Update **Recent Changes Log** with date and summary
3. Update the **Phase Completion Tracker** if phase changed
4. Review and update **Crate Implementation Status** table

### After Every Phase Completion
1. Update **Current Phase** at the top of Project State
2. Add **Phase Completion** date
3. Add new phase's milestones to **Milestone Tracker**
4. Review all docs for accuracy — architecture may have evolved
5. Update `docs/ROADMAP.md` with actual timelines vs planned
