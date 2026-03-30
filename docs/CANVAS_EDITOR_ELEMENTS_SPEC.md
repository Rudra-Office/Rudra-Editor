# Canvas-First Editor Elements Specification

**Status:** Draft element specification  
**Last updated:** 2026-03-30

## Related Documents

- `CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md`
- `CANVAS_EDITOR_LOW_LEVEL_DESIGN.md`

## Purpose

This document defines how each major document/editor element should behave in the canvas-first architecture.

For each element, the key question is:

- what remains in Rust/model/layout
- what must be painted in canvas
- what remains in DOM shell/overlay layers
- what interaction model is required

## Global Rules

### Rule 1: document semantics live in Rust

Every editable document element must be represented in the Rust model and resolved into layout geometry there.

### Rule 2: canvas paints visible document content

If the user perceives it as part of the page surface, canvas should paint it unless there is a strong browser-primitive reason not to.

### Rule 3: DOM is reserved for browser primitives

DOM may be used for:

- hidden IME/text input
- accessibility mirror
- popup UI
- menus / toolbars / dialogs
- external overlay affordances

## Element Matrix

| Element | Rust owns | Canvas paints | DOM owns | Notes |
|---|---|---|---|---|
| Page | geometry, page bounds, content rect | yes | no | page paper, page chrome |
| Section | margins, columns, headers/footers, breaks | indirectly | no | layout-only semantics |
| Paragraph | structure, line layout, alignment | yes | no | canvas paints lines/runs |
| Text run | content, shaping, metrics, attributes | yes | no | DOM mirror only for a11y |
| List marker | numbering, bullet text, indent | yes | no | derive from layout output |
| Table | structure, cell layout, border model | yes | no | no DOM table editing surface |
| Table border | resolved border segments | yes | no | deterministic painting |
| Image | object model, wrap, anchor, size | yes | maybe overlay handles | image controls may be overlay |
| Shape | geometry, anchor, style | yes | maybe overlay handles | text box content still Rust-owned |
| Header/footer | section ownership + layout | yes | no | same editor model as body |
| Footnotes/endnotes | model + layout placement | yes | no | interactions still model-based |
| Hyperlink | target + range | yes | popup overlay | click handling from hit-test |
| Bookmark/anchor | model marker + geometry | optional marker | no | mostly non-visual |
| Field / TOC | field semantics + generated content | yes | no | editing rules may differ |
| Track changes | insertion/deletion metadata | yes | review panels | review UI can be DOM |
| Comment anchor | anchor range + geometry | yes | popup/thread UI | bubble/thread in DOM |
| Selection | model range | yes | no | canvas highlight rects |
| Caret | model position | yes | hidden input only | DOM caret not canonical |
| IME composition | composition state | yes for visual underline | hidden input | bridge required |
| Spellcheck marks | diagnostics | yes | popup suggestions | browser-native path optional |
| Search highlight | query matches/ranges | yes | no | canvas overlay layer |
| Rulers/guides | page metrics + user guides | yes | no | viewport layer |
| Context menus | no | no | yes | DOM shell concern |

## Element Details

### 1. Pages

Canvas responsibilities:

- paper background
- page shadow / page gap chrome
- page outline
- print-preview fidelity under zoom

Rust responsibilities:

- page size
- page index
- content rect
- section association

### 2. Sections

Rust responsibilities:

- margins
- columns
- section breaks
- header/footer references
- page orientation and size

Canvas responsibilities:

- visual consequences only, such as guides or column boundaries if shown in UI

Sections should not become a JS-owned layout abstraction.

### 3. Paragraphs and Text Runs

Rust responsibilities:

- paragraph boundaries
- line breaking
- bidi order
- glyph/run metrics
- paragraph spacing, indentation, alignment

Canvas responsibilities:

- paint paragraph background/border
- paint list markers
- paint shaped runs at exact positions

DOM responsibilities:

- semantic mirror for accessibility only

### 4. Lists

Rust responsibilities:

- numbering sequence
- restart rules
- bullet/number formatting
- indent model

Canvas responsibilities:

- paint bullet/number glyphs and text aligned to layout geometry

Open issue:

- selection and caret movement across marker vs body text needs explicit geometry rules

### 5. Tables

Rust responsibilities:

- table structure
- row/cell ownership
- cell widths/heights
- repeats/header rows
- cell padding and background
- border resolution

Canvas responsibilities:

- paint fills, borders, text, selection regions
- paint resize affordances if desired

DOM responsibilities:

- optional floating resize handles or context panels only

### 6. Table Borders

Border painting must be explicit and deterministic.

Rust should provide resolved border segments, including:

- stroke width
- stroke style
- stroke color
- collapse resolution result
- exact segment bounds

Canvas paints the final segments in order.

### 7. Images

Rust responsibilities:

- image node metadata
- anchor/wrap behavior
- bounds/crop/transform metadata

Canvas responsibilities:

- paint the actual bitmap in page coordinates
- paint placeholder if image asset is still loading

DOM responsibilities:

- resize handles / contextual toolbar as overlays if needed

### 8. Shapes and Text Boxes

Rust responsibilities:

- shape geometry
- fill/stroke styles
- z ordering
- text-frame ownership for text boxes

Canvas responsibilities:

- vector/path painting
- control point visuals
- rotation/resize affordances if in-page

Important rule:

Text inside text boxes must still go through Rust shaping and layout, not through a separate browser text system.

### 9. Headers and Footers

Headers and footers should be treated as first-class laid-out page content, not special DOM fragments.

Rust responsibilities:

- section references
- first/default/even variants
- geometry and page association

Canvas responsibilities:

- page-relative painting in their own regions

### 10. Footnotes and Endnotes

Rust responsibilities:

- references
- numbering
- placement and flow rules

Canvas responsibilities:

- footnote separator
- footnote text blocks
- hit-testable ranges

### 11. Hyperlinks, Bookmarks, Fields, TOC

Rust responsibilities:

- anchor ranges and targets
- TOC generation/state
- field semantics

Canvas responsibilities:

- link styling
- hover decoration if desired

DOM responsibilities:

- link popup / open-link menu

### 12. Track Changes

Rust responsibilities:

- insertion/deletion/move/comment metadata
- author/date info
- accept/reject operations

Canvas responsibilities:

- insertion/deletion styling
- review highlights or badges near content

DOM responsibilities:

- review side panel, change cards, action menus

### 13. Comments

Rust responsibilities:

- comment anchors and IDs
- anchor geometry or range mapping

Canvas responsibilities:

- inline comment markers / highlight anchors

DOM responsibilities:

- thread popups, side panel, replies, resolve actions

### 14. Selection

Selection must be model-based.

Rust responsibilities:

- map anchor/focus to geometry rects
- support selection across lines/pages/tables/headers

Canvas responsibilities:

- draw highlight rects
- draw multi-range selections if needed

DOM responsibilities:

- none for visible selection

### 15. Caret / Cursor

Rust responsibilities:

- caret rect for a model position
- directional navigation semantics

Canvas responsibilities:

- visible caret painting
- blink timing integration

DOM responsibilities:

- hidden input positioning for IME and clipboard integration

### 16. IME Composition

Rust responsibilities:

- temporary composition state or composition target range

Canvas responsibilities:

- composition underline / marked text visuals

DOM responsibilities:

- actual browser IME bridge via hidden input/textarea

### 17. Spellcheck

Rust responsibilities:

- optional spellcheck diagnostics if custom spellcheck is used

Canvas responsibilities:

- underline marks and highlighted misspellings

DOM responsibilities:

- suggestion popups
- temporary bridge if browser-native spellcheck is still used

### 18. Search Highlights

Rust responsibilities:

- match ranges if search is model-aware

Canvas responsibilities:

- paint match highlight rectangles
- paint active match differently from passive matches

DOM responsibilities:

- find bar only

### 19. Rulers, Margins, Guides

These are viewport/editor chrome, not document content.

Rust responsibilities:

- page dimensions and margin values
- tab stops / indent markers if they are model-backed

Canvas responsibilities:

- ruler painting
- guide lines
- margin visuals
- drag feedback for user adjustments

DOM responsibilities:

- none unless specific controls are easier as overlays

### 20. Toolbars, Dialogs, Context Menus

These should remain DOM/UI shell concerns.

Rust does not own them.
Canvas does not paint them.

They may call Rust APIs and consume geometry/state, but they are not part of the page render scene.

## Editing Rules by Element Class

### Inline content

Examples:

- text runs
- hyperlinks
- bookmarks
- tracked insertions/deletions

Editing should be range-based and text-position-based.

### Block content

Examples:

- paragraphs
- list items
- tables
- shapes anchored as blocks

Editing should be node-based plus position-based.

### Floating / anchored objects

Examples:

- images
- shapes
- comments popups

Editing should use anchor references plus object geometry.

## Element Prioritization for Migration

### Wave 1

- page surface
- paragraphs and text runs
- selection and caret
- zoom
- rulers/margins/guides

### Wave 2

- lists
- hyperlinks
- search highlights
- headers/footers
- images

### Wave 3

- tables and table borders
- comments
- track changes
- footnotes/endnotes
- spellcheck marks

### Wave 4

- shapes and text boxes
- collaborative cursors and selections
- advanced review UI parity

## Acceptance Checklist

For each element migrated to canvas-first, verify:

1. source of truth is Rust model/layout
2. painting is canvas-based
3. interaction is model-position-based
4. DOM is used only for shell/overlay/accessibility needs
5. pagination remains Rust-owned
6. zoom fidelity remains stable
7. copy/search/accessibility behavior still works via supporting layers
