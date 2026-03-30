# Canvas-First Editor High-Level Architecture

**Status:** Draft architecture baseline  
**Last updated:** 2026-03-30  
**Applies to branch:** `feature/reimagine-port`

## Related Documents

- `CANVAS_EDITOR_LOW_LEVEL_DESIGN.md`
- `CANVAS_EDITOR_ELEMENTS_SPEC.md`

## Purpose

This document defines the target browser editor architecture for s1engine after moving away from a DOM/contenteditable-owned editing model.

The core decision is:

> **Adopt a canvas-first page editor over a Rust-owned document, layout, and text engine.**

Canvas owns visual rendering. Rust owns document state, editing operations, text shaping, pagination, geometry, and hit-testable layout data.

## Why This Direction Fits This Repo

The existing Rust workspace already centers engine ownership in the right crates:

- `s1-model`: document structure and attributes
- `s1-ops`: editing transactions, undo/redo, selection/cursor primitives
- `s1-layout`: pagination and page geometry
- `s1-text`: shaping, bidi, line breaking, font metrics
- `s1engine`: high-level document facade

The current mismatch is mainly in the browser layer:

- `ffi/wasm/src/lib.rs` exposes `to_html()` and `render_node_html()` as first-class rendering APIs.
- `editor/src/render.js` is built around DOM re-rendering and incremental HTML patching.
- `editor/src/input.js`, `editor/src/pagination.js`, and related modules assume `.page-content` contenteditable containers are the editing surface.
- `crates/s1-layout/src/html.rs` explicitly optimizes for HTML output rather than canvas output.

That means the repo is **engine-first underneath, DOM-first at the UI boundary**. The redesign should correct that boundary rather than rethinking the core crates from scratch.

## Architectural Decision

### Keep in Rust / WASM

Rust remains authoritative for:

- document structure
- editing transactions and undo/redo
- text shaping and line breaking
- pagination and page geometry
- hit-testing data and cursor/selection geometry
- page-relative object placement
- section logic, headers/footers, tables, borders, images, and shapes metadata

### Move to Canvas

Canvas becomes authoritative for visible editor painting:

- page rendering
- custom pagination visuals
- rulers, margins, guides
- selection painting
- caret painting
- table border painting
- image and shape painting
- zooming and print-like fidelity

### Keep in DOM / HTML Shell

DOM stays only where browser primitives matter:

- hidden text input / IME bridge
- accessibility mirror layer
- menus, toolbars, dialogs, context menus
- popup overlays for comments, spellcheck suggestions, link UI, etc.

## Non-Goals

This redesign does **not** mean:

- moving pagination into JavaScript
- rasterizing the entire document into static images
- dropping accessibility
- dropping search/select/copy behavior
- rewriting the Rust core model and ops crates
- a big-bang replacement of every UI subsystem at once

## Target System Layers

```text
Browser Application Shell
  ├─ Toolbar / Menus / Dialogs / Popups (DOM)
  ├─ Hidden Input + IME Bridge (DOM)
  ├─ Accessibility Mirror (DOM)
  └─ Canvas Viewport + Interaction Controller (JS/TS)
         │
         ▼
WASM UI Boundary
  ├─ Document operations
  ├─ Render scene / page scene APIs
  ├─ Hit-testing APIs
  ├─ Caret / selection geometry APIs
  └─ Incremental layout APIs
         │
         ▼
Rust Engine Core
  ├─ s1engine facade
  ├─ s1-model document tree
  ├─ s1-ops transactions + history
  ├─ s1-layout pagination + geometry
  └─ s1-text shaping + bidi + line breaking
```

## Ownership Model

| Layer | Owns | Does Not Own |
|---|---|---|
| Rust core | Model, ops, layout, geometry, hit data | DOM focus, browser UI widgets |
| WASM boundary | Stable serialized API surface | Business logic duplication |
| JS canvas app | Viewport, painting, gesture routing, input orchestration | Canonical document state |
| DOM shell | IME, a11y, overlays, menus | Page content model or pagination |

## Key Principles

### 1. Model-first, not DOM-first

The browser DOM must no longer be the editable source of truth. All meaningful edits must resolve to model positions and `s1-ops` transactions.

### 2. Pagination is an engine responsibility

Pagination should remain in `s1-layout`. Canvas paints what Rust paginated. JavaScript may decide what to draw, but not where content truly belongs.

### 3. View state is separate from document state

Selection, cursor blink, hover, viewport scroll, zoom, active handles, and comment popups are view concerns. The document model should stay independent.

### 4. Accessibility is preserved via a semantic mirror

Canvas alone is insufficient for screen readers and some browser features. A semantic accessibility layer must reflect the current document/page context.

### 5. Migration is additive first, subtractive later

A new canvas path should coexist with the existing DOM editor until the core workflows reach parity.

## Recommended Migration Shape

### Phase 0: Freeze the target boundary

Define what JS is allowed to own and what must stay in Rust.

### Phase 1: Render scene output

Expose a structured page scene from WASM alongside existing HTML output.

### Phase 2: Read-only canvas pages

Render paginated pages to canvas using existing layout output, without replacing editing yet.

### Phase 3: Geometry APIs

Add hit-testing, caret rectangles, selection rectangles, and block/run lookup APIs.

### Phase 4: Input bridge

Move typing, IME, cursor, and selection off contenteditable and onto hidden-input + canvas painting.

### Phase 5: Rich objects and overlays

Bring tables, images, shapes, comments, spellcheck marks, rulers, and guides onto the canvas pipeline.

### Phase 6: Retire DOM editing path

Remove `.page-content` contenteditable ownership once parity and test coverage are sufficient.

## What Must Change in the Current Repo

### Rust / WASM

- `ffi/wasm` must expose scene and geometry APIs, not only HTML strings.
- `s1-layout` must be treated as the rendering scene source.
- `s1engine` document operations should remain the only mutation path.

### Editor UI

- `editor/src/render.js` should stop treating DOM HTML as the canonical page content.
- `editor/src/input.js` should stop depending on DOM range + contenteditable as the main editing protocol.
- `editor/src/pagination.js` should stop assuming page DOM containers define layout truth.

## Immediate Deliverables

The first architecture milestone should produce:

1. a render-scene API shape
2. a geometry API shape
3. a canvas renderer that can paint current paginated pages read-only
4. a migration plan for replacing DOM selection and caret logic

## Planning Outcome

This document is the high-level contract.
The low-level implementation shape lives in `CANVAS_EDITOR_LOW_LEVEL_DESIGN.md`.
Element-by-element ownership and behavior lives in `CANVAS_EDITOR_ELEMENTS_SPEC.md`.

## Success Criteria

This architecture is successful when:

- Rust remains the single source of truth for document and page geometry
- canvas is the primary page renderer
- DOM is reduced to shell responsibilities
- zoom and pagination behave like a print editor
- selection/caret/table/images/shapes render consistently across pages
- HTML output still exists as export/compatibility tooling, not as the editor surface
