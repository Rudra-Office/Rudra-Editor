# Rudra Office vs OnlyOffice — Fine-Grained Comparison

**Date**: 2026-04-08
**Purpose**: Identify every gap in Rudra Office (s1engine + editor) against OnlyOffice as the ideal reference. Prioritized action items for product parity.

---

## Executive Summary

| Area | OnlyOffice | Rudra Office | Gap Severity |
|------|-----------|--------------|-------------|
| **Rendering Engine** | Custom Canvas + WASM FreeType | Canvas default + overlay, engine widths, multipage view | LOW (was CRITICAL) |
| **DOCX Fidelity** | 95%+ round-trip | ~85% (good core, shapes persisted, text-to-table, SEQ fields) | MEDIUM (was HIGH) |
| **Editing UX** | Mature, desktop-grade | Spell check, autocorrect, grammar (AI), track changes, watermarks, drop caps | LOW (was HIGH) |
| **Collaboration** | OT-based, battle-tested | CRDT-based (stronger foundation) | LOW |
| **Format Coverage** | 40+ formats via C++ x2t | 6 formats + image export (DOCX, ODT, PDF, TXT, MD, XLSX, PNG) | MEDIUM |
| **Tables** | Full (draw, merge, formulas, styles) | Merge, auto-fit, header repeat, styles (8), formulas, sorting, text-to-table | LOW (was HIGH) |
| **Track Changes** | Full (accept/reject, compare, 4 display modes) | Full: 3 display modes, per-author colors, move tracking, status bar toggle | LOW (was HIGH) |
| **Spell Check** | Custom Hunspell WASM + red underlines + suggestions | Worker spell check, 9.6K dict, red underlines, suggestions, custom dict, grammar (AI) | LOW (was CRITICAL) |
| **Page Layout** | Pixel-perfect cross-platform | Pagination, keep-with-next, widow/orphan, overflow clip, drop caps, line numbers | LOW (was HIGH) |

---

## 1. RENDERING ENGINE — CRITICAL GAP

### OnlyOffice Approach
- **Pure Canvas rendering** — ALL text drawn via Canvas 2D API
- **WASM-compiled FreeType** — font rasterization in WebAssembly, NOT browser text APIs
- **Pixel-perfect across platforms** — same document looks identical on Windows/Mac/Linux/any browser
- **Custom glyph cache** — LRU font file cache + per-glyph cache + stream cache
- **Overlay system** — separate layers for cursors, selections, highlights
- **Single-page rendering** — only current page drawn to canvas, others in memory

### Rudra Office Current State
- **contentEditable DOM** — browser renders text using native font engine
- **Optional Canvas mode** — `canvas-render.js` exists but is secondary/experimental
- **Browser-dependent rendering** — same document may look different across browsers/OSes
- **Virtual scrolling** — 3-page buffer for performance (good)
- **Layout JSON** — WASM produces layout data, canvas mode can consume it

### What Must Change

| # | Action | Priority | Effort |
|---|--------|----------|--------|
| R-01 | **Make Canvas the PRIMARY rendering mode**, not optional | P0 | LARGE |
| R-02 | **Compile FreeType to WASM** or use rustybuzz WASM for glyph rendering instead of browser `fillText()` | P0 | LARGE |
| R-03 | **Build glyph cache** — LRU cache for font files + per-codepoint glyph bitmaps | P0 | MEDIUM |
| R-04 | **Separate overlay canvas** for cursors/selections (avoid full page redraws for blinking cursor) | P1 | MEDIUM |
| R-05 | **Font metric normalization** — ensure identical metrics regardless of platform font engine | P0 | MEDIUM |
| R-06 | **Keep DOM mode as fallback** for accessibility (screen readers need DOM) but render visually via Canvas | P1 | SMALL |

**Why this matters**: Without canvas-first rendering, the same DOCX will look different on Chrome vs Safari vs Firefox. OnlyOffice solved this in 2012. This is the single biggest differentiator.

---

## 2. DOCUMENT MODEL — LOW GAP (Strong Foundation)

### OnlyOffice Model
- **CDocument** → CDocumentContent → Paragraph → ParaRun → Text/Break/Tab/etc.
- ~400,000 lines of JavaScript
- Inline objects: runs, hyperlinks, math, drawings, complex fields, content controls
- Style compilation with lazy caching
- Serialization: OOXML + custom JSON + binary DOCY format

### Rudra Model
- **Document** → Body → mixed block/inline → Paragraph → Run → Text/etc.
- ~4,021 lines (Rust, compact)
- 35 node types covering all major structures
- 50+ attribute keys for character, paragraph, table, section formatting
- Style system with inheritance chains
- CRDT-ready NodeId(replica_id, counter) on every node

### Gaps

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| M-01 | **Content Controls (SDT)** | Full: TextForm, CheckBox, ComboBox, DatePicker, PictureForm | Raw XML preservation only | HIGH |
| M-02 | **Complex Fields** | Full: MERGEFIELD, TOC, PAGE, DATE, SEQ, cross-references | Basic: PageNumber, PageCount, Date, Time, Hyperlink | MEDIUM |
| M-03 | **Run Content Types** | Text, Break, Tab, Space, FootnoteRef, EndnoteRef, PageNum, PagesCount, ComplexField, Separator | Text, LineBreak, PageBreak, ColumnBreak, Tab, Image, Field | MEDIUM |
| M-04 | **Drawing Object Model** | Full: inline/floating, anchoring, z-order, text wrapping modes, shape fills/strokes/effects | Basic: Drawing node with attributes | HIGH |
| M-05 | **Numbering System** | Full multi-level with abstract/instance numbering, level overrides | Multi-level lists with abstract/instance, bullet/decimal/roman/alpha | LOW |
| M-06 | **Theme System** | 25 color schemes, theme colors with tint/shade | ThemeColor + ThemeTintShade attributes (preservation only) | MEDIUM |
| M-07 | **Character Effects** | Shadow, Outline, Emboss, Small Caps, All Caps, character spacing | SmallCaps, Caps, FontSpacing, BaselineShift | LOW |

---

## 3. RENDERING & PAGE LAYOUT — HIGH GAP

### OnlyOffice Layout
- **True WYSIWYG** — "100% view, conversion, print, and pagination fidelity"
- **Complex text wrapping** — `WrapManager.js` (44,867 lines!) handles text flow around floating objects
- **Column layout** — multi-column sections with balanced column breaks
- **Widow/orphan control** — enabled by default, respected during pagination
- **Drop caps** — in text or in margin, configurable
- **Line numbering** — continuous, per-page, per-section
- **Hyphenation** — automatic

### Rudra Layout
- **Knuth-Plass line breaking** — optimal algorithm (actually better than OnlyOffice's greedy)
- **Basic pagination** — page breaks, spacing, margins
- **Section support** — page size, margins, orientation per section
- **Columns** — basic multi-column support
- **Hyphenation** — Knuth-Liang algorithm with English dictionary

### Gaps

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| L-01 | **Text wrapping around objects** | Full: Square, Tight, Through, Behind, In Front, Top/Bottom with distance params | None — objects don't affect text flow | CRITICAL |
| L-02 | **Floating objects positioning** | Anchored to paragraph/page/column/character, relative positioning | Inline only | CRITICAL |
| L-03 | **Widow/orphan control** | Full, default on | Attribute exists, not enforced in layout | HIGH |
| L-04 | **Keep with next / Keep lines together** | Full, respected in pagination | Attribute exists, not enforced in layout | HIGH |
| L-05 | **Drop caps** | In text or margin, configurable lines/distance | Not supported | MEDIUM |
| L-06 | **Line numbering** | Continuous, per-page, per-section, custom start | Not supported | LOW |
| L-07 | **Table layout** | Auto-fit, fixed, percentage; spanning pages; header row repeat | Basic width calculation | HIGH |
| L-08 | **Incremental layout** | Only affected pages re-layout | Full re-layout on every edit | HIGH |
| L-09 | **Header/footer layout** | Per-section, first/even/odd, fields substituted | Implemented in WASM + editor | LOW |
| L-10 | **Footnote layout** | Bottom of page, auto-numbered, overflow to next page | Editor-side only, not in layout engine | MEDIUM |
| L-11 | **Column balancing** | Auto-balance columns at section end | Not implemented | MEDIUM |

---

## 4. DOCX FORMAT FIDELITY — HIGH GAP

### OnlyOffice DOCX Support
- **Native working format** — all documents converted to DOCX internally
- **95%+ fidelity** for complex documents
- **Full element support**: paragraphs, runs, tables, shapes (VML + DrawingML), SmartArt, Charts, OLE objects, content controls, complex fields, track changes, comments, footnotes, endnotes, bookmarks, sections, headers/footers, numbering, styles, themes
- **x2t C++ converter** handles 40+ format conversions

### Rudra DOCX Support (~80% fidelity)
- **Strong core**: paragraphs, runs, formatting, styles, lists, tables, images, comments, bookmarks, sections, headers/footers, track changes detection
- **Good preservation**: unknown elements stored as raw XML for round-trip

### Gaps

| # | Element | OnlyOffice | Rudra | Impact |
|---|---------|-----------|-------|--------|
| D-01 | **Charts** | Full editing + rendering | Dropped on import | HIGH — documents with charts lose data |
| D-02 | **SmartArt** | Full creation + editing | Dropped on import | HIGH — diagrams lost |
| D-03 | **DrawingML shapes** | Full: fills, strokes, effects, text bodies | Simplified to Drawing nodes | HIGH |
| D-04 | **VML shapes** | Full parsing + conversion to DrawingML | Not parsed | MEDIUM |
| D-05 | **Equations (OMML)** | Full editor, Unicode/LaTeX input | Preserved as raw XML, not rendered | MEDIUM |
| D-06 | **OLE objects** | Supported (embedded spreadsheets etc.) | Not supported | MEDIUM |
| D-07 | **Content controls** | TextForm, CheckBox, ComboBox, DatePicker, PictureForm | Raw XML preservation | MEDIUM |
| D-08 | **Complex fields** | Full: all field types, field code parsing | Basic: limited field types | MEDIUM |
| D-09 | **Table advanced features** | Draw table, merge, formulas, auto-fit, text-to-table conversion | Basic: insert, merge, borders | HIGH |
| D-10 | **Track changes detail** | Full: insertions, deletions, moves (double underline/strikethrough), formatting changes, author/timestamp | Detection with author/date, no rendering distinction for moves | HIGH |
| D-11 | **Revision properties** | Full round-trip of all revision attributes | Basic detection | MEDIUM |
| D-12 | **w14/w15/wp14 extensions** | Full support | Debug warnings, raw XML preservation | LOW |
| D-13 | **Digital signatures** | Full signing + validation (desktop) | Detection + metadata extraction only | LOW |

---

## 5. EDITING EXPERIENCE — HIGH GAP

### OnlyOffice Editing
- **Canvas-based cursor** — custom rendered, no browser cursor bugs
- **Transaction-based history** — atomic change groups
- **IME composition** — dedicated `DocumentCompositeInput` class
- **Triple-click** paragraph selection (v9.3)
- **Spell check** built-in with language detection
- **Grammar check** via LanguageTool plugin
- **AutoCorrect** — math symbols, common typos, auto-lists
- **Track changes UI** — 4 display modes, accept/reject per change or all
- **Document comparison** — word-level diff with track changes overlay
- **Mail merge** — spreadsheet data source, field insertion, preview
- **Content controls** — interactive form fields
- **Freehand drawing** — pen/highlighter/eraser tools

### Rudra Office Editing
- **contentEditable cursor** — browser-managed, quirks per browser
- **Transaction-based history** with batch typing (good)
- **IME composition** — dedicated handling with CRDT guards (good)
- **Triple-click** paragraph selection (FS-38, implemented)
- **Format painter** — single-use and sticky mode (good)
- **Slash menu** — contextual insert commands (good, unique feature)
- **AI integration** — context-aware assistant (unique advantage)

### Gaps

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| E-01 | **Spell check** | Built-in, red underlines, right-click suggestions, custom dictionaries, language retention | None | CRITICAL |
| E-02 | **Grammar check** | LanguageTool plugin (20+ languages) | None (AI panel can do it manually) | HIGH |
| E-03 | **AutoCorrect** | Math auto-correct, text auto-correct, auto-format-as-you-type | Basic smart quotes + auto-capitalize (FS-36) | MEDIUM |
| E-04 | **Track changes UI** | Full: toggle, 4 display modes (Markup+Balloons, Only Markup, Final, Original), accept/reject per-change and all, reviewer identification | Detection only, no accept/reject UI, no display modes | CRITICAL |
| E-05 | **Document comparison** | Compare + Combine, word-level diff, load from URL/portal/upload | Not implemented | HIGH |
| E-06 | **Mail merge** | Spreadsheet data source, merge fields, preview, output to PDF/DOCX/Email | Not implemented | MEDIUM |
| E-07 | **Content controls / Forms** | Interactive: text input, checkbox, combobox, date picker, picture form | Not implemented | MEDIUM |
| E-08 | **Freehand drawing** | Pen, highlighter, eraser (Draw tab) | Not implemented | LOW |
| E-09 | **Custom cursor rendering** | Canvas-rendered cursor, no browser quirks | Browser contentEditable cursor | MEDIUM |
| E-10 | **Block/column selection** | Not a strong feature in either | Not implemented | LOW |
| E-11 | **Find & Replace regex** | No native regex (requested feature) | Regex supported (FS-08) — ADVANTAGE | N/A |
| E-12 | **Keyboard shortcut customization** | File > Advanced Settings (v9.2+) | Not implemented | LOW |
| E-13 | **Macro support** | JavaScript macros, macro recording (v9.2+) | Not implemented | LOW |

---

## 6. TABLES — HIGH GAP

### OnlyOffice Tables
- **Insert methods**: Quick grid (10x8), custom size, draw table (pencil tool), text-to-table conversion
- **Cell operations**: Merge, split, erase boundaries, draw additional rows/columns
- **Properties**: Auto-fit, manual width (cm/pt/in/%), row height, cell margins, cell spacing, vertical alignment, text direction
- **Styles**: Predefined templates, custom borders per side, cell backgrounds
- **Advanced**: Header row repeat across pages, table formulas, captions, alt text, text wrapping (inline/flow), nested tables, table-to-text conversion, sorting
- **Embedded spreadsheet**: OLE spreadsheet object in table cell

### Rudra Office Tables
- **Insert**: Rows x columns dialog
- **Cell operations**: Insert/delete rows/columns, merge cells
- **Properties**: Borders, cell background, basic width
- **Rendering**: Multi-page continuation tracking, nested tables in DOCX parsing

### Gaps

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| T-01 | **Draw table** tool (pencil) | Yes — freeform row/column creation | No | MEDIUM |
| T-02 | **Text to table / Table to text** | Yes — delimiter selection | No | MEDIUM |
| T-03 | **Table auto-fit** | Yes — auto-fit to content, window, fixed | No — manual width only | HIGH |
| T-04 | **Table formulas** | Yes — Insert Formula in table cell | No | MEDIUM |
| T-05 | **Header row repeat** | Yes — repeat on every page | No | HIGH |
| T-06 | **Table styles / templates** | Yes — predefined gallery | No — manual formatting only | HIGH |
| T-07 | **Cell vertical alignment** | Top, Center, Bottom | Attribute exists, not rendered | MEDIUM |
| T-08 | **Cell text direction** | Horizontal, Rotated down, Rotated up | Attribute exists, not rendered | LOW |
| T-09 | **Table captions** | Yes — with auto-numbering | No | LOW |
| T-10 | **Table sorting** | Yes | No | LOW |
| T-11 | **Table alt text** | Yes — title + description for a11y | No | LOW |

---

## 7. IMAGES & OBJECTS — HIGH GAP

### OnlyOffice Objects
- **Images**: BMP, GIF, JPEG, PNG, HEIF; crop (to shape, fill, fit), Photo Editor plugin (brightness, contrast, filters), position/size/alignment, alt text, hyperlinks on images
- **SmartArt**: Full creation and editing
- **Charts**: 12+ types, inline data editor, combo charts, secondary axis
- **Shapes**: 100+ autoshapes, fills (solid/gradient/pattern/texture/picture), stroke, shadow, reflection, glow, 3D effects
- **Text Art (WordArt)**: Styled text objects
- **Text Boxes**: Free-positioned text containers
- **Watermarks**: Text (predefined + custom) and image watermarks
- **Text wrapping**: 7 modes (Inline, Square, Tight, Through, Top/Bottom, In Front, Behind) with distance parameters
- **Equations**: Full OMML editor with categories, Unicode/LaTeX input, Professional/Linear display

### Rudra Office Objects
- **Images**: Insert from file, paste from clipboard, drag-drop; resize with handles, context menu (change, replace from URL, delete, alt text, properties, crop); alignment (left/center/right/inline); wrapping options in UI
- **Shapes**: 6 types (rectangle, oval, line, arrow, textbox, callout); stroke/fill; text content; grouping; z-order
- **Equations**: KaTeX rendering of LaTeX input; inline and block

### Gaps

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| O-01 | **SmartArt** | Full creation + editing | Not supported | HIGH |
| O-02 | **Charts** | 12+ types, inline data editor | Not supported | HIGH |
| O-03 | **Shape gallery** | 100+ autoshapes with categories | 6 basic types | HIGH |
| O-04 | **Shape effects** | Shadow, reflection, glow, 3D, gradient fills, pattern fills, texture fills | Solid fill + stroke only | HIGH |
| O-05 | **WordArt / Text Art** | Full styled text objects | Not supported | MEDIUM |
| O-06 | **Watermarks** | Text (predefined + custom) and image, transparency, orientation | Not supported | MEDIUM |
| O-07 | **Image editing** | Crop to shape, Photo Editor (filters, adjustments) | Basic crop + resize | MEDIUM |
| O-08 | **Text wrapping (engine)** | 7 modes with distance params, actual text flow affected | UI dropdown exists but engine doesn't reflow text | CRITICAL |
| O-09 | **Equation editor** | Full OMML, categories panel, Unicode/LaTeX input, Pro/Linear toggle | LaTeX input via KaTeX only | MEDIUM |
| O-10 | **OLE objects** | Embedded spreadsheets, etc. | Not supported | MEDIUM |
| O-11 | **Freehand drawing** | Pen, highlighter, eraser | Not supported | LOW |

---

## 8. REFERENCES & NAVIGATION — MEDIUM GAP

### OnlyOffice References
- **TOC**: Auto from headings, refresh, customizable levels
- **Table of Figures**: Auto from captions
- **Footnotes/Endnotes**: Auto-numbering, popup preview, customizable format, per-section, convert between
- **Cross-references**: 7 reference types, 8 target types, hyperlink option, separator customization
- **Bookmarks**: Named navigation targets
- **Captions**: Auto-numbered, for figures/tables/equations
- **Bibliography**: Zotero + Mendeley plugins
- **Field codes**: PAGE, NUMPAGES, DATE, TIME, SEQ, custom fields

### Rudra Office References
- **TOC**: Insert from headings, style selector (default, dotted, dashed, no-page-numbers)
- **Footnotes/Endnotes**: Insert via Ctrl+Alt+F/D, auto-numbering
- **Bookmarks**: Named anchors
- **Page numbers**: Page number field in headers/footers

### Gaps

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| N-01 | **Cross-references** | Full: 7 ref types, 8 target types | Not supported | HIGH |
| N-02 | **Table of Figures** | Auto from captions | Not supported | MEDIUM |
| N-03 | **Captions** | Auto-numbered for figures/tables/equations | Not supported | MEDIUM |
| N-04 | **Bibliography/Citations** | Zotero + Mendeley plugins | Not supported | MEDIUM |
| N-05 | **Footnote popup preview** | Hover to preview | Not supported (click to view) | LOW |
| N-06 | **Footnote/Endnote conversion** | Convert all between types | Not supported | LOW |
| N-07 | **SEQ fields** | Auto-numbered sequences | Not supported | MEDIUM |
| N-08 | **TOC refresh** | Update on demand | Not supported | MEDIUM |

---

## 9. COLLABORATION — LOW GAP (Rudra Advantage)

### OnlyOffice Collaboration
- **OT-based** — operational transformation for conflict resolution
- **Two modes**: Fast (real-time, no redo) and Strict (save-then-share, full undo/redo)
- **Paragraph locking** in Strict mode
- **Binary protocol** (DOCY format) for change sync
- **WebSocket transport** via Node.js CoAuthoringService
- **Session management**: Redis + PostgreSQL backend
- **Built-in chat** with Telegram/Jitsi plugins
- **Review-only access mode**

### Rudra Collaboration
- **CRDT-based** — Fugue algorithm, mathematically guaranteed convergence (STRONGER than OT)
- **WebSocket relay** — lightweight Node.js relay server
- **Offline support** — operations buffered, applied on reconnect
- **Exponential backoff** reconnection
- **Peer presence** with color-coded avatars and cursors
- **State vector sync** for efficient delta exchange
- **No paragraph locking needed** — CRDTs handle concurrent edits

### Gaps (Minor)

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| C-01 | **Built-in chat** | Yes, with plugin integration | Not implemented | LOW |
| C-02 | **Strict mode** (save-to-share) | Yes — gives users explicit control | No equivalent — always real-time | LOW |
| C-03 | **Review-only access** | Yes — can view but not accept/reject | Not implemented | MEDIUM |
| C-04 | **Session persistence** | Redis + PostgreSQL for crash recovery | In-memory relay (no persistence) | HIGH |
| C-05 | **Conflict resolution** | OT (works but has theoretical limits) | CRDT (mathematically correct) — ADVANTAGE | N/A |

---

## 10. FILE FORMAT COVERAGE — MEDIUM GAP

### OnlyOffice: 40+ formats
**Edit**: DOC, DOCM, DOCX, DOTX, EPUB, FB2, FODT, HTML, HWP, HWPX, MD, MHTML, ODT, OTT, RTF, STW, SXW, TXT, WPS, WPT, XML, XPS
**View**: DjVu, PDF, PDF/A, JPEG, PNG, HEIF
**Export**: DOCX, DOTX, ODT, OTT, RTF, TXT, HTML, FB2, EPUB, MD, PDF, PDF/A, JPG, PNG

### Rudra Office: 6 formats
**Read/Write**: DOCX, ODT, TXT, Markdown
**Export only**: PDF
**Partial**: XLSX (read), DOC (basic text extraction)

### Gaps

| # | Format | OnlyOffice | Rudra | Priority |
|---|--------|-----------|-------|----------|
| F-01 | **RTF** | Full read/write | Not supported | MEDIUM |
| F-02 | **HTML** | Full read/write | Not supported (to_html for rendering only) | MEDIUM |
| F-03 | **EPUB** | Read/write | Not supported | LOW |
| F-04 | **DOC (legacy)** | Full read, convert to DOCX | Basic text extraction only | HIGH |
| F-05 | **DOTX (templates)** | Full | Not supported | MEDIUM |
| F-06 | **FB2** | Read/write | Not supported | LOW |
| F-07 | **PDF reading** | View-only with text layer | PDF.js viewer in editor (separate from engine) | LOW |
| F-08 | **PDF/A export** | Yes (ISO 19005) | No | MEDIUM |
| F-09 | **Image export** (JPG/PNG) | Yes | No | LOW |
| F-10 | **XLSX** | Full (separate Spreadsheet Editor) | Partial (basic read, no formulas) | HIGH |

---

## 11. UI / UX — HIGH GAP

### OnlyOffice UI
- **Tabbed ribbon** — File, Home, Insert, Draw, Layout, References, Forms, Collaboration, Protection, View, Plugins, AI
- **Context-sensitive tabs** — Table tools, Header & Footer tab appear when relevant
- **Status bar** — page number (clickable), word count (clickable for stats), zoom controls, track changes toggle, spell check language
- **Navigation panel** — headings outline, search, comments, page thumbnails
- **Right sidebar** — contextual properties panel
- **Alt key tips** — keyboard navigation for entire ribbon
- **Dark mode** — full dark document mode with adjusted text colors
- **Interface themes** — 8 themes (Light, Dark, Contrast Dark, etc.)
- **Zoom**: 50-500%, multipage view

### Rudra Office UI
- **Menu bar** — File, Edit, View, Insert, Format, Tools
- **Single toolbar** — formatting buttons, font/size, alignment, lists
- **Status bar** — page count, word count, zoom dropdown
- **Right sidebar** — properties panel (paragraph, image, table, section)
- **View options** — pages panel, outline, comments panel
- **Zoom**: 50-200%

### Gaps

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| U-01 | **Ribbon interface** | Full tabbed ribbon with categories | Single toolbar + menus | HIGH |
| U-02 | **Context-sensitive tabs** | Appear for tables, images, headers, charts | No context tabs | HIGH |
| U-03 | **Alt key navigation** | Full keyboard ribbon navigation via key tips | Basic keyboard shortcuts only | MEDIUM |
| U-04 | **Dark mode** | Full: 8 interface themes + dark document mode | CSS prepared but not implemented | MEDIUM |
| U-05 | **Zoom range** | 50-500% + multipage view | 50-200%, single page view | MEDIUM |
| U-06 | **Page number click** | Jump to specific page | No page navigation shortcut | LOW |
| U-07 | **Word count click** | Detailed stats (words, chars, paragraphs) | Basic word/char count in status bar | LOW |
| U-08 | **Document protection** | Password (AES-256), restrict editing modes (read-only, forms only, tracked changes only, comments only) | Read-only mode toggle only | MEDIUM |
| U-09 | **Plugin system** | Full plugin API, 10+ official plugins | No plugin system | MEDIUM |
| U-10 | **Print preview** | Matches canvas exactly | Browser print dialog | LOW |

---

## 12. SECURITY & PROTECTION — MEDIUM GAP

| # | Feature | OnlyOffice | Rudra | Gap |
|---|---------|-----------|-------|-----|
| S-01 | **Password protection** | AES-256 encryption on document | Not supported | MEDIUM |
| S-02 | **Restrict editing** | Read-only, Forms only, Track changes only, Comments only | Read-only toggle | MEDIUM |
| S-03 | **Digital signatures** | Certificate-based (desktop), document becomes non-editable | Detection + metadata only | LOW |
| S-04 | **Restrict download/print/copy** | Server-level permissions | Not implemented | MEDIUM |

---

## PRIORITY ACTION PLAN

### P0 — Must Fix (Blocking for Production Use)

| # | Item | Description | Est. Effort |
|---|------|-------------|-------------|
| **P0-1** | Canvas-First Rendering | Make canvas the primary rendering mode with WASM font rasterization | 4-6 weeks |
| **P0-2** | Spell Check | Integrate Hunspell (WASM) or similar; red underlines, suggestions, custom dictionaries | 2-3 weeks |
| **P0-3** | Track Changes UI | Accept/reject per change, display modes (Markup, Final, Original), reviewer colors | 2-3 weeks |
| **P0-4** | Text Wrapping Engine | Implement text flow around floating objects in s1-layout | 3-4 weeks |
| **P0-5** | Floating Object Positioning | Anchor to paragraph/page/column, relative positioning | 2-3 weeks |

### P1 — High Priority (Feature Parity)

| # | Item | Description | Est. Effort |
|---|------|-------------|-------------|
| **P1-1** | Table Auto-Fit & Header Row Repeat | Auto-fit to content/window, header row on every page | 1-2 weeks |
| **P1-2** | Table Styles Gallery | Predefined table templates | 1 week |
| **P1-3** | Widow/Orphan + Keep With Next enforcement | Enforce in layout engine pagination | 1-2 weeks |
| **P1-4** | Cross-References | Insert cross-ref to headings, bookmarks, footnotes, figures | 2 weeks |
| **P1-5** | Captions + Table of Figures | Auto-numbered captions, auto-generated figure/table lists | 1-2 weeks |
| **P1-6** | Shape Gallery Expansion | 50+ autoshapes (flowchart, arrows, callouts, stars) | 1-2 weeks |
| **P1-7** | Document Comparison | Compare two docs, show differences as track changes | 2-3 weeks |
| **P1-8** | Chart Support (DOCX) | At minimum: preserve charts on import, display as image. Ideal: basic editing | 3-4 weeks |
| **P1-9** | SmartArt (DOCX) | At minimum: preserve on import, display as image | 2-3 weeks |
| **P1-10** | Incremental Layout | Re-layout only affected pages, not entire document | 2-3 weeks |

### P2 — Medium Priority (Polish & Completeness)

| # | Item | Description | Est. Effort |
|---|------|-------------|-------------|
| **P2-1** | Content Controls / Forms | Interactive text inputs, checkboxes, dropdowns, date pickers | 2-3 weeks |
| **P2-2** | Equation Editor (OMML) | Parse OMML from DOCX, render natively (not just LaTeX) | 2-3 weeks |
| **P2-3** | RTF Format Support | Read/write RTF files | 2 weeks |
| **P2-4** | Watermarks | Text and image watermarks with transparency | 1 week |
| **P2-5** | Dark Mode | Complete dark mode implementation | 1 week |
| **P2-6** | Grammar Check | LanguageTool integration or similar | 1-2 weeks |
| **P2-7** | AutoCorrect | Math symbols, common typos, auto-format-as-you-type | 1 week |
| **P2-8** | Ribbon Interface | Replace menubar+toolbar with tabbed ribbon | 2-3 weeks |
| **P2-9** | Session Persistence | Redis/PostgreSQL for collaboration crash recovery | 1-2 weeks |
| **P2-10** | DOC (legacy) Full Support | Complete DOC parsing beyond text extraction | 3-4 weeks |
| **P2-11** | PDF/A Export | ISO 19005 compliance for archival | 1-2 weeks |
| **P2-12** | Mail Merge | Data source integration, field insertion, preview, output | 2-3 weeks |

### P3 — Nice to Have

| # | Item | Description | Est. Effort |
|---|------|-------------|-------------|
| **P3-1** | Shape Effects | Shadow, glow, 3D, gradient/pattern/texture fills | 2 weeks |
| **P3-2** | WordArt | Styled text objects | 1 week |
| **P3-3** | Line Numbers | Continuous, per-page, per-section | 1 week |
| **P3-4** | Drop Caps | In text or margin | 1 week |
| **P3-5** | Plugin System | API for third-party extensions | 3-4 weeks |
| **P3-6** | Macro Recording | JavaScript macros | 3-4 weeks |
| **P3-7** | Built-in Chat | Collaboration chat panel | 1 week |
| **P3-8** | EPUB/FB2 Support | E-book format read/write | 2-3 weeks |
| **P3-9** | Document Protection | Password encryption, restrict editing modes | 2 weeks |
| **P3-10** | Image Export | Export pages as JPG/PNG | 1 week |

---

## RUDRA ADVANTAGES (Keep & Promote)

These are areas where Rudra Office is AHEAD of OnlyOffice:

| # | Feature | Why It's Better |
|---|---------|----------------|
| **A-1** | **CRDT collaboration** | Mathematically proven convergence vs OT's edge cases. Offline-first. No paragraph locking needed. |
| **A-2** | **Pure Rust engine** | No C/C++ dependencies, memory safe, single binary, cross-compile anywhere |
| **A-3** | **Knuth-Plass line breaking** | Optimal algorithm vs OnlyOffice's greedy approach — better typography |
| **A-4** | **Regex find & replace** | OnlyOffice doesn't have it natively (open GitHub issue) |
| **A-5** | **AI integration** | Built-in AI assistant with context awareness |
| **A-6** | **Slash menu** | Quick insert commands — modern UX pattern |
| **A-7** | **Format painter (double-click sticky)** | Well-implemented convenience feature |
| **A-8** | **WASM-first architecture** | Designed for browser from day 1, not retrofitted |
| **A-9** | **Modular crate system** | Each format is independent, consumers pick what they need |
| **A-10** | **Operation-based editing** | Every mutation is an Operation — perfect undo/redo, audit trail |
| **A-11** | **AGPL license** | Same as OnlyOffice — no licensing disadvantage |
| **A-12** | **Compact codebase** | ~100K lines (Rust) vs ~400K lines (JS) for same core functionality — easier to maintain |

---

## METRICS COMPARISON

| Metric | OnlyOffice sdkjs (word) | Rudra s1engine |
|--------|------------------------|----------------|
| **Language** | JavaScript | Rust + JavaScript (editor) |
| **Core engine size** | ~400,000 lines JS | ~100,000 lines Rust |
| **Editor UI size** | N/A (integrated) | ~41,000 lines JS |
| **Document model complexity** | CDocument (28,942 lines alone) | s1-model (4,021 lines) |
| **Rendering** | Canvas + WASM FreeType | contentEditable DOM (canvas experimental) |
| **Test count** | Unknown (not easily countable) | 1,390+ tests |
| **Format support** | 40+ formats | 6 formats |
| **Maturity** | ~14 years (since 2012) | ~1 year |
| **Architecture** | Monolithic JS | Modular Rust crates |
| **Collaboration** | OT (operational transformation) | CRDT (Fugue algorithm) |

---

## CONCLUSION

Rudra Office has a **strong architectural foundation** — the Rust engine, CRDT collaboration, operation-based editing, and modular crate system are genuinely superior to OnlyOffice's approach. The core gaps are:

1. **Rendering fidelity** — Canvas-first with WASM font rasterization is non-negotiable for a production document editor
2. **Editing completeness** — Spell check, track changes UI, and document comparison are table stakes
3. **Layout sophistication** — Text wrapping around objects and proper widow/orphan enforcement
4. **DOCX completeness** — Charts, SmartArt, and advanced shapes need at minimum preservation-and-display

The **P0 items** (canvas rendering, spell check, track changes UI, text wrapping, floating objects) should be the immediate focus. These 5 items alone would close ~60% of the perceived quality gap with OnlyOffice.
