# Rudra Office — AI Development Context

## What Is This Project?

**Two parts:**

1. **s1engine** — Modular document engine SDK in pure Rust. Reads, writes, edits, converts DOCX/ODT/PDF/TXT/HTML/RTF. CRDT-based collaborative editing. 1,589 tests passing.

2. **Web Editor** — OnlyOffice sdkjs-based web editor with s1engine WASM as the document backend. OnlyOffice handles rendering, input, cursor, canvas. Our WASM handles document model, format conversion, layout, collaboration.

## Current Goal

**Integrate OnlyOffice's web editor (sdkjs) with our s1engine WASM backend.**

OnlyOffice sdkjs is a battle-tested canvas-based document editor (400K+ lines JS). It handles:
- Canvas rendering with pixel-perfect cross-platform consistency
- Hidden textarea input capture (no contentEditable)
- Cursor positioning from document model coordinates
- IME/composition support
- Incremental paragraph recalculation
- Selection, formatting, undo/redo UI

Our s1engine WASM handles:
- Document model (read/write DOCX, ODT, PDF, TXT, HTML, RTF)
- CRDT collaboration (Fugue algorithm)
- Page layout (Knuth-Plass line breaking, pagination)
- Text shaping (rustybuzz)
- Format conversion

The integration approach: **Replace OnlyOffice's C++ backend (x2t/DocumentServer) with our s1engine WASM.** Keep their editor frontend as-is.

## Architecture

```
OnlyOffice sdkjs (JS)          s1engine WASM (Rust)
─────────────────────          ────────────────────
Canvas rendering               Document model
Hidden textarea input          DOCX/ODT/PDF read/write
Cursor management              CRDT collaboration
Selection/formatting UI        Page layout engine
Undo/redo UI                   Text shaping
                               Format conversion
         ↕
    Adapter Layer (JS)
    Translates between sdkjs API calls
    and s1engine WASM API calls
```

## Project Structure

```
crates/
  s1-model/          Core document model (zero dependencies)
  s1-ops/            Operations, transactions, undo/redo
  s1-format-docx/    DOCX reader/writer
  s1-format-odt/     ODT reader/writer
  s1-format-pdf/     PDF export
  s1-format-txt/     Plain text reader/writer
  s1-format-html/    HTML reader/writer
  s1-format-rtf/     RTF reader/writer
  s1-format-xlsx/    XLSX reader (partial)
  s1-format-md/      Markdown reader/writer
  s1-convert/        Format conversion + legacy DOC reader
  s1-layout/         Page layout engine
  s1-text/           Text processing (rustybuzz, fonts, Unicode)
  s1-crdt/           CRDT collaboration algorithms
  s1engine/          Facade crate — public API
ffi/
  wasm/              WASM bindings (wasm-bindgen)
  c/                 C FFI bindings
server/              Axum HTTP API server
web/                 OnlyOffice sdkjs editor (TO BE SET UP)
scripts/
  relay.js           WebSocket relay for collaboration
  build-wasm.sh      WASM build script
```

## Architecture Rules (MUST Follow)

### 1. Document Model is Sacred
- `s1-model` has **ZERO external dependencies**
- Every node has globally unique `NodeId(replica_id, counter)`
- Never expose internal model representation in public API

### 2. All Mutations Via Operations
- ALL changes go through `Operation` → applied via `s1-ops`
- Every `Operation` must implement `invert()` for undo

### 3. Format Isolation
- Each format crate ONLY depends on `s1-model`
- Format crates NEVER depend on each other

### 4. No Panics in Library Code
- ALL public functions return `Result<T, Error>`
- No `.unwrap()` or `.expect()` in library code

### 5. OnlyOffice Integration
- Do NOT modify OnlyOffice sdkjs source unless absolutely necessary
- Write adapter layers that translate between sdkjs API and s1engine WASM API
- Keep OnlyOffice attribution clear in README and LICENSE

## Coding Conventions

### Rust
- `cargo fmt`, `cargo clippy -- -D warnings`
- `snake_case` functions, `PascalCase` types
- Every public function needs tests
- Use `thiserror` for error types

### After Every Code Change
1. `cargo test` — all tests must pass
2. `cargo clippy -- -D warnings` — no warnings
3. `cargo fmt --check` — formatting correct
