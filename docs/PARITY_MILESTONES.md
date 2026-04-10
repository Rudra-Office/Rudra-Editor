# OnlyOffice Parity — Milestones & Tracker

**Reference**: `docs/ONLYOFFICE_COMPARISON.md`
**Created**: 2026-04-08
**Goal**: Close critical gaps between Rudra Office and OnlyOffice for production readiness.

---

## Milestone Overview

```
Phase 13: Parity Sprint 1 — Rendering & Core UX    ████████████████████  COMPLETE
Phase 14: Parity Sprint 2 — Layout & Objects        ████████████████████  COMPLETE
Phase 15: Parity Sprint 3 — Format & Polish         ██████████████████░░  IN PROGRESS
                                                    88/113 items done (78%)
```

---

## Phase 13: Parity Sprint 1 — Rendering & Core UX

**Goal**: Canvas-first rendering, spell check, track changes UX, and key editing gaps.

### M13.1: Canvas as Default Rendering Mode
- [x] Make canvas the default rendering mode (DOM as fallback for a11y) — already defaults ON
- [x] Font-aware canvas rendering — engine-computed run widths used for underline/highlight/strikethrough
- [x] Glyph cache — LRU cache (5000 entries) for canvas text measurements
- [x] Separate overlay canvas for cursor/selection (avoid full page redraws)
- [x] Cross-platform font metric normalization — engine widths + scale correction in hit testing
- [x] Hidden DOM layer — visually hidden div mirrors document text for screen readers
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
- [x] Dictionary lazy loading — worker fetches dictionary on first init, not at page load
- [x] Spell check toggle button in toolbar with localStorage persistence

### M13.3: Track Changes — Full UX
- [x] Display modes: Markup / Final / Original (CSS class-based + dropdown)
- [x] Visual marks: underline for insertions, strikethrough for deletions (per-reviewer color)
- [x] Move tracking: double-underline (move-to), double-strikethrough (move-from) — CSS rules
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
- [x] Grammar check — AI-powered via AI panel grammar mode, Tools menu entry
- [x] Keyboard shortcut reference dialog — comprehensive shortcut listing (full customization deferred)

---

## Phase 14: Parity Sprint 2 — Layout & Objects

**Goal**: Text wrapping, floating objects, advanced tables, shapes, and references.

### M14.1: Floating Object Positioning
- [x] Wrap mode API: set_image_wrap_mode + get_image_wrap_mode (7 modes) — already exists
- [x] Layout engine: wrap type mapping from editor names (wrapLeft/wrapBoth → Square) — fixed
- [x] Editor: context menu with wrap mode selection — already exists
- [x] Anchor types — relativeFrom parsed from DOCX (column/margin/page/character/paragraph/line)
- [x] HorizontalPosition — ImageHorizontalOffset + ImageHorizontalRelativeFrom attributes
- [x] VerticalPosition — ImageVerticalOffset + ImageVerticalRelativeFrom attributes
- [x] Z-order management — behind/inFront maps to WrapType::None in layout, z-index in editor shapes
- [x] Distance-from-text parameters — parsed from ImageDistanceFromText, used in exclusion_rect()
- [x] DOCX round-trip: wp:anchor positioning read (positionH/V, distT/B/L/R) + write back
- [x] Editor UI: position dropdown — image context menu with 7 wrap modes (Inline/WrapLeft/Right/Both/TopBottom/Behind/InFront)
- [x] Drag-to-reposition — shapes support drag-move, images support drag-reorder (full canvas drag deferred)

### M14.2: Text Wrapping Engine (s1-layout)
- [x] Float collection: page_floats Vec with FloatingImageRect per page
- [x] Square wrapping: adjust_rect_for_floats narrows content rect around floats
- [x] Tight/Through wrapping: same exclusion logic as Square (simplified)
- [x] Top-and-Bottom wrapping: advance Y past float
- [x] Integration with paragraph line-breaking: content_rect narrowed before layout
- [x] Wrap side logic — bothSides, left, right, largest via ImageWrapSide attribute in layout engine
- [x] Interval merging — gaps < 4.5pt between floats trigger push-below-all-floats behavior
- [ ] Wrap polygon calculation from shape geometry (scanline algorithm)
- [ ] Performance: cache wrap intervals per page, invalidate on object move

### M14.3: Advanced Tables
- [x] Table auto-fit: auto-fit to content, auto-fit to window, fixed width — already implemented (Auto/Fixed/Percent + proportional distribution)
- [x] Header row repeat on every page — already implemented and tested
- [x] Table styles gallery (8 predefined styles via apply_table_style WASM API + context menu)
- [x] Cell vertical alignment (top/center/bottom) rendered in layout — already implemented
- [x] Cell text direction — CSS writing-mode (btLr, tbRl) in HTML cell rendering
- [x] Table formulas — SUM, AVERAGE, COUNT, MIN, MAX, PRODUCT with ABOVE/BELOW/LEFT/RIGHT directions
- [x] Text-to-table — text_to_table WASM API (tab/comma/semicolon/paragraph delimiters)
- [x] Table captions with auto-numbering — via insert_caption WASM API (label=Table)
- [ ] Draw table tool (pencil for freeform row/column creation)
- [x] Table sorting — sort_table_by_column WASM API + Sort A-Z / Z-A context menu buttons

### M14.4: Shape & Drawing Expansion
- [x] Persist shapes to document model — insert_shape/update_shape WASM APIs, editor saves on creation
- [x] 33 autoshape presets (basic, flowchart, lines/arrows, callouts, stars/banners)
- [x] Shape effects — CSS shadow, heavy shadow, glow, reflection classes
- [x] Shape text body — contentEditable text in textbox, callout, calloutRect shapes
- [x] DOCX round-trip for DrawingML shapes — raw XML preservation + VML textbox generation
- [x] Shape grouping — groupSelected/ungroupSelected with groupId tracking, context menu
- [x] Connector lines — SVG arrow lines between shapes with auto-update on move
- [x] Shape z-order UI controls — bringToFront, sendToBack, bringForward, sendBackward exports

### M14.5: References & Navigation
- [x] Cross-references: WASM API (get_reference_targets_json, insert_cross_reference) + editor dialog
- [x] Captions: insert_caption WASM API (auto-numbered Figure/Table/Equation) + image caption integration
- [x] Table of Figures — insert_table_of_figures WASM API, generates from Caption-styled paragraphs
- [x] TOC refresh/update on demand — already implemented (update_table_of_contents WASM API + UI button)
- [x] SEQ fields — insert_seq_field WASM API with auto-numbering per sequence name
- [x] Footnote popup preview on hover — tooltip popup on mouseover with footnote text
- [ ] Bibliography plugin (Zotero/Mendeley format)

---

## Phase 15: Parity Sprint 3 — Format & Polish

**Goal**: Format coverage, document protection, UI modernization.

### M15.1: DOCX Advanced Elements
- [ ] Chart preservation (read DrawingML chart, render as image, write back)
- [ ] SmartArt preservation (read, render as image fallback, write back)
- [ ] Equation editor (OMML ↔ LaTeX bidirectional conversion)
- [x] Content controls — CheckBox toggle + Dropdown + TextForm interactivity (WASM API + form-controls.js)
- [x] Complex field codes — 40+ field types recognized (PAGE, REF, SEQ, MERGEFIELD, IF, TOC, CITATION, INDEX, etc.)
- [ ] VML shape conversion to DrawingML on read

### M15.2: Format Coverage Expansion
- [ ] RTF reader/writer (s1-format-rtf crate)
- [ ] HTML reader/writer (s1-format-html crate)
- [ ] EPUB export
- [x] PDF/A export — to_pdf_a/to_pdf_a_data_url WASM APIs + File menu entry
- [ ] Legacy DOC full parsing (beyond text extraction)
- [x] Image export — Export Page as PNG via canvas toDataURL, File menu entry

### M15.3: Document Protection & Security
- [ ] Password protection (AES-256 encryption on DOCX)
- [x] Restrict editing modes: read-only, tracked-changes-only, comments-only (modal + mode application)
- [ ] Digital signature creation (certificate-based)
- [ ] Restrict download/print/copy (server-level flags)

### M15.4: Document Comparison
- [ ] Word-level diff algorithm (compare two documents)
- [ ] Display differences as track changes
- [ ] Combine: merge tracked changes from both versions
- [x] Side-by-side view — CSS layout for comparison panes (side-by-side-container)
- [ ] Accept/reject comparison differences

### M15.5: UI Modernization
- [ ] Tabbed ribbon interface (File, Home, Insert, Layout, References, Review, View tabs)
- [x] Context-sensitive tabs — toolbar sections show/hide based on table/image/shape selection
- [x] Dark mode — already fully implemented (toggle, CSS variables, OS preference, localStorage)
- [x] Zoom range expansion (50-500%) — max increased from 200 to 500, added 300/400/500% presets
- [x] Multipage view — CSS flex-wrap, View menu toggle
- [ ] Alt-key ribbon navigation (key tips)
- [x] Print preview matching canvas — converts canvas pages to images for preview

### M15.6: Advanced Editing Features
- [ ] Mail merge (spreadsheet data source, field insertion, preview, PDF/DOCX output)
- [x] Watermarks — text watermarks with preset/custom text, diagonal/horizontal orientation, CSS overlay
- [x] Drop caps — CSS ::first-letter toggle via Format menu, .drop-cap class
- [x] Line numbering — CSS counter-based, Format menu toggle
- [x] Hyphenation toggle in UI — Format menu entry, layout engine always uses Knuth-Liang hyphenation
- [x] Freehand drawing — pen, highlighter, eraser tools with canvas overlay per page
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
