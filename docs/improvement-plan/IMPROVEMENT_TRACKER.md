# Rudra Office — Improvement Pipeline Tracker

Authoritative roadmap for moving from prototype to production-grade.

## Current Maturity: ~82% (Phase 1-2 Complete, Phase 4 Mostly Complete)

Foundation hardening (Phase 1) is mostly done. **Phase 2 (WASM Ownership) is now substantially complete** — the editor uses `get_page_html()` per-page rendering from the WASM layout engine as the primary path. DOM measurement and CSS clipping are deprecated (kept as fallback only). Direct input interception sends keystrokes to WASM without DOM round-trip. Production features (Phase 3-4) are not started.

---

## Phase 1: Foundation Hardening — Mostly Complete

| ID | Task | Status | Evidence |
|---|---|---|---|
| H-01 | Replace `set_paragraph_text` with `replace_text` diff | 🟢 Done | render.js:867 uses `replace_text()` with prefix/suffix diff |
| H-02 | Fix reconnect protocol (`sync-req` → `requestCatchup`) | 🟢 Done | collab.js:698 sends `requestCatchup` with `fromVersion` |
| H-03 | Targeted unicast for catch-up ops | 🟢 Done | collab.rs:621 uses `_target` field, line 369 filters delivery |
| H-04 | Unify `access`/`mode` semantics | 🟢 Done | collab.js:371 sends both params from single `accessLevel` var |
| H-06 | Cursor position preservation across re-renders | 🟢 Done | render.js:131-175 saves/restores cursor via nodeId+charOffset |
| H-07 | Use `renderNodeById` as default for single-paragraph edits | 🟢 Done | input.js has 13 calls to `renderNodeById` for typing/formatting |
| H-08 | Wire `applySplitParagraphClipping` into repaginate | 🟡 Deprecated | pagination.js:441 — now bypassed by WASM page rendering |
| H-09 | Add 15 missing co-editing op handlers | 🟢 Done | collab.js:1240-1750 handles all structural ops |
| H-10 | `normalizeRemoteOp` for field name mismatches | 🟢 Done | collab.js:1001 normalizes 8 action/field variations |
| H-05 | E2E regression suite | 🟡 Partial | 45 Playwright tests exist but NO concurrent editing tests |

### Remaining Gaps

| Problem | Status | Notes |
|---------|--------|-------|
| First-edit cache miss (`syncedTextCache` empty) | 🟢 Fixed | `renderPageFromWasm` pre-populates cache via `getEditableText()` |
| Full `renderDocument()` called ~109 times | 🟡 Mitigated | `renderDocument()` now delegates to WASM page path; individual callers still use it but it's fast |
| `applySplitParagraphClipping` is CSS-based | 🟢 Bypassed | WASM path renders split fragments with `data-split` attributes — no CSS clipping |
| Checksum sent but validation only warns | 🟢 Fixed | collab.js:1752-1756 rejects mismatch + requests retransmission |
| No concurrent editing E2E tests | 🔴 Open | Test suite is single-user only |

---

## Phase 2: WASM Ownership — Substantially Complete

| ID | Task | Status | Notes |
|---|---|---|---|
| W-01 | `get_page_html(page_index)` per-page rendering | 🟢 Done | lib.rs: `get_page_html()`, `get_page_html_with_fonts()` — returns document-model HTML per page |
| W-01b | `get_page_count()` page count API | 🟢 Done | lib.rs: `get_page_count()`, `get_page_count_with_fonts()` |
| W-01c | `get_affected_pages(nodeId)` incremental API | 🟢 Done | lib.rs: returns JSON array of affected + adjacent page indices |
| W-01d | Layout cache with auto-invalidation | 🟢 Done | lib.rs: `RefCell<Option<LayoutDocument>>` cache, invalidated via `doc_mut()` |
| S5-01 | WASM paragraph fragments (`render_node_slice`) | 🟢 Done | Split paragraphs rendered with `render_paragraph_clean_partial` + `data-split` attrs |
| S5-02 | Remove DOM measurements for page break decisions | 🟢 Done (primary path) | WASM layout engine decides page breaks; DOM measurement only in deprecated fallback |
| S5-03 | Reduce `renderDocument()` full re-renders | 🟢 Done | `renderDocument()` delegates to `renderDocumentFromWasm()` — per-page WASM HTML |
| W-02 | Deprecate `domBasedOverflowSplit` | 🟢 Deprecated | Marked deprecated, only used in repaginate() fallback path |
| W-04 | Line-by-line page carry (engine-driven) | 🟢 Done | Layout engine splits paragraphs at line boundaries; fragments rendered per-page |
| W-05 | Direct input → WASM (no DOM read) | 🟢 Done | input.js: `insertText`, `deleteContentBackward/Forward` intercepted → WASM → `renderAffectedPages()` |
| W-06 | Paste via WASM-authority rendering | 🟢 Done | input.js: paste handlers use `renderAffectedPages()` with `renderDocument()` fallback |
| W-07 | Split/merge via WASM-authority rendering | 🟢 Done | input.js: Enter/Backspace at boundary use `renderAffectedPages()` |
| W-08 | Collab ops via `renderAffectedPages` | 🟡 Partial | 6 remote op handlers use `renderAffectedPages()`; rest use `renderDocument()` (which is WASM-authority anyway) |
| W-03 | WASM glyph widths for cursor positioning | 🔴 Not Started | Cursor still uses DOM-measured positions |

### Architecture: Primary vs Fallback Paths

```
PRIMARY PATH (active):
  renderDocument() → renderDocumentFromWasm()
    → get_page_count_with_fonts(fontDb) for total pages
    → get_page_html_with_fonts(i, fontDb) for each page
    → mount HTML directly into .page-content elements
    → no DOM measurement, no CSS clipping

INCREMENTAL PATH (active):
  renderAffectedPages(nodeId)
    → get_affected_pages(nodeId) for page indices
    → renderPageFromWasm(i) for each affected page
    → cursor save/restore

DIRECT INPUT PATH (active):
  beforeinput → e.preventDefault()
    → doc.insert_text_in_paragraph() / doc.replace_text()
    → renderAffectedPages(nodeId)
    → setCursorAtOffset()

FALLBACK PATH (legacy, deprecated):
  renderDocument() → to_html() + repaginate()
    → only used if WASM page API unavailable or throws
    → uses DOM measurement, CSS clipping (deprecated functions)
```

---

## Phase 3: Production Scale — Not Started

| ID | Task | Status | Notes |
|---|---|---|---|
| S-01 | Binary WebSocket protocol for fullSync | 🟢 Done | Client sends Binary frame (header+NUL+bytes); server decodes, stores snapshot, re-encodes base64 for text-only peers |
| S-02 | Structural CRDTs (SplitNode/MergeNode) | 🔴 Not Started | Enter/Backspace still triggers fullSync for convergence |
| S-03 | Server-side authoritative document state | 🔴 Not Started | Server stores peer-provided snapshots |
| S-04 | Snapshot staleness detection | 🔴 Not Started | No `last_snapshot_at` tracking |
| S6-04 | Cache-miss fallback improvement | 🟢 Done | `renderPageFromWasm` pre-populates syncedTextCache with getEditableText |
| S6-05 | AI/Autocorrect use `replace_text` | 🟢 Done | ai-inline.js:453, ai-panel.js:640, input.js:2342 |
| S7-07 | Checksum validation for fullSync | 🟢 Done | Receive-side rejects mismatched checksums and requests retransmission (collab.js:1752) |
| S7-08 | Op log replay for fresh joiners | 🟡 Partial | Server sends snapshot + catch-up ops but freshness not guaranteed |

---

## Phase 4: Editor UX & Feature Parity — Complete

| ID | Task | Priority | Status |
|---|---|---|---|
| U-01 | Comment threading — reply, resolve, edit, delete | Critical | 🟢 Done | Panel with threads, reply/resolve/edit/delete UI + WASM APIs (edit_comment, resolve_comment, delete_comment_reply) |
| U-02 | Track changes — accept/reject with sidebar UI | Critical | 🟢 Done | TC bar + panel, per-change accept/reject, accept/reject all, navigation, set_track_changes_enabled API |
| U-03 | Per-section headers/footers | High | 🟢 Done | WASM renders correct header/footer per section per page |
| U-04 | Tab navigation in document tables | High | 🟢 Done | input.js:1292-1376 — Tab/Shift+Tab, cross-page chunk nav, auto-add row |
| U-05 | Footnote/endnote editing (click-to-edit, delete) | High | 🟢 Done | Click-to-navigate, contentEditable body, auto-numbering |
| U-06 | Paste Special dialog | High | 🟢 Done | input.js: showPasteSpecialDialog with formatted/plain options |
| U-07 | Table column resize with drag handles | Medium | 🟢 Done | input.js: setupTableColumnResize — drag cell borders, commits via set_table_column_widths |
| U-08 | Table sort by column | Medium | 🟢 Done | Context menu sort ascending/descending via WASM sort_table_by_column |
| U-09 | Section breaks (next page, continuous, odd/even) | Medium | 🟢 Done | WASM renders section break indicators; layout engine handles break types |
| U-10 | Conflict indicators ("X is editing this paragraph") | Medium | 🟢 Done | collab.js: peer-cursor labels + peer-editing class on paragraphs with colored border |
| U-11 | Import fidelity reporting | Medium | 🟢 Done | file.js: shows toast on open with count of placeholder objects (charts, SmartArt, OLE, missing images) |
| U-12 | Self-host fonts, remove CDN dependency | Medium | 🟢 Done | KaTeX copied to public/katex/; index.html references local paths; no external CDN calls |
| U-13 | PDF editor wiring (`_wasmPdfEditor` on open) | High | 🟢 Done | 6 PDF modules fully wired: viewer, annotations, forms, signatures, text-edit, pages |

---

## What's Actually Working Well

- Rust engine: DOCX/ODT/PDF/TXT read/write, 1172+ tests passing (226 WASM-specific)
- CRDT: Character-level text sync works reliably
- Layout engine: Per-page HTML rendering with layout cache and auto-invalidation
- **WASM-authority rendering**: `get_page_html()` produces ready-to-mount HTML per page
- **Direct input interception**: Keystrokes go to WASM, then only affected pages re-render
- **Incremental page rendering**: `renderAffectedPages()` re-renders 1-3 pages instead of full document
- `replace_text` diff-based sync: Formatting preservation with pre-populated cache
- Co-editing protocol: reconnect, normalized ops, 15+ structural op handlers
- Virtual scrolling + lazy page rendering for large docs
- 45 Playwright E2E tests (single-user)

## What's Still Broken / Not Started

- WASM glyph widths for cursor positioning (W-03) — cursor uses DOM-measured positions
- fullSync on every Enter/Backspace in collab (S-02: structural CRDTs needed)
- Server-side authoritative document state (S-03)
- Snapshot staleness detection (S-04)
- No concurrent editing E2E tests (H-05)

---

**Legend:** 🔴 Not Started | 🟡 Partial | 🟢 Done | 🔵 Verified (automated tests)
