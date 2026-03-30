# Canvas-First Editor Low-Level Design

**Status:** Draft low-level design  
**Last updated:** 2026-03-30

## Related Documents

- `CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md`
- `CANVAS_EDITOR_ELEMENTS_SPEC.md`

## Purpose

This document defines the low-level runtime design for implementing a canvas-first editor on top of the current s1engine Rust/WASM stack.

It assumes the high-level direction in `CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md` is accepted.

## Current Baseline

### Current strengths

- Rust already owns the document model, operations, and layout engine.
- `s1-layout` already produces page-relative geometry.
- `s1-text` already performs shaping, bidi, hyphenation, and line-break support.
- the WASM bridge already exposes document mutation and layout entry points.

### Current blockers

- WASM surface is HTML-heavy.
- editor event handling is deeply tied to `.page-content` contenteditable containers.
- per-node HTML rendering is used as the incremental update protocol.
- DOM range state is used as the editing position model.

## Target Runtime Pipeline

```text
Document Mutation
  -> s1-ops transaction
  -> DocumentModel revision update
  -> layout invalidation
  -> s1-layout recompute (full or incremental)
  -> scene serialization to WASM
  -> JS viewport receives scene diff/full scene
  -> canvas paints pages
  -> DOM overlays update from geometry APIs
```

## Rendering Data Model

### 1. Canonical layout data

The existing `LayoutDocument` remains the internal Rust output of pagination.

### 2. UI-facing render scene

Add a UI-facing scene representation derived from layout output.

Suggested shape:

```text
RenderScene
  revision: u64
  page_count: u32
  page_size_pt: { width, height }
  visible_scale_hint: f32
  pages: Vec<ScenePage>

ScenePage
  index: u32
  section_index: u32
  bounds_pt: Rect
  content_rect_pt: Rect
  header_rect_pt: Option<Rect>
  footer_rect_pt: Option<Rect>
  items: Vec<SceneItem>

SceneItem
  TextRun
  ParagraphBackground
  ParagraphBorder
  ListMarker
  TableGrid
  TableCellBackground
  TableCellBorder
  Image
  Shape
  HeaderFooterAnchor
  BookmarkAnchor
  CommentAnchor
```

The scene should represent static page content only. It should not include transient UI state like active selection, blink state, drag handles, or ruler hover feedback.

## WASM API Additions

### Rendering APIs

Add scene-oriented APIs alongside existing HTML APIs.

Suggested APIs:

- `layout_scene_json(config)`
- `layout_scene_json_with_fonts(font_db, config)`
- `page_scene_json(page_index)`
- `visible_page_scenes_json(start_page, end_page)`
- `document_revision()`

### Geometry APIs

These are required to break the dependency on DOM ranges.

Suggested APIs:

- `hit_test(page_index, x_pt, y_pt) -> PositionHit`
- `caret_rect(position) -> Rect`
- `selection_rects(anchor, focus) -> Vec<Rect>`
- `word_boundary_at(position)`
- `line_boundary_at(position)`
- `node_bounds(node_id) -> Rect | Vec<Rect>`

### Editing APIs

The browser should talk in positions/ranges, not DOM nodes.

Suggested APIs:

- `insert_text_at(position, text)`
- `delete_range(anchor, focus)`
- `replace_range(anchor, focus, text)`
- `toggle_mark_on_range(anchor, focus, mark)`
- `set_block_attrs(node_id, attrs)`
- `insert_paragraph_break(position)`
- `insert_table(position, spec)`
- `insert_image(position_or_anchor, spec)`
- `move_selection(direction, granularity, extend)`

### IME / composition APIs

Composition must not mutate the document model on every intermediate browser event unless explicitly desired.

Suggested APIs:

- `begin_composition(position)`
- `update_composition(text, selection_start, selection_end)`
- `commit_composition(text)`
- `cancel_composition()`

## Frontend Subsystems

### 1. CanvasViewportController

Responsibilities:

- page scroll model
- zoom factor
- device pixel ratio scaling
- visible page calculation
- dirty rect scheduling
- offscreen page culling

Rules:

- internal world coordinates remain in points
- CSS pixels are derived from points and zoom
- backing canvas resolution multiplies by devicePixelRatio

### 2. CanvasPageRenderer

Responsibilities:

- paint page background
- paint margins/guides/rulers
- paint scene items in z-order
- cache immutable page layers when possible
- repaint only dirty pages/regions

Recommended paint order:

1. page paper/background
2. page shadow / gap chrome
3. guides / margin indicators
4. paragraph and cell backgrounds
5. table fills and borders
6. images / behind-text shapes
7. text runs and list markers
8. in-front shapes / handles
9. selection fills
10. cursor(s)
11. composition underline / spell marks

### 3. InputBridge

Responsibilities:

- maintain hidden textarea/input
- translate browser key/input/composition events to model operations
- request caret geometry from WASM for IME anchoring
- handle clipboard bridge

Notes:

- hidden input should be positioned near the active caret rect
- browser selection should not represent document selection
- clipboard serialization should come from model ranges, not DOM selection extraction

### 4. SelectionController

Responsibilities:

- primary caret position
- anchor/focus range state
- multi-cursor state
- selection painting requests
- keyboard navigation granularity

Selection source of truth should be model positions, not DOM nodes.

### 5. OverlayManager

Responsibilities:

- context menus
- comment cards
- spellcheck suggestion popups
- resize/rotate handles for shapes and images
- hyperlink popups

These may remain DOM overlays positioned from canvas/world geometry.

### 6. AccessibilityMirror

Responsibilities:

- expose semantic reading order
- map current page/range/focus to assistive tech
- support screen reader traversal of paragraphs, headings, tables, and links

This layer should be derived from the model/layout scene, not hand-maintained from the page DOM editor.

## Proposed Module Split

### Rust side

- `s1-model`: document nodes, attributes, semantic ranges, anchors
- `s1-ops`: transactions, transforms, undo/redo, selection movement semantics
- `s1-text`: shaping, bidi, break opportunities, font metrics
- `s1-layout`: pagination, line boxes, table geometry, page object placement
- `ffi/wasm`: serialized scene, geometry queries, editing entry points

### Browser side

- `editor/src/canvas/viewport.*`: scroll, zoom, DPR handling, page visibility
- `editor/src/canvas/renderer.*`: scene painting, caches, invalidation scheduling
- `editor/src/canvas/selection.*`: canvas selection and caret state
- `editor/src/input/bridge.*`: hidden textarea, keyboard, composition, clipboard
- `editor/src/overlay/*`: comments, spellcheck popups, resize handles, menus
- `editor/src/a11y/*`: semantic mirror and assistive-tech synchronization

This split keeps the browser thin and prevents layout logic from drifting into JS.

## Pagination and Incrementality

### Rust-side ownership

Pagination remains fully owned by `s1-layout`.

JS may request:

- full scene
- affected pages only
- page-map summary
- geometry for a given revision

JS must not decide page breaks.

### Incremental strategy

Recommended invalidation levels:

1. **paint-only dirty**: selection, cursor, hover, guides
2. **page-scene dirty**: formatting or object changes that do not shift later pages
3. **pagination dirty from page N**: content height changes, section changes, table changes
4. **document-wide dirty**: style/global setting changes that alter all pages

Use the current layout cache machinery as the foundation for page-scene caching.

## Zoom Model

Keep all engine geometry in points.

Frontend transform chain:

```text
point-space -> zoomed CSS pixel space -> device backing pixel space
```

Rules:

- do not mutate model/layout units for zoom
- canvas size = css_size * devicePixelRatio
- text/object painting should happen in world coordinates with a transform, not by rewriting scene coordinates

## Tables and Borders

Table rendering should be canvas-painted from explicit border geometry, not inferred from nested DOM boxes.

Recommended internal representation per cell/table:

- content rect
- padding rect
- background rect
- resolved border segments (top/right/bottom/left)
- collapsed-border resolution result if applicable

The renderer should paint borders segment-first to avoid DOM-style inconsistencies.

## Images and Shapes

### Images

Rust scene should provide:

- source reference / decoded handle key
- object bounds
- clipping rect
- wrap mode
- z order
- transform metadata if rotation/crop is supported

### Shapes / text boxes

Do not treat shapes as ad hoc DOM widgets.

Scene should provide:

- geometry primitive or path
- fill/stroke
- z layer
- anchor behavior
- optional embedded text frame

Text inside text boxes should still route through Rust shaping/layout logic.

## Spellcheck and Comments

### Spellcheck

Canvas cannot rely on browser-native red underlines without a DOM text surface.

Options:

1. custom spellcheck engine + canvas painting
2. hidden mirror text surface for browser spellcheck only

Recommendation: plan for custom spellcheck rendering, even if a hidden mirror is used temporarily.

### Comments

Comment markers/anchors should be part of scene geometry.
Comment threads/panels can remain DOM overlays.

## Performance Targets

Recommended editor targets:

- steady 60 FPS for cursor/selection/scroll interactions
- no full-document repaint on caret move
- visible-page-only painting under normal scroll
- repaint of changed page(s) only after local edit when possible
- no HTML diffing requirement for editor correctness

## Risks and Open Decisions

### Hard problems

- accessibility parity with canvas
- browser IME edge cases
- custom spellcheck UX
- clipboard fidelity for rich ranges
- selection behavior across tables, headers/footers, and shapes

### Decisions to lock early

- JSON vs binary scene serialization
- whether scene lives in `s1-layout` or a dedicated UI-facing crate/module
- exact model position format across WASM boundary
- first-pass approach for accessibility mirror

## Recommended First Implementation Slice

The first low-risk slice should be:

1. scene serialization for read-only pages
2. canvas page painting for visible pages
3. page hit-testing and caret rect API
4. hidden-input bridge for a single caret
5. selection painting for plain paragraphs only

Do not start with tables, comments, shapes, collaborative cursors, or review UI first.

This slice removes the riskiest DOM ownership while keeping the migration bounded.
