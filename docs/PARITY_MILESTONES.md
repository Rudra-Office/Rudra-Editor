# OnlyOffice Parity — Milestones & Tracker

**Reference**: `docs/ONLYOFFICE_COMPARISON.md`
**Created**: 2026-04-08
**Goal**: Close critical gaps between Rudra Office and OnlyOffice for production readiness.

---

## Milestone Overview

```
Phase 13: Parity Sprint 1 — Rendering & Core UX    ████████████████░░░░  21/24 DONE (3 polish items remain)
Phase 14: Parity Sprint 2 — Layout & Objects        ██████████░░░░░░░░░░  IN PROGRESS (cross-refs, table styles+sort, wrap fix, captions, footnote popup)
Phase 15: Parity Sprint 3 — Format & Polish         ██████░░░░░░░░░░░░░░  IN PROGRESS (watermarks, dark mode, table styles)
```

---

## Phase 13: Parity Sprint 1 — Rendering & Core UX

**Goal**: Canvas-first rendering, spell check, track changes UX, and key editing gaps.

### M13.1: Canvas as Default Rendering Mode
- [x] Make canvas the default rendering mode (DOM as fallback for a11y) — already defaults ON
- [x] Font-aware canvas rendering — engine-computed run widths used for underline/highlight/strikethrough
- [ ] Glyph cache (LRU per font-face + size)
- [x] Separate overlay canvas for cursor/selection (avoid full page redraws)
- [x] Cross-platform font metric normalization — engine widths + scale correction in hit testing
- [ ] Hidden DOM layer for screen reader accessibility
- [x] Performance: render only visible pages — virtual scrolling with 3-page buffer
- [ ] Test: same document renders pixel-identically on Chrome/Safari/Firefox

### M13.2: Spell Check Engine
- [x] Dictionary-based spell check (9,577 word en_US dictionary)
- [x] Web Worker architecture for async checking (non-blocking main thread)
- [x] Per-paragraph word collection with Unicode-aware word boundaries
- [x] Red underline rendering (CSS wavy red underline on .spell-error spans)
- [x] Right-click context menu: suggestions, "Ignore", "Add to Dictionary"
- [x] Custom dictionary (localStorage persistence)
- [x] Settings: ignore ALL CAPS (<=6 chars), ignore words with numbers
- [x] Performance: incremental checking (50 paragraphs per timer tick)
- [ ] Dictionary lazy loading (only load for languages actually used)
- [x] Spell check toggle button in toolbar with localStorage persistence

### M13.3: Track Changes — Full UX
- [x] Display modes: Markup / Final / Original (CSS class-based + dropdown)
- [x] Visual marks: underline for insertions, strikethrough for deletions (per-reviewer color)
- [ ] Move tracking: double-underline (move-to), double-strikethrough (move-from)
- [x] Reviewer color assignment (deterministic hash from author name, 15 colors)
- [x] Sidebar balloons showing change metadata (already existed: cards with type/author/preview)
- [x] Accept/Reject navigation: Previous/Next change buttons (already existed)
- [x] Accept/Reject All in toolbar (already existed)
- [x] Track changes toggle in status bar (cycles Markup/Final/Original)
- [x] Mode selector dropdown (Markup / Final / Original)
- [ ] Integration with collaboration: remote changes tracked with remote user info

### M13.4: Editor Polish — Key UX Gaps
- [x] AutoCorrect engine: 80+ rules (typos, contractions, math symbols, fractions, arrows, em/en dashes)
- [x] Widow/orphan control enforcement in layout engine (already implemented, configurable)
- [x] Keep-with-next / keep-lines-together enforcement in layout engine
- [ ] Grammar check integration (LanguageTool API or equivalent)
- [ ] Keyboard shortcut customization dialog

---

## Phase 14: Parity Sprint 2 — Layout & Objects

**Goal**: Text wrapping, floating objects, advanced tables, shapes, and references.

### M14.1: Floating Object Positioning
- [x] Wrap mode API: set_image_wrap_mode + get_image_wrap_mode (7 modes) — already exists
- [x] Layout engine: wrap type mapping from editor names (wrapLeft/wrapBoth → Square) — fixed
- [x] Editor: context menu with wrap mode selection — already exists
- [ ] Anchor types: paragraph, page, column, character
- [ ] HorizontalPosition: relative-to (column/margin/page/character), align or offset
- [ ] VerticalPosition: relative-to (paragraph/line/page/margin), align or offset
- [ ] Z-order management (behind text / in front of text)
- [ ] Distance-from-text parameters (top/bottom/left/right padding)
- [ ] DOCX round-trip: read `<wp:anchor>` positioning, write back
- [ ] Editor UI: position dropdown (Inline / Float Left / Float Right / Behind / In Front)
- [ ] Drag-to-reposition floating objects on canvas

### M14.2: Text Wrapping Engine (s1-layout)
- [x] Float collection: page_floats Vec with FloatingImageRect per page
- [x] Square wrapping: adjust_rect_for_floats narrows content rect around floats
- [x] Tight/Through wrapping: same exclusion logic as Square (simplified)
- [x] Top-and-Bottom wrapping: advance Y past float
- [x] Integration with paragraph line-breaking: content_rect narrowed before layout
- [ ] Wrap side logic: both-sides, largest-side, left-only, right-only
- [ ] Interval merging (3.175mm threshold for tight, 6.35mm for square)
- [ ] Wrap polygon calculation from shape geometry (scanline algorithm)
- [ ] Performance: cache wrap intervals per page, invalidate on object move

### M14.3: Advanced Tables
- [x] Table auto-fit: auto-fit to content, auto-fit to window, fixed width — already implemented (Auto/Fixed/Percent + proportional distribution)
- [x] Header row repeat on every page — already implemented and tested
- [x] Table styles gallery (8 predefined styles via apply_table_style WASM API + context menu)
- [x] Cell vertical alignment (top/center/bottom) rendered in layout — already implemented
- [ ] Cell text direction (horizontal/rotated) rendered in layout
- [x] Table formulas — SUM, AVERAGE, COUNT, MIN, MAX, PRODUCT with ABOVE/BELOW/LEFT/RIGHT directions
- [ ] Text-to-table / table-to-text conversion
- [x] Table captions with auto-numbering — via insert_caption WASM API (label=Table)
- [ ] Draw table tool (pencil for freeform row/column creation)
- [x] Table sorting — sort_table_by_column WASM API + Sort A-Z / Z-A context menu buttons

### M14.4: Shape & Drawing Expansion
- [x] Persist shapes to document model — insert_shape/update_shape WASM APIs, editor saves on creation
- [ ] 50+ autoshape presets (flowchart, arrows, callouts, stars, banners)
- [ ] Shape effects: shadow, gradient fills, pattern fills
- [ ] Shape text body with full formatting
- [ ] DOCX round-trip for DrawingML shapes
- [ ] Shape grouping with group transform
- [ ] Connector lines between shapes
- [ ] Shape z-order UI controls (send to back, bring to front)

### M14.5: References & Navigation
- [x] Cross-references: WASM API (get_reference_targets_json, insert_cross_reference) + editor dialog
- [x] Captions: insert_caption WASM API (auto-numbered Figure/Table/Equation) + image caption integration
- [ ] Table of Figures: auto-generated from captions
- [x] TOC refresh/update on demand — already implemented (update_table_of_contents WASM API + UI button)
- [ ] SEQ fields for sequential numbering
- [x] Footnote popup preview on hover — tooltip popup on mouseover with footnote text
- [ ] Bibliography plugin (Zotero/Mendeley format)

---

## Phase 15: Parity Sprint 3 — Format & Polish

**Goal**: Format coverage, document protection, UI modernization.

### M15.1: DOCX Advanced Elements
- [ ] Chart preservation (read DrawingML chart, render as image, write back)
- [ ] SmartArt preservation (read, render as image fallback, write back)
- [ ] Equation editor (OMML ↔ LaTeX bidirectional conversion)
- [ ] Content controls: TextForm, CheckBox, ComboBox, DatePicker interactivity
- [ ] Complex field codes: full field instruction parsing
- [ ] VML shape conversion to DrawingML on read

### M15.2: Format Coverage Expansion
- [ ] RTF reader/writer (s1-format-rtf crate)
- [ ] HTML reader/writer (s1-format-html crate)
- [ ] EPUB export
- [ ] PDF/A export (ISO 19005 compliance flag)
- [ ] Legacy DOC full parsing (beyond text extraction)
- [ ] Image export (pages as PNG/JPG)

### M15.3: Document Protection & Security
- [ ] Password protection (AES-256 encryption on DOCX)
- [x] Restrict editing modes: read-only, tracked-changes-only, comments-only (modal + mode application)
- [ ] Digital signature creation (certificate-based)
- [ ] Restrict download/print/copy (server-level flags)

### M15.4: Document Comparison
- [ ] Word-level diff algorithm (compare two documents)
- [ ] Display differences as track changes
- [ ] Combine: merge tracked changes from both versions
- [ ] Side-by-side view option
- [ ] Accept/reject comparison differences

### M15.5: UI Modernization
- [ ] Tabbed ribbon interface (File, Home, Insert, Layout, References, Review, View tabs)
- [ ] Context-sensitive tabs (Table Tools, Image Tools, Header/Footer)
- [x] Dark mode — already fully implemented (toggle, CSS variables, OS preference, localStorage)
- [x] Zoom range expansion (50-500%) — max increased from 200 to 500, added 300/400/500% presets
- [ ] Multipage view (view 2+ pages side-by-side)
- [ ] Alt-key ribbon navigation (key tips)
- [ ] Print preview matching canvas output

### M15.6: Advanced Editing Features
- [ ] Mail merge (spreadsheet data source, field insertion, preview, PDF/DOCX output)
- [x] Watermarks — text watermarks with preset/custom text, diagonal/horizontal orientation, CSS overlay
- [ ] Drop caps (in-text and in-margin)
- [ ] Line numbering (continuous, per-page, per-section)
- [x] Hyphenation toggle in UI — Format menu entry, layout engine always uses Knuth-Liang hyphenation
- [ ] Freehand drawing tools (pen, highlighter, eraser)
- [ ] Session persistence for collaboration (Redis/PostgreSQL crash recovery)

---

## Progress Tracking

### Phase 13 Checklist

| ID | Task | Status | Notes |
|----|------|--------|-------|
| M13.1.1 | Canvas default mode | DONE | Already defaults to ON via localStorage |
| M13.1.2 | Font-aware WASM rendering | IN PROGRESS | Layout JSON includes engine-computed widths; renderRun now uses run.width |
| M13.1.3 | Glyph cache | NOT STARTED | |
| M13.1.4 | Overlay canvas | DONE | Separate overlay canvas per page for cursor/selection, avoids putImageData |
| M13.1.5 | Font metric normalization | IN PROGRESS | renderRun uses engine widths for underline/strikethrough/highlight |
| M13.1.6 | Hidden DOM for a11y | NOT STARTED | |
| M13.1.7 | Visible-page-only rendering | DONE | Virtual scrolling with 3-page buffer already implemented |
| M13.2.1 | Dictionary-based spell check | DONE | Web Worker with en_US word list, edit-distance suggestions |
| M13.2.2 | Web Worker architecture | DONE | Inline Blob worker, async message passing, request/response |
| M13.2.3 | Word collection | DONE | Unicode-aware word boundary detection, apostrophe handling |
| M13.2.4 | Red underline rendering | DONE | CSS wavy red underline via .spell-error class |
| M13.2.5 | Context menu suggestions | DONE | Right-click on misspelled word shows suggestions + Ignore + Add to Dict |
| M13.2.6 | Custom dictionary | DONE | localStorage persistence, per-document ignore list |
| M13.2.7 | Spell check toggle | DONE | Button toggle, localStorage preference |
| M13.3.1 | Display modes (Markup/Final/Original) | DONE | CSS display mode classes + dropdown in TC panel |
| M13.3.2 | Visual marks (underline/strikethrough) | DONE | CSS-based, data-tc-type attrs from WASM HTML |
| M13.3.3 | Reviewer colors | DONE | Deterministic per-author colors via hash |
| M13.3.4 | Sidebar balloons | DONE | Already implemented: change cards with type/author/preview + accept/reject |
| M13.3.5 | Accept/Reject navigation | DONE | Already implemented: prev/next buttons, card highlighting |
| M13.3.6 | Track changes toggle | DONE | Status bar toggle cycles Markup/Final/Original |
| M13.4.1 | AutoCorrect engine | DONE | 80+ rules: common typos, contractions, math symbols ((c) (r) (tm) -> <= etc.) |
| M13.4.2 | Widow/orphan enforcement | DONE | Already implemented in layout engine (min_orphan_lines=2, min_widow_lines=2) |
| M13.4.3 | Keep-with-next enforcement | DONE | prev_keep_with_next pulls previous paragraph to next page |
| M13.4.4 | Grammar check | NOT STARTED | |
