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

## Maintenance Instructions

### After Every Code Change
1. Run `cargo test` — all tests must pass
2. Run `cargo clippy -- -D warnings` — no warnings
3. Run `cargo fmt --check` — formatting correct

### After Every Milestone Completion
1. Update **Phase Completion Tracker** if phase changed
2. Review and update **Crate Implementation Status** table

### After Every Phase Completion
1. Update **Current Phase** at the top of Project State
2. Add **Phase Completion** date
3. Review all docs for accuracy — architecture may have evolved
4. Update `docs/ROADMAP.md` with actual timelines vs planned
