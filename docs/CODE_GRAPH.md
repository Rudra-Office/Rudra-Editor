# Code Graph — Rudra Office / s1engine

**Auto-maintained. Update after structural changes.**
**Last updated**: 2026-04-12

---

## Dependency Graph

```
                    ┌─────────────┐
                    │   s1-model   │  4,023 lines │ 76 tests │ ZERO deps
                    │  (core types)│  Nodes, attributes, styles, media
                    └──────┬──────┘
                           │
              ┌────────────┼────────────────────────────────┐
              │            │                                │
        ┌─────┴─────┐ ┌───┴────┐                    ┌──────┴──────┐
        │  s1-ops    │ │s1-text │                    │ Format crates│
        │  2,160 L   │ │2,172 L │                    │ (each depends│
        │  50 tests  │ │65 tests│                    │  only on     │
        │ Operations │ │Shaping │                    │  s1-model)   │
        │ Undo/Redo  │ │Fonts   │                    └──────┬──────┘
        └─────┬──────┘ │Unicode │                           │
              │        └───┬────┘              ┌────────────┼────────────┐
              │            │                   │            │            │
        ┌─────┴────┐  ┌───┴──────┐     ┌──────┴───┐ ┌─────┴────┐ ┌────┴─────┐
        │ s1-crdt  │  │s1-layout │     │s1-format- │ │s1-format-│ │s1-format-│
        │ 5,852 L  │  │11,530 L  │     │  docx     │ │  odt     │ │  pdf     │
        │166 tests │  │83 tests  │     │20,355 L   │ │10,068 L  │ │ 3,701 L  │
        │  CRDT    │  │Pagination│     │296 tests  │ │143 tests │ │ 27 tests │
        │  Fugue   │  │Line break│     └───────────┘ └──────────┘ └──────────┘
        └──────────┘  └──────────┘
                                       ┌──────────┐ ┌──────────┐ ┌──────────┐
                                       │s1-format-│ │s1-format-│ │s1-format-│
                                       │  txt     │ │  md      │ │  html    │
                                       │1,336 L   │ │1,334 L   │ │  408 L   │
                                       │ 41 tests │ │ 32 tests │ │  0 tests │
                                       └──────────┘ └──────────┘ └──────────┘
                                       ┌──────────┐ ┌──────────┐
                                       │s1-format-│ │s1-format-│
                                       │  rtf     │ │  xlsx    │
                                       │  407 L   │ │7,456 L   │
                                       │  0 tests │ │139 tests │
                                       └──────────┘ └──────────┘

              ┌──────────┐
              │s1-convert│  6,480 L │ 152 tests
              │DOC reader│  Depends: s1-model, s1-format-docx, s1-format-odt
              └──────────┘

                    ┌─────────────────┐
                    │    s1engine      │  3,223 lines │ 72 tests
                    │  (facade crate)  │  Public API, re-exports all above
                    └────────┬────────┘
                             │
                ┌────────────┼────────────┐
                │            │            │
          ┌─────┴─────┐ ┌───┴────┐ ┌─────┴─────┐
          │ ffi/wasm   │ │ ffi/c  │ │  server   │
          │ 22,943 L   │ │ 625 L  │ │ 4,798 L   │
          │ WASM API   │ │ C API  │ │ Axum HTTP │
          └────────────┘ └────────┘ └───────────┘
                │
          ┌─────┴──────────────┐
          │    web/ (NEW)       │
          │ OnlyOffice sdkjs    │
          │ + adapter layer     │
          │ + our WASM backend  │
          └────────────────────┘
```

## Crate Details

| Crate | Lines | Tests | Dependencies | Purpose |
|-------|-------|-------|-------------|---------|
| **s1-model** | 4,023 | 76 | **NONE** | Core types: Node, NodeId, attributes, styles, media |
| **s1-ops** | 2,160 | 50 | s1-model | Operations, transactions, undo/redo |
| **s1-text** | 2,172 | 65 | s1-model | Text shaping (rustybuzz), fonts, Unicode, hyphenation |
| **s1-layout** | 11,530 | 83 | s1-model, s1-text | Page layout, Knuth-Plass line breaking, pagination |
| **s1-crdt** | 5,852 | 166 | s1-model, s1-ops | CRDT collaboration (Fugue algorithm) |
| **s1-format-docx** | 20,355 | 296 | s1-model | OOXML DOCX read/write |
| **s1-format-odt** | 10,068 | 143 | s1-model | ODF ODT read/write |
| **s1-format-pdf** | 3,701 | 27 | s1-model, s1-layout, s1-text | PDF export |
| **s1-format-txt** | 1,336 | 41 | s1-model | Plain text read/write |
| **s1-format-md** | 1,334 | 32 | s1-model | Markdown read/write |
| **s1-format-html** | 408 | 0 | s1-model | HTML read/write |
| **s1-format-rtf** | 407 | 0 | s1-model | RTF read/write |
| **s1-format-xlsx** | 7,456 | 139 | (standalone) | XLSX read (partial) |
| **s1-convert** | 6,480 | 152 | s1-model, s1-format-docx, s1-format-odt | Format conversion, legacy DOC reader |
| **s1engine** | 3,223 | 72 | all above (optional features) | Facade: Engine, Document, Format |

## FFI & Server

| Component | Lines | Purpose |
|-----------|-------|---------|
| **ffi/wasm** | 22,943 | WASM bindings via wasm-bindgen. 60+ JS-callable APIs. |
| **ffi/c** | 625 | C FFI bindings via cbindgen. Basic doc I/O. |
| **server** | 4,798 | Axum HTTP API: conversion, collaboration, file management |

## Scripts

| Script | Lines | Purpose |
|--------|-------|---------|
| **relay.js** | 1,143 | WebSocket relay for real-time collaboration |
| **build-wasm.sh** | 48 | Build WASM bindings (dev/release) |
| **download-fonts.sh** | 129 | Download Google Fonts for layout engine |
| **feature-test.sh** | 522 | Automated feature + fidelity test runner |
| **test.sh** | 30 | Quick test runner |

## Totals

```
Rust engine:     80,505 lines
WASM bindings:   22,943 lines
C bindings:         625 lines
Server:           4,798 lines
Scripts:          2,511 lines
─────────────────────────
Total:          111,382 lines
Tests:            1,589 passing
```

## Integration Point (web/)

```
OnlyOffice sdkjs                    s1engine WASM
──────────────────                  ──────────────
                    Adapter Layer
                    ─────────────
Open document   →   adapter.openDoc(bytes)    →  wasm.open(bytes)
Save document   ←   adapter.saveDoc()         ←  wasm.export('docx')
Insert text     →   adapter.insertText(...)   →  wasm.canvas_insert_text(...)
Delete          →   adapter.deleteRange(...)  →  wasm.canvas_delete_range(...)
Format          →   adapter.toggleBold(...)   →  wasm.canvas_toggle_mark(...)
Undo            →   adapter.undo()            →  wasm.undo()
Collaborate     ↔   adapter.applyRemoteOp()   ↔  wasm.apply_crdt_op(...)
Layout          ←   adapter.getLayout()       ←  wasm.to_layout_json()
Cursor          ←   adapter.getCaretRect()    ←  wasm.caret_rect(pos)
```
