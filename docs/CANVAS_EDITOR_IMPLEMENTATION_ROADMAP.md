# Canvas-First Editor Implementation Roadmap

**Status:** Draft delivery roadmap  
**Last updated:** 2026-03-30  
**Applies to branch:** `feature/reimagine-port`

## Related Documents

- `CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md`
- `CANVAS_EDITOR_LOW_LEVEL_DESIGN.md`
- `CANVAS_EDITOR_ELEMENTS_SPEC.md`
- `WASM_DESIGN.md`

## Goal

This roadmap turns the canvas-first editor architecture into a staged delivery plan that can run inside the current repo without a big-bang rewrite.

The core migration rule is simple:

> **Add a canvas editor path beside the current DOM editor, then retire DOM ownership only after parity.**

## Current State Snapshot

The current browser editor is still DOM-first in the areas that matter most:

- `ffi/wasm/src/lib.rs` exposes HTML-heavy rendering methods as the main UI surface.
- `editor/src/render.js` owns full document re-rendering and incremental node HTML patching.
- `editor/src/input.js` depends on DOM selection, DOM ranges, and `contenteditable` mutation flow.
- `editor/src/canvas-render.js` already exists, but it is a prototype renderer over layout JSON rather than the final scene-based runtime.

That means the migration is not starting from zero, but it is also not just a rendering swap. Input, geometry, and state ownership all need to move.

## Program Constraints

- Rust remains the source of truth for model, ops, shaping, pagination, and geometry.
- The DOM editor path remains available as a fallback until final parity.
- All geometry crossing the browser boundary uses points, not CSS pixels.
- Canvas mode must be behind an explicit feature flag while the migration is in progress.
- Every phase needs measurable exit criteria, not just code completion.

## Delivery Workstreams

### 1. Rust scene and geometry workstream

Owns:

- scene generation from `s1-layout`
- hit-testing and caret/selection geometry
- model-position navigation helpers
- dirty-page and layout-revision reporting

### 2. WASM boundary workstream

Owns:

- scene API exposure
- structured JS/WASM contract
- edit-operation return payloads
- composition and clipboard helpers

### 3. Canvas frontend workstream

Owns:

- viewport and page stack
- canvas page painting
- pointer interaction routing
- cursor and selection painting
- zoom, rulers, guides, and page chrome

### 4. Shell, overlay, and accessibility workstream

Owns:

- hidden input and IME bridge
- context menus and toolbars
- spellcheck and comment popups
- accessibility mirror and focus proxy

### 5. Verification and performance workstream

Owns:

- phase entry and exit checks
- document regression coverage
- large-document interaction benchmarks
- parity matrix against the legacy DOM editor

## Phase Plan

### Phase 0: Contract Freeze and Observability

**Purpose:** lock the boundary before implementation fragments the design.

Deliverables:

- final approval of the high-level, low-level, and elements specs
- exact WASM contract for scene, geometry, editing, and composition
- explicit feature flag for `dom` vs `canvas` editor surface
- document revision and layout revision reporting in the browser session
- benchmark baselines for open, paginate, scroll, caret move, and local text edit

Exit criteria:

- the team can describe which layer owns pagination, selection, hit-testing, and rendering without ambiguity
- the editor can switch surfaces without changing the document model
- benchmark baselines exist for at least small, medium, and large documents

### Phase 1: Read-Only Canvas Pages

**Purpose:** replace page DOM ownership for display before replacing editing.

Scope:

- page scene summary from Rust/WASM
- visible-page scene fetch APIs
- canvas viewport and page renderer
- zoom and print-like page chrome
- optional fallback to current DOM path if canvas render fails

Files likely touched:

- `crates/s1-layout/*`
- `ffi/wasm/src/lib.rs`
- new `editor/src/canvas/*`
- `editor/src/render.js`

Exit criteria:

- canvas mode can open and display multi-page documents read-only
- scroll and zoom operate on visible pages only
- page dimensions, margins, and page count match Rust pagination
- no DOM `.page-content` nodes are required for visual page display in canvas mode

### Phase 2: Geometry and Hit-Testing

**Purpose:** make model positions addressable without DOM ranges.

Scope:

- hit-test API
- caret rect API
- selection rect API
- page lookup and node bounds API
- keyboard navigation helpers for character, word, line, and paragraph movement

Exit criteria:

- mouse click can resolve to a model position in canvas mode
- keyboard caret moves do not depend on DOM selection APIs
- geometry remains correct across page boundaries and headers/footers

### Phase 3: Single-Caret Text Input and IME

**Purpose:** replace `contenteditable` text insertion with a hidden-input bridge.

Scope:

- hidden textarea near caret geometry
- insert, delete, replace, and paragraph break APIs
- composition lifecycle APIs
- canvas caret painting
- undo grouping for typing in the new path

Exit criteria:

- typing works in normal paragraphs with the DOM editor surface disabled
- IME composition works for at least the primary supported browser matrix
- browser-native caret is no longer the canonical caret in canvas mode

### Phase 4: Selection, Navigation, Clipboard, and Search

**Purpose:** move the rest of core text editing off the DOM path.

Scope:

- drag selection and shift-selection extension
- multi-line and cross-page selection painting
- copy and cut via model ranges
- paste via plain text and structured insert APIs
- search highlight painting
- selection-aware toolbar state driven from model ranges

Exit criteria:

- mouse selection works in canvas mode without DOM range ownership
- copy, cut, and paste work for plain text and basic rich content
- find/highlight works entirely from model/layout geometry

### Phase 5: Structured Regions and Common Objects

**Purpose:** bring the common non-trivial page elements onto the canvas path.

Scope:

- list markers and list navigation
- headers and footers
- hyperlinks and bookmark hit targets
- images and resize handles
- footnotes/endnotes display and hit-testing
- rulers, margins, and guides

Exit criteria:

- the main “print editor” feel exists in canvas mode
- headers/footers and body content share one model-based interaction system
- images render and can be selected without DOM page ownership

### Phase 6: Tables, Review Features, and Diagnostics

**Purpose:** move the highest-complexity document features.

Scope:

- table cell hit-testing and selection
- deterministic table border painting
- track changes marks and review cues
- comment anchors and popup positioning
- spellcheck underline painting and suggestion overlays

Exit criteria:

- table editing and navigation no longer depend on a DOM table surface
- comments and review UI use geometry from the canvas path
- spellcheck no longer depends on visible DOM page text

### Phase 7: Accessibility, Parity, and DOM Retirement

**Purpose:** remove DOM ownership only when the replacement is demonstrably complete.

Scope:

- semantic accessibility mirror
- screen-reader traversal checks
- parity checklist against the elements spec
- deprecation of DOM-only incremental rendering APIs from active editor usage
- fallback DOM surface kept only as a compatibility mode if still needed

Exit criteria:

- canvas mode is the default editor surface
- DOM editing is no longer required for mainstream authoring workflows
- accessibility, clipboard, and search are still acceptable after the switch

## Element-to-Phase Map

| Element group | First required phase | Notes |
|---|---|---|
| Page surface, zoom, page chrome | Phase 1 | Foundation for everything else |
| Caret, click hit-test, basic navigation | Phase 2 | Removes DOM range dependency |
| Text input, IME | Phase 3 | First true editing milestone |
| Selection, clipboard, search | Phase 4 | Core editor parity |
| Lists, headers/footers, links, images, rulers | Phase 5 | Print-editor feel |
| Tables, comments, track changes, spellcheck | Phase 6 | Highest complexity bucket |
| Accessibility mirror and DOM retirement | Phase 7 | Final parity and cleanup |

## Recommended First Milestone Package

If delivery has to be sliced aggressively, the best first milestone is:

1. scene summary and visible-page scene APIs
2. read-only canvas page rendering
3. click hit-testing and caret rectangle API
4. hidden-input bridge for a single caret
5. plain paragraph typing in canvas mode

That package proves the new architecture on the critical path without starting with tables, comments, or review features.

## Risks To Watch Early

- IME behavior can look complete in simple tests while still failing under rapid focus changes.
- Accessibility debt compounds if the mirror layer is postponed too long.
- Performance will regress if scene serialization is too chatty or page caching is weak.
- Table interaction can consume the schedule if introduced before plain-text parity is stable.
- Keeping DOM and canvas paths side-by-side for too long will increase maintenance cost.

## Definition of Success

The program is successful when the browser no longer needs DOM page content to determine what the document is, where content belongs, or where the selection is. At that point, DOM becomes shell infrastructure rather than editor truth.
