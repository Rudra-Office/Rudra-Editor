# s1engine — AI Development Context

## What Is This Project?

s1engine is a modular document engine SDK built in pure Rust. It reads, writes, edits, and converts document formats (DOCX, ODT, PDF, TXT). Includes CRDT-based collaborative editing.

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
- This is non-negotiable — it's the foundation for undo/redo and CRDT collaboration
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
  s1-crdt/           CRDT algorithms for collaborative editing
  s1-layout/         Page layout engine (pagination, line breaking)
  s1-text/           Text processing (rustybuzz, ttf-parser, fontdb — pure Rust)
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

## Text Processing

`s1-text` uses pure-Rust alternatives instead of C/C++ FFI:
- **rustybuzz** — text shaping (pure Rust HarfBuzz port)
- **ttf-parser** — font parsing (pure Rust)
- **fontdb** — system font discovery
- **unicode-bidi** — BiDi support (UAX #9)
- **unicode-linebreak** — line breaking (UAX #14)

This eliminates all C/C++ dependencies while providing full Unicode support.

## What NOT To Do

- Don't add GUI/rendering code — this is a headless engine
- Don't add networking code — collaboration transport is consumer's responsibility
- Don't add async — keep the API synchronous (consumers can wrap in async)
- Don't use `unsafe` unless absolutely necessary, and document why
- Don't break the `s1-model` zero-dependency rule
- Don't merge format crate dependencies
- Don't skip writing tests for "simple" code

## Editor UI Standards (MUST Follow)

All UI modifications to the editor (`editor/src/`, `editor/index.html`, `demo/`) must follow these rules:

1. **Clean & Professional** — No funky/bright/neon colors. Use neutral, professional palettes (grays, whites, subtle blues). Match the look of Google Docs, Microsoft Word Online, or OnlyOffice.
2. **No Emojis in Layout** — Never use emoji characters as icons or labels in the UI. Use text labels or Unicode symbols (like checkmark/cross) sparingly and only when they look professional.
3. **Tooltips Required** — Every toolbar button, dropdown, and interactive element must have a descriptive `title` attribute tooltip (e.g., `title="Bold (Ctrl+B)"`).
4. **Competitor Parity** — The editor should look and feel like a real document editor. Reference Google Docs / Word Online / OnlyOffice for toolbar layout, color schemes, dropdown styling, font choices, spacing.
5. **Production Grade** — No placeholder styling, no TODO colors, no debug borders. Every element should look finished and polished.

## Collaboration Relay Server

For E-19 (real-time collaboration UI), use a lightweight **Node.js WebSocket relay server** (`scripts/relay.js`). This is the simplest approach for broadcasting CRDT operations between peers.

---

## Project State (KEEP UPDATED)

> **This section MUST be updated after every significant change, milestone completion, or phase transition.**

### Current Phase: Post-Production — All Issues Resolved, Full Collaboration Support
### Status: s1-model (72), s1-ops (48), s1-format-txt (41), s1-format-docx (201), s1-format-odt (119), s1-format-md (32), s1-format-pdf (23), s1-convert (90), s1-layout (62), s1-text (39), s1engine (143 = 120 + 23 real-world), s1-crdt (182), s1engine-wasm (105), s1engine-c (10), proptests (5). ~1,172 total tests.

### Phase Completion Tracker

| Phase | Status | Started | Completed | Notes |
|---|---|---|---|---|
| Phase 0: Planning | COMPLETE | 2026-03-11 | 2026-03-11 | Specs, architecture, roadmap finalized |
| Phase 1: Foundation | COMPLETE | 2026-03-11 | 2026-03-11 | 7 milestones done; 206 tests |
| Phase 2: Rich Documents | COMPLETE | 2026-03-11 | 2026-03-12 | 6 milestones; tables, images, lists, sections, ODT, advanced DOCX |
| Phase 3: Layout & Export | COMPLETE | 2026-03-12 | 2026-03-12 | Layout complete; PDF polish (images, hyperlinks, bookmarks) deferred to 3.6 |
| Phase 4: Collaboration | COMPLETE | 2026-03-12 | 2026-03-12 | 4 milestones; Fugue text CRDT, tree CRDT, LWW attributes/metadata, CollabDocument API, awareness, serialization, compression; 172 tests |
| Phase 5: Production | COMPLETE | 2026-03-12 | 2026-03-12 | WASM bindings (12 tests), C FFI (10 tests), proptest (4 tests), security hardening (ZIP bomb limits, image dimension caps) |

### Milestone Tracker (Current Phase)

Phase 1 milestones (update when Phase 1 begins):
- [x] 1.1 Project Setup — Cargo workspace, CI/CD, license
- [x] 1.2 Document Model — NodeId, Node, tree, attributes, styles (52 tests)
- [x] 1.3 Operations — Operation enum, transactions, undo/redo, cursor (37 tests)
- [x] 1.4 TXT Format — Reader/writer with encoding detection (25 tests)
- [x] 1.5 Basic DOCX Reader — ZIP, XML parsing, paragraphs, runs, formatting, styles, metadata (37 tests)
- [x] 1.6 Basic DOCX Writer — ZIP packaging, content/styles/metadata writers, round-trip tests (27 new tests, 64 total)
- [x] 1.7 Facade — Engine, Document, Format, Error, DocumentBuilder; open/create/export/undo/redo (28 tests)

Phase 2 milestones:
- [x] 2.1 Tables — DOCX table read/write, builder API (19 new tests)
- [x] 2.2 Images — DOCX inline image read/write, MediaStore, round-trip (7 new tests)
- [x] 2.3 Lists — numbering parser/writer, numPr read/write, builder (30 new tests)
- [x] 2.4 Sections, Headers, Footers — section model, sectPr, header/footer, fields, builder (29 new tests)
- [x] 2.5 ODT Format — Full ODT reader/writer with paragraphs, formatting, tables, images, lists, styles, metadata (63 tests)
- [x] 2.6 Advanced DOCX Features — Hyperlinks, bookmarks, tab stops, paragraph borders/shading, character spacing, superscript/subscript, comments (read/write/round-trip/builder). 43 new tests.

Phase 3 milestones:
- [x] 3.1 Text Processing (`s1-text`) — Pure-Rust text shaping (rustybuzz), font parsing (ttf-parser), font discovery (fontdb), BiDi (unicode-bidi), line breaking (unicode-linebreak). 39 tests.
- [x] 3.2 Layout Engine (`s1-layout`) — Style resolution, Knuth-Plass line breaking, block stacking, pagination, table layout, image placement, header/footer placement, widow/orphan control, page-number substitution. 30 tests.
- [x] 3.3 Incremental Layout — Content-hash-based LayoutCache, cache hit/miss/invalidation. 8 tests.
- [x] 3.4 PDF Export (`s1-format-pdf`) — Core: font embedding/subsetting, text rendering, table borders, metadata. 8 tests.
- [x] 3.5 Format Conversion (`s1-convert`) — DOC reader (OLE2/CFB heuristic text extraction), cross-format conversion pipeline (DOC/DOCX/ODT → DOCX/ODT), format detection. 15 tests.
- [x] 3.6 PDF Polish — Image embedding (JPEG/PNG), hyperlink annotations, bookmarks/outline. 13 tests.

Phase 4 milestones:
- [x] 4.1 Core CRDT Primitives — LamportClock, VectorClock, OpId, StateVector, CrdtOperation, CrdtError (25 tests)
- [x] 4.2 CRDT Algorithms — Fugue-based TextCrdt, TreeCrdt, AttrCrdt, MetadataCrdt, CrdtResolver, TombstoneTracker (40 tests)
- [x] 4.3 Collaboration API — CollabDocument, AwarenessState, binary serialization, operation compression (40 tests)
- [x] 4.4 Collaboration Testing — 16 convergence tests (multi-replica, partition/heal, snapshot sync), 17 scenario tests (concurrent edits, deterministic ordering, undo). 33 integration tests.

Phase 5 milestones:
- [x] 5.1 WASM Bindings — WasmEngine, WasmDocument, WasmDocumentBuilder, WasmFontDatabase, detect_format. 12 tests.
- [x] 5.2 C FFI Bindings — s1_engine/document/error/string/bytes opaque handles, null-safety, format roundtrip. 10 tests.
- [x] 5.3 Performance & Hardening — Proptest for model tree invariants, ops inversion roundtrip, CRDT convergence (4 tests). ZIP bomb size limits (256MB/64MB). Image dimension caps (16384px). Security hardening in DOCX/ODT/PDF.

### Crate Implementation Status

| Crate | Status | Tests | Notes |
|---|---|---|---|
| `s1-model` | **COMPLETE** | 72 passing | Core types, zero deps, all modules + numbering defs + sections + proptest tree invariants + Unicode text safety + cycle detection + is_descendant |
| `s1-ops` | **COMPLETE** | 48 passing | Operations, transactions, undo/redo, cursor/selection + proptest inversion roundtrip + subtree undo + mixed attribute undo + Unicode text roundtrip |
| `s1-format-docx` | **COMPLETE** | 201 passing | Reader + writer: paragraphs, runs, formatting, styles, metadata, tables, images (inline + floating/anchor with positioning), lists, sections, headers/footers, fields (fldSimple + fldChar complex, non-self-closing), hyperlinks, bookmarks, tab stops, paragraph borders/shading, character spacing, superscript/subscript, comments, TOC (SDT), mc:AlternateContent image support (run + paragraph level, fallback skipping), track changes (w:ins/w:del/w:rPrChange), VML shapes (rect/oval/line/roundrect with dimensions, fill/stroke, raw XML round-trip), round-trip. ZIP bomb protection. |
| `s1-format-odt` | **COMPLETE** | 119 passing | Reader + writer: paragraphs, runs, formatting, styles, metadata, tables (with table:name, table-column), images (text:anchor-type), lists (nested XML well-formed), auto-styles, TOC, superscript/subscript, character spacing, paragraph shading, keep-lines-together, hyperlinks, bookmarks, tab stops, paragraph borders, comments (annotations, xmlns:dc), headers/footers (with page-number text:select-page), sections (page size, margins, orientation), office:version="1.2", text line-break/tab conversion, conditional manifest. ZIP bomb protection. |
| `s1-format-md` | **COMPLETE** | 32 passing | Reader (pulldown-cmark): headings, bold/italic/strikethrough, code, links, lists, GFM tables, thematic breaks. Writer: Markdown generation from DocumentModel. |
| `s1-format-pdf` | **COMPLETE** | 23 passing | PDF export: font embedding/subsetting, text rendering, tables, metadata, images (JPEG/PNG), hyperlinks, bookmarks, PDF/A-1b compliance (ICC sRGB profile, XMP metadata, output intent). Image dimension caps. |
| `s1-format-txt` | **COMPLETE** | 41 passing | Reader (UTF-8/UTF-16/Latin-1 detection, heading/list/break markers), writer (headings, lists, TOC, thematic breaks), round-trip |
| `s1-convert` | **COMPLETE** | 90 passing | DOC reader (FIB + piece table + CHPx/SPRM char formatting + PAPx/SPRM para formatting + font table + stylesheet + table detection + SummaryInformation metadata), cross-format conversion (DOC/DOCX/ODT → DOCX/ODT), format detection |
| `s1-layout` | **COMPLETE** | 62 passing | Style resolution, Knuth-Plass line breaking, pagination, row-by-row table splitting across pages (header repeat, continuation flag), image placement, header/footer placement, widow/orphan control, page-number field substitution, incremental layout cache (with width/indent context), multi-section layout, paginated HTML output (layout_to_html). GlyphRun: superscript/subscript/highlight_color/character_spacing/revision_type/revision_author. LayoutTableCell: per-side borders + background_color. Paragraph: text_align/background_color/border/list_marker/list_level. Track changes ins/del in HTML. Bookmark Y-positioning. |
| `s1-text` | **COMPLETE** | 39 passing | Pure Rust: text shaping (rustybuzz), font parsing (ttf-parser), font discovery (fontdb), BiDi, line breaking |
| `s1-crdt` | **COMPLETE** | 182 passing | Fugue text CRDT, tree CRDT, LWW attr/metadata, resolver, CollabDocument, awareness, binary serialization, compression, tombstones, op_log compaction, gc_tombstones, auto_compact, snapshot_and_truncate; 16 convergence + 17 scenario + 1 proptest integration tests |
| `s1engine` | **COMPLETE** | 143 passing (64 unit + 56 integration + 23 real-world) | Engine, Document, Format, Error, DocumentBuilder, TableBuilder, list builder, section/header/footer builder, hyperlink/bookmark/superscript/subscript/TOC builder; open/create/export; undo/redo; ODT support; feature-gated CRDT re-exports; feature-gated layout facade; feature-gated PDF export (export_pdf/export_pdf_with_config); track changes accept/reject API; 12 fidelity integration tests; 23 real-world document tests (testdocs/) |
| `s1engine-wasm` | **COMPLETE** | 105 passing | WASM bindings: WasmEngine, WasmDocument, WasmDocumentBuilder, WasmFontDatabase, format detection, to_paginated_html (with config/fonts), to_pdf (with fonts/data_url), track changes accept/reject, visual indicators, editor API (render_node_html, split_paragraph, merge_paragraphs, get_formatting_json, set_heading_level), P.1 selection/range formatting (split_run, format_run, format_selection, get_run_ids/text/formatting, get_selection_formatting_json), P.2 table ops (insert_table, insert/delete row/column, cell text, dimensions, merge, background), P.3 image ops (insert/delete/resize, data_url, alt_text), P.4 structural (hyperlinks, bookmarks, lists, page_break, hr, comments, sections), P.5 find/replace (find_text, replace_text, replace_all, paste_plain_text, get_document_text), P.6 collaboration (create_collab, state_vector, op_log, tombstone, compact, undo/redo, changes_since, peers, free) |
| `s1engine-c` | **COMPLETE** | 10 passing | C FFI: opaque handles, null-safety, error handling, format roundtrip |

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
| 2026-03-11 | Milestone 2.1: Tables — DOCX read/write, builder (19 new tests, 83 docx, 32 s1engine) | property_parser.rs, content_parser.rs, content_writer.rs, writer.rs, builder.rs |
| 2026-03-11 | Milestone 2.2: Images — DOCX read/write, round-trip (7 new tests, 90 docx total) | content_parser.rs, content_writer.rs, reader.rs, writer.rs, xml_util.rs |
| 2026-03-11 | Milestone 2.3: Lists — numbering parser/writer, numPr read/write, builder (30 new tests) | numbering.rs, numbering_parser.rs, numbering_writer.rs, property_parser.rs, content_parser.rs, content_writer.rs, reader.rs, writer.rs, builder.rs |
| 2026-03-11 | Milestone 2.4: Sections, Headers, Footers — section model, sectPr parser/writer, header/footer parser/writer, field support, builder API (29 new tests) | section.rs, section_parser.rs, section_writer.rs, header_footer_parser.rs, header_footer_writer.rs, content_parser.rs, content_writer.rs, reader.rs, writer.rs, builder.rs, lib.rs |
| 2026-03-12 | Milestone 2.5: ODT Format — full reader/writer crate with paragraphs, formatting, tables, images, lists, styles, metadata, auto-styles, round-trip (63 new tests, 2 s1engine integration tests) | crates/s1-format-odt/src/* (11 modules), crates/s1engine/src/engine.rs, document.rs, error.rs, lib.rs |
| 2026-03-12 | Milestone 2.6: Advanced DOCX — hyperlinks (external/internal/tooltip, rId resolution), bookmarks (start/end), tab stops (left/center/right/decimal with leaders), paragraph borders, paragraph shading, character spacing, superscript/subscript, comments (parser/writer/round-trip); builder API (hyperlink, bookmark_start/end, superscript, subscript); 43 new tests | comments_parser.rs, comments_writer.rs, content_parser.rs, content_writer.rs, property_parser.rs, writer.rs, reader.rs, builder.rs, lib.rs, node.rs |
| 2026-03-12 | Milestone 3.1: Text Processing — pure-Rust text shaping via rustybuzz, font parsing via ttf-parser, system font discovery via fontdb, BiDi via unicode-bidi, line breaking via unicode-linebreak (39 tests) | crates/s1-text/src/* (7 modules) |
| 2026-03-12 | Milestone 3.2: Layout Engine — style resolver, greedy line breaking, block stacking with spacing, pagination, table layout, image placement, page-break-before support (22 tests) | crates/s1-layout/src/* (4 modules) |
| 2026-03-12 | Milestone 3.4: PDF Export — PDF generation from LayoutDocument, CIDFont embedding with subsetting, glyph width tables, content streams, table border rendering, metadata, multi-page support (8 tests) | crates/s1-format-pdf/src/* (3 modules) |
| 2026-03-12 | Milestone 3.5: Format Conversion — DOC reader (OLE2/CFB heuristic text extraction), cross-format pipeline (DOC/DOCX/ODT → DOCX/ODT), format detection, convert_to_model API (15 tests) | crates/s1-convert/src/* (4 modules) |
| 2026-03-12 | Layout Polish: Knuth-Plass optimal line breaking, header/footer placement from SectionProperties, page-number field substitution (PAGE/NUMPAGES), widow/orphan control, section page size resolution (8 new tests, 30 total) | crates/s1-layout/src/engine.rs |
| 2026-03-12 | Phase 4 CRDT Integration: Milestone 4.1 — LamportClock, VectorClock, OpId, StateVector, CrdtOperation, CrdtError (25 unit tests) | crates/s1-crdt/src/* (clock.rs, op_id.rs, state_vector.rs, crdt_op.rs, error.rs, lib.rs) |
| 2026-03-12 | Milestone 4.2 — Fugue-based TextCrdt (YATA integration), TreeCrdt (Kleppmann moves, cycle detection), AttrCrdt (per-key LWW), MetadataCrdt, CrdtResolver, TombstoneTracker (40 tests) | crates/s1-crdt/src/* (text_crdt.rs, tree_crdt.rs, attr_crdt.rs, metadata_crdt.rs, resolver.rs, tombstone.rs) |
| 2026-03-12 | Milestone 4.3 — CollabDocument (fork, snapshot, apply_local/remote, changes_since, undo/redo), AwarenessState, binary serialization, operation compression (40 tests) | crates/s1-crdt/src/* (collab.rs, awareness.rs, serialize.rs, compression.rs) |
| 2026-03-12 | Milestone 4.4 — 16 convergence tests (2/3/5 replicas, partition/heal, snapshot sync, delayed delivery) + 17 scenario tests (concurrent inserts, attribute LWW, delete+modify, undo) | crates/s1-crdt/tests/* (convergence.rs, scenarios.rs) |
| 2026-03-12 | Feature-gated CRDT integration into s1engine facade: crdt feature flag, create_collab/open_collab methods, conditional re-exports, CrdtError variant | Cargo.toml, crates/s1engine/src/* (lib.rs, engine.rs, error.rs) |
| 2026-03-12 | Milestone 3.6: PDF Polish — image embedding (JPEG DCTDecode pass-through, PNG decode+FlateDecode), hyperlink annotations, PDF outline/bookmarks (13 tests) | crates/s1-layout/src/types.rs, engine.rs, crates/s1-format-pdf/src/writer.rs, Cargo.toml |
| 2026-03-12 | Milestone 3.3: Incremental Layout — content-hash-based LayoutCache with FNV-1a, per-block caching, cache hit/miss/invalidation (8 tests) | crates/s1-layout/src/types.rs, engine.rs, lib.rs |
| 2026-03-12 | Milestone 5.1: WASM Bindings — WasmEngine, WasmDocument, WasmDocumentBuilder, WasmFontDatabase, detect_format, wasm32 font guard (12 tests) | ffi/wasm/*, Cargo.toml, crates/s1-text/src/font_db.rs |
| 2026-03-12 | Milestone 5.2: C FFI Bindings — opaque handles (S1Engine, S1Document, S1Error, S1Bytes, S1String), extern "C" functions, null-safety (10 tests) | ffi/c/* |
| 2026-03-12 | Milestone 5.3: Hardening — proptest for model tree invariants, ops inversion roundtrip, CRDT convergence (4 tests). ZIP bomb limits in DOCX/ODT readers. Image dimension caps in PDF writer. | crates/s1-model/src/tree.rs, crates/s1-ops/src/operation.rs, crates/s1-crdt/tests/proptests.rs, crates/s1-format-docx/src/reader.rs, crates/s1-format-odt/src/reader.rs, crates/s1-format-pdf/src/writer.rs |
| 2026-03-12 | P0 Correctness: Unicode-safe text ops (char_offset_to_byte helper, char-based validation), cycle detection (is_descendant + move_node guard), subtree undo (full DFS snapshot + restore_node), mixed attribute undo (remove added keys + restore overwritten values). 21 new regression tests. | crates/s1-model/src/tree.rs, crates/s1-ops/src/operation.rs |
| 2026-03-12 | P1 Documentation truthfulness: Rewrote README.md (actual status, real API examples, format support matrix), API_DESIGN.md (examples matching real facade API), DEPENDENCIES.md (pure Rust stack, correct deps), ARCHITECTURE.md (correct file tree, no C++ FFI, Fugue CRDT). Added model_mut() escape hatch documentation. | README.md, docs/API_DESIGN.md, docs/DEPENDENCIES.md, docs/ARCHITECTURE.md, crates/s1engine/src/document.rs |
| 2026-03-12 | P2 Hardening: 11 invariant integration tests (undo/redo reversibility, cross-format text preservation, builder output validity, Unicode roundtrip, tree integrity). CLI examples (convert, create_report). CHANGELOG.md. | crates/s1engine/tests/invariants.rs, crates/s1engine/examples/convert.rs, crates/s1engine/examples/create_report.rs, CHANGELOG.md |
| 2026-03-12 | Table of Contents support — NodeType::TableOfContents, TocMaxLevel/TocTitle attributes, collect_headings(), update_toc(), builder API (table_of_contents/table_of_contents_with_title), DOCX SDT read/write, ODT text:table-of-content read/write, TXT fallback generation, layout engine expansion. 14 new tests. | node.rs, attributes.rs, tree.rs, document.rs, builder.rs, content_parser.rs, content_writer.rs (docx+odt), writer.rs (txt), engine.rs (layout) |
| 2026-03-12 | Fidelity F.1: ODT Quick Wins — superscript/subscript (style:text-position), character spacing (fo:letter-spacing), paragraph shading (fo:background-color), keep-lines-together (fo:keep-together). 10 new tests. | crates/s1-format-odt/src/property_parser.rs, property_writer.rs |
| 2026-03-12 | Fidelity F.2: Markdown Format — s1-format-md crate with pulldown-cmark reader and Markdown writer. Headings, bold/italic/strikethrough, inline code, code blocks, hyperlinks, ordered/unordered/nested lists, GFM tables, line breaks, thematic breaks, Unicode. Integrated into s1engine facade (Format::Md, open/export). 32 new tests. | crates/s1-format-md/src/* (lib.rs, reader.rs, writer.rs), Cargo.toml, crates/s1engine/src/* |
| 2026-03-12 | Fidelity F.3: ODT Hyperlinks + Bookmarks — parse_hyperlink_into for <text:a> with xlink:href, bookmark-start/end/collapsed parsing, stateful hyperlink writing. 8 new tests. | crates/s1-format-odt/src/content_parser.rs, content_writer.rs |
| 2026-03-12 | Fidelity F.4: ODT Tab Stops + Paragraph Borders — parse_paragraph_properties_children for <style:tab-stops>, parse_border_value for fo:border-*, border_side_to_odf writer, tab stop child elements. 7 new tests. | crates/s1-format-odt/src/property_parser.rs, property_writer.rs, style_parser.rs |
| 2026-03-12 | Fidelity F.5: TXT Fidelity — heading # markers, bullet - markers, numbered N. markers, nested list indent, thematic break --- (PageBreakBefore), marker detection in reader. 14 new tests. | crates/s1-format-txt/src/writer.rs, reader.rs |
| 2026-03-12 | Fidelity F.6: ODT Comments — parse/write office:annotation with dc:creator, dc:date, text:p body. CommentStart/CommentEnd inline, CommentBody on root. annotation-end parsing. 7 new tests. | crates/s1-format-odt/src/content_parser.rs, content_writer.rs |
| 2026-03-13 | Fidelity F.7: ODT Headers/Footers/Sections — parse style:page-layout (page-layout-properties), parse style:master-page (header/footer with text + page-number/page-count fields), parse header-first/footer-first (ODF 1.3). Build SectionProperties + Header/Footer nodes from master page. Write page-layout + master-page with header/footer content to styles.xml. Round-trip page layout, headers, footers, first-page headers, footer with page number fields. 12 new tests. | crates/s1-format-odt/src/style_parser.rs, style_writer.rs, reader.rs, writer.rs |
| 2026-03-13 | Release 1.0.0: R.1 `#[non_exhaustive]` on 25 public enums + wildcard match arms. R.2 thiserror migration for all error types (except s1-model). R.3 API surface audit: added missing re-exports (Borders, BorderSide, BorderStyle, TabStop, TabAlignment, TabLeader, MediaId, MediaStore, TableWidth, VerticalAlignment). R.4 CI/CD GitHub Actions (7 jobs). R.5 crates.io metadata. R.6 large-document benchmarks. R.7 version bump to 1.0.0. | ~40 files across workspace |
| 2026-03-13 | Bug Fix: Complex field (fldChar) parsing — added FieldState machine to track fldChar begin/instrText/separate/end across runs within a paragraph. Creates Field nodes (PageNumber/PageCount/etc.) for complex field format. Fixes page number loss in footers. 1 new test (single-run), 1 new test (cross-run). | crates/s1-format-docx/src/content_parser.rs |
| 2026-03-13 | Bug Fix: mc:AlternateContent image parsing — added `parse_alternate_content()` to descend into `<mc:Choice>` and extract `<w:drawing>` elements. Fixes image loss in DOCX files produced by Google Docs and other editors that wrap drawings in AlternateContent. 1 new test. | crates/s1-format-docx/src/content_parser.rs |
| 2026-03-13 | WASM: Header/footer rendering in to_html() — renders first header and footer from sections with border styling. Build tooling: Makefile (build/test/wasm/demo targets), scripts/build-wasm.sh, scripts/test.sh. | ffi/wasm/src/lib.rs, Makefile, scripts/*.sh |
| 2026-03-13 | Milestone B.1: Expose Layout Engine Through Facade — `layout` feature flag (optional), `Document::layout()` and `layout_with_config()` methods, `LayoutError` variant in Error enum, conditional re-exports for layout/text types. 6 new tests. | crates/s1engine/Cargo.toml, src/document.rs, src/error.rs, src/lib.rs |
| 2026-03-13 | Milestone C.1: Multi-Section Layout — refactored layout engine for multi-section support. Per-section page sizes/margins via `resolve_page_layout_for_section()`, section block mapping via `build_section_map()`, section break types (NextPage/Continuous/EvenPage/OddPage with blank page insertion), per-section header/footer layout. Fixed widow/orphan control to preserve intentionally blank pages. 8 new tests. | crates/s1-layout/src/engine.rs |
| 2026-03-13 | Milestone C.3: Track Changes Read/Write — RevisionType/Author/Date/Id/OriginalFormatting attribute keys in s1-model. DOCX parser: w:ins, w:del (block + inline), w:rPrChange, w:delText. DOCX writer: grouping tracked runs under w:ins/w:del wrappers, w:delText for deleted content, w:rPrChange emission. 14 new tests (8 parser, 3 writer, 3 roundtrip). | crates/s1-model/src/attributes.rs, crates/s1-format-docx/src/content_parser.rs, content_writer.rs, property_parser.rs, writer.rs |
| 2026-03-14 | Milestone B.4: Browser Demo Paginated Viewer — Added "Pages" tab to demo/index.html showing paginated layout view from `to_paginated_html()`. Page navigation (Previous/Next with scroll-to-page), page count indicator, page dimensions display, lazy rendering on tab switch, graceful fallback when layout engine not available in WASM build. Styled pages with white backgrounds on gray container, drop shadows, 20px gaps, centered layout. | demo/index.html |
| 2026-03-14 | Hardening Plan Phase 1 — A.1 FIB/Piece Table (24 tests), B.1 Layout Facade (6 tests), B.2 Paginated HTML (10 tests), C.1 Multi-Section Layout (8 tests), C.5 CRDT Hardening (10 tests). | crates/s1-convert/src/fib.rs, piece_table.rs; crates/s1-layout/src/html.rs, engine.rs; crates/s1-crdt/src/collab.rs, compression.rs, resolver.rs |
| 2026-03-14 | Hardening Plan Phase 2 — A.2 CHPx/SPRM char formatting (12 tests), A.3 PAPx/SPRM para formatting (14 tests), C.2 Tables across page breaks (6 tests), B.3 Wire HTML into WASM (6 tests). | crates/s1-convert/src/sprm.rs, chpx.rs, papx.rs; crates/s1-layout/src/engine.rs, types.rs; ffi/wasm/src/lib.rs |
| 2026-03-14 | Hardening Plan Phase 3 — A.4 Style sheet/font table (17 tests), A.5 Tables/metadata (8 tests), C.4 Track changes accept/reject API (6 tests), C.6 Fidelity testing (12 tests). | crates/s1-convert/src/font_table.rs, stylesheet.rs, summary_info.rs, doc_reader.rs; crates/s1engine/src/document.rs; crates/s1engine/tests/fidelity.rs |
| 2026-03-14 | Bug Fixes: ODT content.xml — nested lists well-formed XML, xmlns:dc namespace, office:version="1.2", table:table-column/table:name, text:anchor-type on images, text line-break/tab conversion, conditional manifest meta.xml, text:select-page on page-number. 3 new tests. | crates/s1-format-odt/src/content_writer.rs, style_writer.rs, manifest_writer.rs, writer.rs |
| 2026-03-14 | Bug Fix: Footer page numbers — handle_fld_char helper for non-self-closing fldChar elements. Google Docs images — parse_alternate_content_into_paragraph for paragraph-level mc:AlternateContent, fallback skipping. 5 new tests. | crates/s1-format-docx/src/content_parser.rs, header_footer_parser.rs |
| 2026-03-14 | WASM PDF Export — to_pdf(), to_pdf_with_fonts(), to_pdf_data_url(), to_pdf_data_url_with_fonts() on WasmDocument. Feature-gated pdf export on s1engine (export_pdf, export_pdf_with_config). 4 new tests. | ffi/wasm/src/lib.rs, Cargo.toml; crates/s1engine/src/document.rs, error.rs, Cargo.toml |
| 2026-03-14 | Phase E.1: Editor WASM API — render_node_html(), split_paragraph(), merge_paragraphs(), get_formatting_json(), set_heading_level(). Fixed find_first_text_node Unicode bug (bytes→chars). 16 new tests (58 total WASM). Phase E.2: Full demo rewrite as WYSIWYG editor (Google Docs style) — menu bar, formatting toolbar, contentEditable, Enter/Backspace paragraph handling, keyboard shortcuts, drag-drop, export dropdown, Pages/Text views, track changes bar. | ffi/wasm/src/lib.rs, demo/index.html |
| 2026-03-14 | Phase P.1: Selection & Range Formatting — split_run (split Run at char offset, preserve attributes), format_run (set attribute on specific run), format_selection (format text range spanning runs/paragraphs, auto-splits runs at boundaries, single transaction), get_run_ids/get_run_text/get_run_formatting_json, get_selection_formatting_json (uniform/mixed detection). Helper: format_range_in_paragraph, split_run_internal, parse_format_kv. 12 new tests. | ffi/wasm/src/lib.rs |
| 2026-03-14 | Phase P.2: Table Operations — insert_table (creates Table>Row>Cell>Para>Run>Text structure), insert_table_row, delete_table_row, insert_table_column, delete_table_column, set_cell_text, get_cell_text, get_table_dimensions, merge_cells (ColumnSpan/RowSpan), set_cell_background. 10 new tests. | ffi/wasm/src/lib.rs |
| 2026-03-14 | Phase P.3: Image Operations — insert_image (Image directly under Paragraph, not Run per model constraints), delete_image, resize_image, get_image_data_url (base64 data URL), set_image_alt_text. Test helper for non-wasm targets. 6 new tests. | ffi/wasm/src/lib.rs |
| 2026-03-14 | Phase P.4: Structural Elements — insert_hyperlink/remove_hyperlink, insert_bookmark, set_list_format (bullet/decimal/none), insert_page_break, insert_horizontal_rule, get_comments_json, insert_comment (CommentRangeStart/End/Body), delete_comment, get_sections_json. 10 new tests. | ffi/wasm/src/lib.rs |
| 2026-03-14 | Phase P.5: Find & Replace — find_text (case-sensitive/insensitive, returns JSON matches), replace_text (single occurrence), replace_all (atomic transaction, returns count), paste_plain_text (splits on newlines into new paragraphs), get_document_text (full document text). 6 new tests. | ffi/wasm/src/lib.rs |
| 2026-03-14 | Phase P.7: Demo Editor Rewrite — Complete rewrite of demo/index.html as operation-based editor. All formatting via WASM format_selection() (zero document.execCommand calls). Selection mapping (DOM → WASM node IDs + char offsets). WASM-based: bold/italic/underline/strike/super/sub toggle, font family/size/color, alignment, lists (bullet/numbered via set_list_format), tables (insert via WASM + context menu for row/col insert/delete/cell background), images (insert/delete/resize via WASM), hyperlinks, page breaks, horizontal rules. Find & Replace UI (Ctrl+F, highlight matches, replace/replace all via WASM). Toolbar state from get_selection_formatting_json (uniform/mixed). Insert dropdown menu. Paste via WASM paste_plain_text for multi-paragraph. | demo/index.html |
| 2026-03-14 | Updated research.md — Track changes now DONE (was NOT IMPLEMENTED). DOC format features (FIB, piece table, CHPx, PAPx, font table, stylesheet, tables, metadata) now DONE. Paginated HTML DONE. Image alt text DONE. Added sections 12 (WASM Editor API) and 13 (Collaboration Standards). Updated compliance summary. | research.md |
| 2026-03-14 | Updated ROADMAP.md — Added Phase 8 (Production Editor API, P.1-P.5 COMPLETE) and Phase 9 (Production Editor Demo, P.7 IN PROGRESS, P.6/P.8/P.9 PLANNED). Updated phase overview bar. | docs/ROADMAP.md |
| 2026-03-14 | Floating Image Write Support — 7 new AttributeKey variants (ImagePositionType, ImageWrapType, ImageHorizontalOffset, ImageVerticalOffset, ImageHorizontalRelativeFrom, ImageVerticalRelativeFrom, ImageDistanceFromText). DOCX parser extracts wp:anchor positioning (positionH/V, posOffset, wrap types, distances). DOCX writer conditionally outputs wp:anchor or wp:inline. 3 new tests (parser with full positioning, writer anchor, writer inline default). | crates/s1-model/src/attributes.rs, crates/s1-format-docx/src/content_parser.rs, content_writer.rs |
| 2026-03-14 | ODT settings.xml — New settings_parser.rs (parse zoom factor from config items) and settings_writer.rs (generate settings.xml with view/configuration settings). Reader wired up to parse settings.xml. Writer generates settings.xml in ZIP. Manifest updated to include settings.xml entry. 6 new tests. | crates/s1-format-odt/src/settings_parser.rs, settings_writer.rs, reader.rs, writer.rs, manifest_writer.rs, lib.rs |
| 2026-03-14 | VML Shape Support — NodeType::Drawing allowed as child of Paragraph. New AttributeKey variants (ShapeType, ShapeWidth, ShapeHeight, ShapeFillColor, ShapeStrokeColor, ShapeStrokeWidth, ShapeRawXml). DOCX parser: parse_pict() extracts VML shapes (v:rect, v:shape, v:oval, etc.) with dimensions from CSS style, fill/stroke colors, raw XML for round-trip. DOCX writer: outputs Drawing nodes as raw VML in w:r wrapper. 4 new tests. | crates/s1-model/src/node.rs, attributes.rs, crates/s1-format-docx/src/content_parser.rs, content_writer.rs |
| 2026-03-14 | PDF/A Compliance — PdfAConformance enum (PdfA1b). write_pdf_a() function with ICC sRGB output intent and XMP metadata stream. Minimal sRGB ICC profile generation. XMP with pdfaid:part/conformance, dc:title/creator, xmp:CreatorTool. Facade: Document::export_pdf_a(). 2 new tests. | crates/s1-format-pdf/src/writer.rs, lib.rs, crates/s1engine/src/document.rs |
| 2026-03-14 | Editor: ARIA accessibility (toolbar roles, aria-pressed, textbox role, status role), toolbar features (print, clear formatting, indent/outdent, line spacing, zoom, comments panel, table Tab/Shift+Tab navigation), 19 new Playwright tests (47 total). | editor/index.html, editor/src/toolbar.js, toolbar-handlers.js, input.js, styles.css, state.js |
| 2026-03-14 | Rendering Pipeline Fixes — C-01 to C-08 (all critical), H-01 to H-08 (all high), M-01 to M-06 (all medium) fixed. GlyphRun extended with superscript/subscript/highlight_color/character_spacing/revision_type/revision_author. LayoutTableCell extended with background_color/border per-side. LayoutBlockKind::Paragraph extended with text_align/background_color/border/list_marker/list_level. Style resolver, ShapedRunInfo, cache hash all updated. Paginated HTML: track changes ins/del, superscript/subscript CSS, highlight background, character spacing, table cell styling, paragraph styling, list markers, bookmark Y-positioning. WASM: Section/Body/ColumnBreak rendering. | crates/s1-layout/src/types.rs, engine.rs, html.rs, style_resolver.rs; ffi/wasm/src/lib.rs |
| 2026-03-14 | Editor UI/UX Fixes — E-01 to E-18 fixed (find/replace DOM safety, toolbar debounce, status bar debounce, autosave, beforeunload, auto-recovery, keyboard shortcuts, table modal, URL validation, dark mode, responsive, focus-visible, ARIA, image DPI, table cell focus, panel transitions, enter-in-replace, dirty flag). | editor/src/*.js, editor/src/styles.css |
| 2026-03-14 | Real-World Document Testing — testdocs/ directory with 16 sample documents (DOCX, ODT, TXT, MD, DOC, PDF). Integration tests in crates/s1engine/tests/real_world.rs covering open-without-panic, content verification, cross-format export, round-trip preservation, format auto-detection, performance. | testdocs/*, crates/s1engine/tests/real_world.rs |
| 2026-03-14 | CRIT-03: .unwrap() Elimination — 65 instances removed from library code across 8 files (tree.rs, operation.rs, reader.rs, content_writer.rs, engine.rs, tree_crdt.rs, lib.rs, builder.rs). Only 1 documented .expect() remains (root node invariant). Techniques: if-let, ok_or, unwrap_or_default, let-else. | crates/s1-model/src/tree.rs, crates/s1-ops/src/operation.rs, crates/s1-format-txt/src/reader.rs, crates/s1-format-odt/src/content_writer.rs, crates/s1-layout/src/engine.rs, crates/s1-crdt/src/tree_crdt.rs, ffi/c/src/lib.rs, crates/s1engine/src/builder.rs |
| 2026-03-14 | L-01/L-02/L-03 Code Quality Fixes — L-01: documented intentional HTML escape duplication with sync comments. L-02: extracted shared render_field_html() helper, deduplicating field rendering. L-03: added semantic CSS classes (s1-bold, s1-italic, s1-underline, s1-strikethrough, s1-sup, s1-sub, s1-highlight, s1-ins, s1-del) alongside inline styles in paginated HTML. | ffi/wasm/src/lib.rs, crates/s1-layout/src/html.rs |
| 2026-03-14 | E-20: Version History — IndexedDB version snapshots (every 5 min + manual save), history panel with restore, 20-version cap, auto-prune. | editor/src/file.js, toolbar-handlers.js, index.html, styles.css |
| 2026-03-14 | E-21: Style Gallery — Visual paragraph styles dropdown (Normal, Title, Subtitle, H1-H4, Quote, Code) with formatting previews, applies heading level + font/size/color via WASM. Replaced #blockType select. | editor/src/toolbar-handlers.js, toolbar.js, index.html, styles.css |
| 2026-03-14 | E-22: Page Ruler — Horizontal inch ruler reads actual page dimensions/margins from WASM get_sections_json(). Tick marks, margin indicators, zoom scaling. Works with DOCX/ODT/DOC section properties. | editor/src/ruler.js (new), file.js, main.js, toolbar-handlers.js, index.html, styles.css |
| 2026-03-14 | E-23: Spell Check — Browser native spellcheck toggle button, lang="en" attribute on contentEditable. | editor/index.html, toolbar-handlers.js |
| 2026-03-14 | E-24: Slash Command Menu — "/" at paragraph start shows 11-command palette (headings, lists, table, image, HR, page break, quote, code), filtered as-you-type, keyboard navigation. | editor/src/input.js, index.html, state.js, styles.css |
| 2026-03-14 | E-25: Individual Track Changes — WASM accept_change()/reject_change()/tracked_changes_json() bindings. Click tracked change shows Accept/Reject popup. data-tc-node-id attributes on ins/del elements. | ffi/wasm/src/lib.rs, editor/src/render.js, file.js, styles.css |
| 2026-03-14 | E-26: Comment Threading — Reply button on comments, inline reply form, threaded display with indented replies, reply deletion. In-memory reply storage. | editor/src/toolbar-handlers.js, state.js, styles.css |
| 2026-03-14 | E-27: Virtual Scrolling — IntersectionObserver-based for 100+ block documents, placeholder divs preserve height, 20-block buffer, teardown on re-render. | editor/src/render.js, styles.css |
| 2026-03-14 | E-22 Enhancement: Interactive Ruler — Draggable indent handles (left indent, first-line indent, right indent) with live preview, quarter-inch snapping, applies changes via WASM set_indent(). Updates on selection change. | editor/src/ruler.js, ffi/wasm/src/lib.rs, editor/src/styles.css |
| 2026-03-14 | WASM: set_indent() method — paragraph-level indent (left/right/firstLine) via SetAttributes operation. parse_format_kv() extended with indentLeft/indentRight/indentFirstLine/hyperlinkUrl keys. | ffi/wasm/src/lib.rs |
| 2026-03-14 | E-19: Real-time Collaboration — Node.js WebSocket relay server (scripts/relay.js) with room-based sessions, op forwarding, awareness broadcast, offline buffer, reconnection. Editor collaboration module (editor/src/collab.js) with WasmCollabDocument integration, peer cursors, Share button, connection status, auto-join via URL params. 10 new collaboration WASM tests. | scripts/relay.js, editor/src/collab.js, editor/src/main.js, editor/src/toolbar-handlers.js, editor/src/state.js, editor/index.html, editor/src/styles.css, ffi/wasm/src/lib.rs |

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
- [x] `read_tables` — Basic table structure (Phase 2)
- [x] `read_merged_cells` — Column span, row span (Phase 2)
- [x] `read_images_inline` — Inline images from word/media/ (Phase 2)
- [ ] `read_images_floating` — Floating/anchored images (Phase 2)
- [x] `read_lists_bulleted` — Bulleted lists from numbering.xml (Phase 2)
- [x] `read_lists_numbered` — Numbered lists (Phase 2)
- [x] `read_lists_multilevel` — Multi-level nested lists (Phase 2)
- [x] `read_headers_footers` — Header/footer XML files (Phase 2)
- [x] `read_sections` — Multiple sections with different page sizes (Phase 2)
- [x] `read_hyperlinks` — Hyperlink elements (Phase 2)
- [x] `read_bookmarks` — Bookmark start/end (Phase 2)
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
- [x] `roundtrip_tables` — Round-trip tables (Phase 2)
- [x] `roundtrip_images` — Round-trip images (Phase 2)
- [x] `roundtrip_section_properties` — Round-trip section page layout (Phase 2)
- [x] `roundtrip_header_footer` — Round-trip header/footer content (Phase 2)
- [x] `roundtrip_first_page_header` — Round-trip first-page header with title_page (Phase 2)
- [x] `roundtrip_section_break` — Round-trip multi-section with continuous break (Phase 2)
- [x] `read_hyperlink_external` — External hyperlink with rId resolution (Phase 2)
- [x] `read_hyperlink_internal` — Internal anchor hyperlink (Phase 2)
- [x] `read_hyperlink_tooltip` — Hyperlink with tooltip (Phase 2)
- [x] `read_hyperlink_multiple_runs` — Multiple runs in one hyperlink (Phase 2)
- [x] `read_bookmark_start_end` — BookmarkStart/BookmarkEnd parsing (Phase 2)
- [x] `read_tab_stops` — Tab stop parsing (left/center/right/decimal with leaders) (Phase 2)
- [x] `read_paragraph_borders` — Paragraph border parsing (Phase 2)
- [x] `read_paragraph_shading` — Paragraph shading/background (Phase 2)
- [x] `read_character_spacing` — Character spacing in run properties (Phase 2)
- [x] `read_superscript` — Superscript via vertAlign (Phase 2)
- [x] `read_subscript` — Subscript via vertAlign (Phase 2)
- [x] `write_hyperlink_external` — External hyperlink with relationship (Phase 2)
- [x] `write_hyperlink_internal_anchor` — Internal anchor hyperlink (Phase 2)
- [x] `write_hyperlink_groups_runs` — Consecutive runs grouped under hyperlink (Phase 2)
- [x] `write_bookmark_start_end` — BookmarkStart/BookmarkEnd XML (Phase 2)
- [x] `write_tab_stops` — Tab stop XML generation (Phase 2)
- [x] `write_paragraph_borders` — Paragraph border XML (Phase 2)
- [x] `write_paragraph_shading` — Paragraph shading XML (Phase 2)
- [x] `write_character_spacing` — Character spacing in run properties (Phase 2)
- [x] `roundtrip_hyperlink_external` — Round-trip external hyperlink (Phase 2)
- [x] `roundtrip_hyperlink_internal` — Round-trip internal anchor hyperlink (Phase 2)
- [x] `roundtrip_bookmarks` — Round-trip bookmarks (Phase 2)
- [x] `roundtrip_tab_stops` — Round-trip tab stops (Phase 2)
- [x] `roundtrip_paragraph_borders` — Round-trip paragraph borders (Phase 2)
- [x] `roundtrip_paragraph_shading` — Round-trip paragraph shading (Phase 2)
- [x] `roundtrip_character_spacing` — Round-trip character spacing (Phase 2)
- [x] `roundtrip_superscript_subscript` — Round-trip superscript/subscript (Phase 2)
- [x] `parse_comment_range` — CommentRangeStart/End parsing (Phase 2)
- [x] `write_comment_range` — CommentRangeStart/End XML output (Phase 2)
- [x] `parse_single_comment` — Parse single comment from comments.xml (Phase 2)
- [x] `parse_multiple_comments` — Parse multiple comments (Phase 2)
- [x] `parse_comment_multiple_paragraphs` — Comment with multiple paragraphs (Phase 2)
- [x] `parse_empty_comments` — Empty comments.xml (Phase 2)
- [x] `write_single_comment` — Write comments.xml (Phase 2)
- [x] `write_no_comments_returns_none` — No comments → no file (Phase 2)
- [x] `write_comment_with_date` — Comment with date attribute (Phase 2)
- [x] `roundtrip_comments` — Full comment round-trip (Phase 2)
- [x] `parse_ins_basic` — Parse w:ins with single run (Track Changes)
- [x] `parse_ins_with_author_date` — Parse w:ins with author/date attributes (Track Changes)
- [x] `parse_del_basic` — Parse w:del with single run (Track Changes)
- [x] `parse_del_text_content` — Parse w:delText content in deleted runs (Track Changes)
- [x] `parse_rpr_change` — Parse w:rPrChange format change tracking (Track Changes)
- [x] `parse_mixed_tracked_untracked` — Mixed tracked and untracked paragraphs (Track Changes)
- [x] `parse_ins_multiple_runs` — Multiple runs within single w:ins (Track Changes)
- [x] `parse_nested_ins_del` — Nested w:ins and w:del in same paragraph (Track Changes)
- [x] `write_ins_basic` — Write w:ins wrapper around inserted runs (Track Changes)
- [x] `write_del_basic` — Write w:del with w:delText (Track Changes)
- [x] `write_rpr_change` — Write w:rPrChange in run properties (Track Changes)
- [x] `roundtrip_ins` — Round-trip inserted text track change (Track Changes)
- [x] `roundtrip_del` — Round-trip deleted text track change (Track Changes)
- [x] `roundtrip_mixed_tracked` — Round-trip mixed tracked/untracked content (Track Changes)
- [ ] `fuzz_reader` — Fuzz DOCX reader with random ZIP/XML input

#### s1-format-odt (Phase 2)
- [x] `read_minimal` — Minimal valid ODT (reader.rs)
- [x] `read_multiple_paragraphs` — Multiple paragraphs (reader.rs)
- [x] `read_invalid_zip` — Invalid input produces error (reader.rs)
- [x] `read_missing_content_xml` — Missing content.xml produces error (reader.rs)
- [x] `parse_paragraph_basic` — Basic paragraph parsing (content_parser.rs)
- [x] `parse_paragraph_with_spans` — Spans with auto-style formatting (content_parser.rs)
- [x] `parse_heading` — Heading elements (content_parser.rs)
- [x] `parse_table` — ODF table structure (content_parser.rs)
- [x] `parse_list` — ODF list structures (content_parser.rs)
- [x] `parse_frame_image` — Images in draw:frame (content_parser.rs)
- [x] `parse_line_break` — Line breaks (content_parser.rs)
- [x] `parse_tab` — Tab characters (content_parser.rs)
- [x] `write_minimal_odt` — Write minimal valid ODT ZIP (writer.rs)
- [x] `write_with_content` — Write paragraphs (writer.rs)
- [x] `write_with_styles` — Write styles.xml (writer.rs)
- [x] `write_with_metadata` — Write meta.xml (writer.rs)
- [x] `roundtrip_basic` — Read → write → read text preserved (writer.rs)
- [x] `roundtrip_metadata` — Round-trip title + creator (writer.rs)
- [x] `roundtrip_styles` — Round-trip style definitions (writer.rs)
- [x] `write_content_empty` — Empty document content.xml (content_writer.rs)
- [x] `write_content_paragraphs` — Paragraphs with text (content_writer.rs)
- [x] `write_content_formatted` — Bold/italic auto-styles (content_writer.rs)
- [x] `write_content_table` — Table structure (content_writer.rs)
- [x] `write_content_list` — List reconstruction (content_writer.rs)
- [x] `write_no_styles` — No styles returns None (style_writer.rs)
- [x] `write_paragraph_style` — Paragraph style output (style_writer.rs)
- [x] `write_style_with_parent` — Style with parent reference (style_writer.rs)
- [x] `write_character_style` — Character style output (style_writer.rs)
- [x] `parse_named_style_paragraph` — Named paragraph style parsing (style_parser.rs)
- [x] `parse_style_with_parent` — Style with parent inheritance (style_parser.rs)
- [x] `parse_auto_styles` — Automatic style parsing (style_parser.rs)
- [x] `parse_empty_style_element` — Self-closing style elements (style_parser.rs)
- [x] `write_manifest_basic` — Manifest with standard entries (manifest_writer.rs)
- [x] `write_manifest_with_images` — Manifest with image entries (manifest_writer.rs)
- [x] `parse_basic_metadata` — Title, creator, description (metadata_parser.rs)
- [x] `parse_empty_metadata` — Empty/missing metadata fields (metadata_parser.rs)
- [x] `parse_keywords` — Multiple keyword elements (metadata_parser.rs)
- [x] `write_meta_basic` — Meta.xml with all fields (metadata_writer.rs)
- [x] `write_meta_empty` — No metadata returns None (metadata_writer.rs)
- [x] `parse_bold_italic` — Bold/italic text properties (property_parser.rs)
- [x] `parse_font_size` — Font size parsing (property_parser.rs)
- [x] `parse_font_name` — Font name parsing (property_parser.rs)
- [x] `parse_color` — Color attribute parsing (property_parser.rs)
- [x] `parse_underline` — Underline style mapping (property_parser.rs)
- [x] `parse_paragraph_alignment` — Text alignment (property_parser.rs)
- [x] `parse_paragraph_margins` — Margin/indent parsing (property_parser.rs)
- [x] `write_text_bold_italic` — Bold/italic output (property_writer.rs)
- [x] `write_text_font_size` — Font size output (property_writer.rs)
- [x] `write_text_color` — Color output (property_writer.rs)
- [x] `write_paragraph_alignment` — Alignment output (property_writer.rs)
- [x] `write_paragraph_margins` — Margin output (property_writer.rs)
- [x] `write_table_cell_background` — Cell background output (property_writer.rs)
- [x] `write_table_cell_vertical_align` — Vertical alignment output (property_writer.rs)
- [x] `parse_length_inches/cm/mm/pt/px` — Unit conversion (xml_util.rs)
- [x] `parse_length_invalid` — Invalid length handling (xml_util.rs)
- [x] `points_to_cm_roundtrip` — Points to cm conversion (xml_util.rs)
- [x] `test_parse_percentage` — Percentage parsing (xml_util.rs)
- [ ] `write_opens_in_libreoffice` — Output opens in LibreOffice
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
- [x] `build_simple_table` — Table builder with rows and cells
- [x] `build_table_with_rich_cells` — Table with formatted cell content
- [x] `build_table_mixed_with_paragraphs` — Tables between paragraphs
- [x] `build_table_docx_roundtrip` — Table builder → DOCX → reopen round-trip
- [x] `build_with_section` — Section builder with custom properties
- [x] `build_with_header_footer` — Section builder with header/footer text
- [x] `build_section_docx_roundtrip` — Section builder → DOCX → reopen round-trip
- [x] `open_and_export_odt` — Open ODT bytes, export, round-trip verify
- [x] `odt_builder_roundtrip` — Builder → ODT → reopen round-trip
- [x] `build_with_superscript` — Superscript builder
- [x] `build_with_subscript` — Subscript builder
- [x] `build_with_hyperlink` — Hyperlink builder
- [x] `build_with_bookmark` — Bookmark start/end builder
- [x] `build_hyperlink_docx_roundtrip` — Hyperlink builder → DOCX → reopen round-trip

#### s1-convert (Phase 3)
- [x] `is_doc_file_magic_bytes` — OLE2 magic byte detection
- [x] `is_doc_file_too_short` — Short input rejected
- [x] `is_doc_file_wrong_magic` — Non-DOC magic rejected
- [x] `read_doc_invalid_data` — Invalid DOC input produces error
- [x] `extract_text_heuristic_basic` — Heuristic text extraction from binary stream
- [x] `extract_text_heuristic_filters_short_runs` — Short text runs filtered out
- [x] `extract_text_heuristic_empty` — Empty/binary-only input returns empty
- [x] `extract_text_heuristic_tabs` — Tab characters preserved
- [x] `detect_doc_format` — OLE2 magic → SourceFormat::Doc
- [x] `detect_zip_format` — ZIP magic → SourceFormat::Docx
- [x] `detect_unknown_format` — Unknown bytes → None
- [x] `convert_docx_to_odt` — DOCX → DocumentModel → ODT round-trip
- [x] `convert_odt_to_docx` — ODT → DocumentModel → DOCX round-trip
- [x] `convert_invalid_doc` — Invalid DOC data produces error
- [x] `convert_to_model_docx` — DOCX → DocumentModel extraction

#### s1-crdt (Phase 4)

**Unit tests (138 tests across 12 modules):**
- [x] `lamport_tick` — LamportClock tick increments (clock.rs)
- [x] `lamport_update` — LamportClock update from remote timestamp (clock.rs)
- [x] `vector_clock_merge` — VectorClock merge from two replicas (clock.rs)
- [x] `vector_clock_dominates` — VectorClock dominance comparison (clock.rs)
- [x] `vector_clock_concurrent` — Concurrent vector clock detection (clock.rs)
- [x] `op_id_ordering` — OpId total ordering (lamport first, replica tiebreak) (op_id.rs)
- [x] `op_id_equality` — OpId equality comparison (op_id.rs)
- [x] `state_vector_includes` — StateVector includes check (state_vector.rs)
- [x] `state_vector_diff` — StateVector diff for sync (state_vector.rs)
- [x] `state_vector_merge` — StateVector merge (state_vector.rs)
- [x] `crdt_op_creation` — CrdtOperation construction (crdt_op.rs)
- [x] `crdt_op_with_origins` — CrdtOperation with origin_left/right (crdt_op.rs)
- [x] `crdt_error_variants` — All CrdtError variants (error.rs)
- [x] `text_crdt_insert` — TextCrdt single insert (text_crdt.rs)
- [x] `text_crdt_multiple_inserts` — TextCrdt sequential inserts (text_crdt.rs)
- [x] `text_crdt_concurrent_inserts` — Concurrent inserts converge (text_crdt.rs)
- [x] `text_crdt_delete` — TextCrdt tombstone delete (text_crdt.rs)
- [x] `text_crdt_materialize` — Materialize visible text (text_crdt.rs)
- [x] `text_crdt_offset_to_op_id` — Offset to OpId mapping (text_crdt.rs)
- [x] `tree_crdt_insert` — TreeCrdt insert node (tree_crdt.rs)
- [x] `tree_crdt_delete` — TreeCrdt tombstone delete (tree_crdt.rs)
- [x] `tree_crdt_move` — TreeCrdt move node (tree_crdt.rs)
- [x] `tree_crdt_cycle_detection` — Move creating cycle is dropped (tree_crdt.rs)
- [x] `tree_crdt_concurrent_moves_lww` — Concurrent moves LWW (tree_crdt.rs)
- [x] `tree_crdt_visible_children` — Only non-tombstoned children (tree_crdt.rs)
- [x] `attr_crdt_set` — AttrCrdt set attribute (attr_crdt.rs)
- [x] `attr_crdt_concurrent_different_keys` — Both apply (attr_crdt.rs)
- [x] `attr_crdt_concurrent_same_key_lww` — Highest OpId wins (attr_crdt.rs)
- [x] `attr_crdt_remove` — AttrCrdt remove attribute (attr_crdt.rs)
- [x] `metadata_crdt_set` — MetadataCrdt LWW set (metadata_crdt.rs)
- [x] `metadata_crdt_concurrent_lww` — Concurrent metadata LWW (metadata_crdt.rs)
- [x] `resolver_integrate_insert_node` — Resolver delegates InsertNode (resolver.rs)
- [x] `resolver_integrate_delete_node` — Resolver delegates DeleteNode (resolver.rs)
- [x] `resolver_integrate_insert_text` — Per-character text integration (resolver.rs)
- [x] `resolver_integrate_set_attributes` — Attribute integration (resolver.rs)
- [x] `resolver_duplicate_op` — Duplicate op returns empty (resolver.rs)
- [x] `tombstone_add_remove` — TombstoneTracker add/check/gc (tombstone.rs)
- [x] `collab_apply_local` — CollabDocument apply local operation (collab.rs)
- [x] `collab_apply_remote` — CollabDocument apply remote operation (collab.rs)
- [x] `collab_fork` — Fork replica without phantom state (collab.rs)
- [x] `collab_snapshot` — Snapshot and restore with resolver (collab.rs)
- [x] `collab_changes_since` — Delta sync via state vector (collab.rs)
- [x] `collab_undo_redo` — Local undo/redo generates broadcast ops (collab.rs)
- [x] `collab_causal_ordering` — Out-of-order ops buffered in pending (collab.rs)
- [x] `awareness_set_cursor` — Set and query cursor state (awareness.rs)
- [x] `awareness_remove_stale` — Remove stale cursors (awareness.rs)
- [x] `serialize_operation_roundtrip` — Binary encode/decode CrdtOperation (serialize.rs)
- [x] `serialize_state_vector_roundtrip` — Binary encode/decode StateVector (serialize.rs)
- [x] `serialize_snapshot_roundtrip` — Binary encode/decode Snapshot (serialize.rs)
- [x] `compress_consecutive_inserts` — Merge consecutive single-char inserts (compression.rs)
- [x] `compress_different_replicas` — Don't merge across replicas (compression.rs)

**Convergence integration tests (16 tests in convergence.rs):**
- [x] `two_replicas_concurrent_insert_nodes` — Both insert nodes, sync, same tree
- [x] `two_replicas_concurrent_text_insert_converge` — Concurrent text inserts converge
- [x] `two_replicas_sequential_typing_converge` — Sequential char-by-char typing
- [x] `two_replicas_delete_while_other_inserts` — Delete + insert concurrent
- [x] `two_replicas_concurrent_metadata_lww` — Metadata LWW convergence
- [x] `two_replicas_concurrent_attributes_different_keys` — Both attrs apply
- [x] `three_replicas_converge_after_sync` — 3-way sync convergence
- [x] `three_replicas_mixed_operations_converge` — Mixed ops 3-way
- [x] `delayed_delivery_converges` — Delayed/reordered delivery
- [x] `partition_and_heal` — Network partition then heal
- [x] `snapshot_sync_new_replica` — Snapshot-based initial sync
- [x] `fork_diverge_and_converge` — Fork, diverge, re-sync
- [x] `changes_since_incremental_sync` — Incremental delta sync
- [x] `idempotent_sync` — Double-apply is idempotent
- [x] `five_replicas_all_insert_converge` — 5-replica convergence
- [x] `duplicate_operations_are_idempotent` — Duplicate ops ignored

**Scenario integration tests (17 tests in scenarios.rs):**
- [x] `concurrent_insert_at_same_offset_both_preserved` — Both inserts preserved
- [x] `concurrent_insert_deterministic_order` — Deterministic ordering across replicas
- [x] `concurrent_bold_and_italic_both_apply` — Different attrs both apply
- [x] `concurrent_same_attribute_lww` — Same attr LWW
- [x] `delete_node_while_other_modifies_it` — Delete wins over modify
- [x] `concurrent_delete_same_node` — Double-delete is safe
- [x] `concurrent_metadata_different_keys` — Different metadata keys both apply
- [x] `concurrent_style_updates_lww` — Style LWW
- [x] `undo_only_affects_local_operations` — Undo is local-only
- [x] `batch_operations_all_arrive` — Batch ops all sync
- [x] `op_log_tracks_all_operations` — Op log completeness
- [x] `state_vector_reflects_all_replicas` — State vector accuracy
- [x] `awareness_cursor_sharing` — Cursor presence sharing
- [x] `empty_sync_is_noop` — Empty sync is safe
- [x] `multi_char_insert_syncs_correctly` — Multi-char text sync
- [x] `concurrent_multi_char_inserts_converge` — Concurrent multi-char convergence
- [x] `apply_local_transaction` — Local transaction generates CrdtOps

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
