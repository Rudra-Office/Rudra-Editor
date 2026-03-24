# Phase 4: Editor UX & Feature Parity

## Status: Not Started

## Goal
Close the gap between Rudra Office and production editors (Google Docs, OnlyOffice, Collabora). These are the features users expect from a document editor and will evaluate against.

## Current vs Target

| Area | Current | Target | Gap |
|------|---------|--------|-----|
| Comments & Review | Insert-only, no threading | Reply, resolve, sidebar | Large |
| Track Changes | Toggle exists, accept/reject stubbed | Full sidebar workflow | Large |
| Tables (doc) | Basic insert/delete, no Tab nav | Tab nav, resize, sort | Medium |
| Page Layout | Single header/footer, section breaks stubbed | Per-section H/F, full section breaks | Medium |
| Footnotes | Insert-only | Edit, delete, renumber | Medium |
| Paste | Works but no options | Paste Special dialog | Small |
| PDF | Viewer works, editing unwired | Page ops, forms, signing | Medium |
| Conflict UX | None | "X is editing here" indicators | Small |
| Offline | Buffer exists | Proper merge strategy | Medium |

## Tasks

### Critical (blocks professional use)

**U-01: Comment Threading**
- Reply button on each comment
- Resolve/unresolve toggle
- Sidebar panel with filter (all / open / resolved)
- Click comment to scroll to anchor
- Files: `input.js` (handlers), `toolbar-handlers.js` (panel), `styles.css`
- WASM: `insert_comment_reply()`, `resolve_comment()` needed

**U-02: Track Changes Accept/Reject**
- Sidebar showing all pending changes
- Accept/Reject per change + bulk actions
- Visual diff in document (green insertions, red deletions)
- Navigate between changes
- Files: `toolbar-handlers.js:5604-5885` (existing handlers)
- WASM: `accept_change()`, `reject_change()`, `get_tracked_changes_json()` exist

### High Priority

**U-03: Per-Section Headers/Footers**
- Different first page
- Odd/even pages
- Section-level editing
- WASM section model supports this; editor rendering doesn't use it

**U-04: Tab Navigation in Document Tables**
- Tab → next cell, Shift+Tab → previous
- Tab at last cell creates new row
- File: `input.js` keydown handler, intercept Tab when inside `<td>`

**U-05: Footnote/Endnote Editing**
- Click reference → scroll to footnote area
- Click text → editable region
- Delete reference → removes footnote
- WASM: needs `edit_footnote()`, `delete_footnote()`

**U-06: Paste Special**
- Ctrl+Shift+V opens modal
- Options: Keep formatting, Match destination, Plain text, Values only

**U-13: PDF Editor Wiring**
- Initialize `_wasmPdfEditor = wasm.WasmPdfEditor.open(bytes)` during PDF open
- Gate page ops UI on editor availability
- Free on close

### Medium Priority

**U-07: Table Column Resize** — Drag handles on column borders
**U-08: Table Sort** — Right-click → Sort Ascending/Descending
**U-09: Section Breaks** — Full implementation (next page, continuous, odd/even)
**U-10: Conflict Indicators** — "Alice is typing..." near affected paragraph
**U-11: Import Fidelity Reporting** — "3 charts shown as placeholders" toast
**U-12: Self-Host Fonts** — Bundle NotoSans + MaterialSymbols, remove CDN

## Dependencies

| Task | Depends On |
|------|-----------|
| U-01, U-02 | Phase 1 complete (stable typing) |
| U-03 | Phase 2 S5-01 (WASM fragments for per-section rendering) |
| U-04, U-06, U-13 | None — can start now |
| U-10 | Phase 1 H-01 (range-aware ops for tracking who edits where) |
