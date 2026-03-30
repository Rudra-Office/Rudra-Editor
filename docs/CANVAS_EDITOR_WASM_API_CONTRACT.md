# Canvas-First Editor WASM API Contract

**Status:** Draft API contract  
**Last updated:** 2026-03-30  
**Applies to branch:** `feature/reimagine-port`

## Related Documents

- `CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md`
- `CANVAS_EDITOR_LOW_LEVEL_DESIGN.md`
- `CANVAS_EDITOR_IMPLEMENTATION_ROADMAP.md`
- `WASM_DESIGN.md`

## Purpose

This document defines the concrete browser boundary for the canvas-first editor.

The current WASM surface in `ffi/wasm/src/lib.rs` is still centered on HTML output and DOM-oriented incremental rendering. The new contract adds scene, geometry, navigation, editing, composition, and clipboard APIs without removing the existing HTML APIs immediately.

## Contract Rules

### 1. Rust is authoritative

- document content and attributes live in Rust
- model positions are resolved in Rust
- layout and page geometry are resolved in Rust
- JS does not invent page breaks, caret rects, or hit-test outcomes

### 2. Geometry units are always points

Every coordinate crossing the boundary uses points.

JS is responsible for converting:

```text
points -> CSS pixels -> backing pixels
```

### 3. Structured values should use `JsValue`, not JSON strings

For canvas rendering and hot geometry queries, the canonical API should return structured objects through `serde_wasm_bindgen`.

Temporary compatibility wrappers may expose `_json` variants, but they are not the preferred long-term surface.

### 4. Boundary text offsets use UTF-16 code units

The browser and IME APIs already speak UTF-16 offsets. To avoid constant JS-side translation, model positions crossing the boundary use UTF-16 offsets.

Rust may normalize internally, but the browser contract is:

- `offset_utf16` for positions inside text-bearing nodes
- navigation APIs return canonical positions after normalization

### 5. Every mutable operation returns revisions and dirty-page information

JS must be able to repaint only what changed.

## Canonical Value Types

### PositionRef

```json
{
  "node_id": "n_1042",
  "offset_utf16": 15,
  "affinity": "downstream"
}
```

Fields:

- `node_id`: globally unique model node id for a text-bearing location
- `offset_utf16`: offset inside that node's logical text content
- `affinity`: `downstream` or `upstream` for ambiguous boundaries

### RangeRef

```json
{
  "anchor": {
    "node_id": "n_1042",
    "offset_utf16": 3,
    "affinity": "downstream"
  },
  "focus": {
    "node_id": "n_1042",
    "offset_utf16": 18,
    "affinity": "downstream"
  }
}
```

### RectPt

```json
{
  "page_index": 2,
  "x": 72.0,
  "y": 144.0,
  "width": 83.5,
  "height": 14.0
}
```

### SceneSummary

```json
{
  "protocol_version": 1,
  "document_revision": 18,
  "layout_revision": 11,
  "page_count": 7,
  "default_page_size_pt": { "width": 612.0, "height": 792.0 },
  "pages": [
    {
      "page_index": 0,
      "section_index": 0,
      "bounds_pt": { "x": 0.0, "y": 0.0, "width": 612.0, "height": 792.0 },
      "content_rect_pt": { "x": 72.0, "y": 72.0, "width": 468.0, "height": 648.0 },
      "has_header": true,
      "has_footer": true,
      "item_count": 128
    }
  ]
}
```

### PageScene

```json
{
  "page_index": 0,
  "document_revision": 18,
  "layout_revision": 11,
  "bounds_pt": { "x": 0.0, "y": 0.0, "width": 612.0, "height": 792.0 },
  "content_rect_pt": { "x": 72.0, "y": 72.0, "width": 468.0, "height": 648.0 },
  "items": [
    {
      "item_id": "run_991",
      "kind": "text_run",
      "z_index": 70,
      "node_id": "n_1042",
      "bounds_pt": { "x": 84.0, "y": 102.0, "width": 120.0, "height": 14.0 },
      "text": "Hello world",
      "font": {
        "family": "Noto Sans",
        "size_pt": 11.0,
        "weight": 400,
        "style": "normal"
      },
      "fill": "#111111"
    }
  ]
}
```

### HitTestResult

```json
{
  "page_index": 0,
  "kind": "text",
  "position": {
    "node_id": "n_1042",
    "offset_utf16": 6,
    "affinity": "downstream"
  },
  "node_id": "n_1042",
  "item_id": "run_991",
  "inside": true
}
```

Possible `kind` values:

- `text`
- `image`
- `shape`
- `table_cell`
- `header`
- `footer`
- `page_margin`
- `none`

### EditResult

```json
{
  "document_revision": 19,
  "layout_revision": 12,
  "dirty_pages": { "start": 0, "end": 1 },
  "selection": {
    "anchor": {
      "node_id": "n_1042",
      "offset_utf16": 12,
      "affinity": "downstream"
    },
    "focus": {
      "node_id": "n_1042",
      "offset_utf16": 12,
      "affinity": "downstream"
    }
  }
}
```

## Proposed `WasmDocument` Additions

### Revision and capability methods

| Method | Returns | Notes |
|---|---|---|
| `scene_protocol_version()` | `u32` | Start at `1` |
| `document_revision()` | `u64` | Bumps on every model mutation |
| `layout_revision()` | `u64` | Bumps when pagination output changes |
| `editor_capabilities()` | `JsValue` | Feature flags such as tables/comments/spellcheck support |

### Scene methods

| Method | Arguments | Returns | Notes |
|---|---|---|---|
| `scene_summary(config)` | `WasmLayoutConfig` | `JsValue` | Light page map for viewport boot |
| `page_scene(page_index, options)` | `u32`, `JsValue` | `JsValue` | Full scene for one page |
| `visible_page_scenes(start_page, end_page, options)` | `u32`, `u32`, `JsValue` | `JsValue` | Batch page fetch for viewport |
| `node_bounds(node_id)` | `string` | `JsValue` | All page rects for a node |

Recommended `options` fields:

- `include_text_runs`
- `include_backgrounds`
- `include_guides`
- `include_debug_ids`
- `include_comment_anchors`

### Geometry and navigation methods

| Method | Arguments | Returns | Notes |
|---|---|---|---|
| `hit_test(page_index, x_pt, y_pt, options)` | `u32`, `f64`, `f64`, `JsValue` | `JsValue` | Primary pointer query |
| `caret_rect(position)` | `JsValue` | `JsValue` | Rect for hidden input and caret paint |
| `selection_rects(range)` | `JsValue` | `JsValue` | Ordered rect list for canvas highlight |
| `move_position(position, direction, granularity)` | `JsValue`, `string`, `string` | `JsValue` | Returns normalized `PositionRef` |
| `move_range(range, direction, granularity, extend)` | `JsValue`, `string`, `string`, `bool` | `JsValue` | Returns normalized `RangeRef` |
| `word_boundary(position)` | `JsValue` | `JsValue` | For double-click and ctrl+arrow logic |
| `line_boundary(position, side)` | `JsValue`, `string` | `JsValue` | For home/end behavior |

Recommended enums:

- `direction`: `forward`, `backward`, `up`, `down`
- `granularity`: `character`, `word`, `line`, `paragraph`, `document`

### Editing methods

| Method | Arguments | Returns | Notes |
|---|---|---|---|
| `insert_text_at(position, text)` | `JsValue`, `string` | `JsValue` | Collapsed insertion |
| `replace_range(range, text)` | `JsValue`, `string` | `JsValue` | Primary typing path |
| `delete_range(range)` | `JsValue` | `JsValue` | Explicit range deletion |
| `insert_paragraph_break(position)` | `JsValue` | `JsValue` | Enter key |
| `toggle_mark(range, mark)` | `JsValue`, `string` | `JsValue` | Bold/italic/etc |
| `set_block_attrs(node_id, attrs)` | `string`, `JsValue` | `JsValue` | Alignment, spacing, list attrs |
| `insert_image(anchor, spec)` | `JsValue`, `JsValue` | `JsValue` | Phase 5+ |
| `insert_table(anchor, spec)` | `JsValue`, `JsValue` | `JsValue` | Phase 6 |

### Composition methods

| Method | Arguments | Returns | Notes |
|---|---|---|---|
| `begin_composition(position)` | `JsValue` | `JsValue` | Starts transient composition state |
| `update_composition(text, selection_start_utf16, selection_end_utf16)` | `string`, `u32`, `u32` | `JsValue` | Returns preview range/rects |
| `commit_composition(text)` | `string` | `JsValue` | Produces final `EditResult` |
| `cancel_composition()` | — | `JsValue` | Clears transient state |

### Clipboard and search helpers

| Method | Arguments | Returns | Notes |
|---|---|---|---|
| `copy_range_plain_text(range)` | `JsValue` | `string` | Clipboard plain text |
| `copy_range_html(range)` | `JsValue` | `string` | Rich clipboard/export fragment |
| `search_matches(query, options)` | `string`, `JsValue` | `JsValue` | Returns ranges and page-local rects |

## Compatibility Strategy

Existing methods stay during migration:

- `to_html()`
- `to_paginated_html*()`
- `render_node_html()`
- current layout JSON helpers already used by `canvas-render.js`

But they should be treated as:

- legacy DOM editor support
- export/inspection tools
- temporary migration shims

They should not remain the canonical canvas editor contract.

## Browser Flow for the First Canvas Slice

1. call `scene_summary(config)` on document open
2. render visible pages via `visible_page_scenes(start, end, options)`
3. on click, call `hit_test(page, x_pt, y_pt, options)`
4. place hidden textarea using `caret_rect(position)`
5. on typing, call `replace_range(range, text)`
6. repaint only `dirty_pages` from the returned `EditResult`

## Acceptance Requirements

The contract is acceptable when:

- it can render and edit without DOM page content ownership
- it does not require JS to infer geometry from HTML
- it returns enough dirty-page information for incremental repaint
- it can drive IME, clipboard, and selection without browser ranges as the source of truth
