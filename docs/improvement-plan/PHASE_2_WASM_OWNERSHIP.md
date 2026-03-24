# Phase 2: WASM Ownership — Full Engine Authority

## Status: Substantially Complete (2026-03-25)

## Architectural Principle

```
User Input → Editor (input.js) → WASM Engine → HTML/Layout Output → Editor (render.js) → DOM

The editor NEVER:
  - measures DOM to decide layout
  - clips/splits paragraphs with CSS
  - computes page breaks from element heights
  - determines formatting state from DOM attributes
  - positions cursors from DOM coordinates

The editor ONLY:
  - captures user input (keyboard, mouse, paste, drag)
  - forwards input as WASM operations (insert_text, delete_text, split_paragraph, etc.)
  - receives HTML/layout data from WASM
  - mounts that output into the DOM exactly as-is
  - manages scroll position and viewport
```

## Current Flow (Broken)

```
1. User types → input.js reads DOM text → syncParagraphText() → set_paragraph_text/replace_text
2. render.js calls doc.to_html() → gets full document HTML
3. pagination.js calls doc.get_page_map_json() → gets page assignments
4. pagination.js distributes paragraphs into page divs by node ID
5. pagination.js measures DOM elements (offsetHeight, scrollHeight) to detect overflow
6. If overflow → applySplitParagraphClipping() clips with CSS negative margins
7. Cursor restored from saved nodeId+offset via DOM TreeWalker
```

**Problems with this flow:**
- Step 1: Reading DOM text is lossy (getEditableText strips markers/fields)
- Step 5: DOM measurements create a render→measure→re-render cycle (flicker)
- Step 6: CSS clipping shows full paragraph briefly before clipping (visual jump)
- Step 7: Cursor position depends on DOM state which may have shifted

## Target Flow (WASM Authority)

```
1. User types → input.js sends operation to WASM (insert_text_in_paragraph)
2. WASM applies operation → returns affected node IDs
3. Editor calls doc.get_page_html(pageIndex) for each affected page
4. Editor replaces ONLY the affected page DOM content
5. Cursor position provided by WASM (nodeId, charOffset → page, x, y)
6. No DOM measurement, no CSS clipping, no paragraph splitting in JS
```

## Implementation Plan

### Step 1: Add `get_page_html(page_index)` to WASM (New API)

The key missing API. Instead of the editor getting full document HTML and then distributing it across pages, WASM provides ready-to-mount HTML per page.

**What it returns:**
```json
{
  "pageIndex": 0,
  "html": "<p data-node-id='1:5'>First part of paragraph...</p><p data-node-id='1:6'>...</p>",
  "width": 612,
  "height": 792,
  "contentArea": { "x": 72, "y": 72, "width": 468, "height": 648 },
  "headerHtml": "...",
  "footerHtml": "...",
  "splitFragments": [
    { "nodeId": "1:5", "isContinuation": false, "visibleLines": "0-15" },
    { "nodeId": "1:5", "isContinuation": true, "visibleLines": "15-28" }
  ]
}
```

**For split paragraphs:** The HTML for each page already contains only the lines belonging to that page. No clipping needed — WASM renders the fragment directly.

**Implementation location:** `ffi/wasm/src/lib.rs` — uses the existing layout engine's `LayoutDocument` which already has per-page blocks with `split_at_line` and `is_continuation` data.

### Step 2: Replace `renderDocument()` with page-level rendering

**Current:** `renderDocument()` → `doc.to_html()` → clear all pages → repaginate → fixups
**Target:** `renderAffectedPages(affectedPageIndices)` → `doc.get_page_html(i)` per page → mount

```javascript
// New rendering flow
function renderAffectedPages(pageIndices) {
  for (const i of pageIndices) {
    const pageData = doc.get_page_html(i);
    const pageEl = state.pageElements[i];
    const contentEl = pageEl.querySelector('.page-content');
    contentEl.innerHTML = pageData.html;
    // Header/footer from WASM
    setPageHeader(pageEl, pageData.headerHtml);
    setPageFooter(pageEl, pageData.footerHtml);
  }
}
```

**For typing (single paragraph edit):**
1. WASM returns which pages are affected (usually just 1, sometimes 2 near page boundary)
2. Editor re-renders only those pages
3. No full document re-render

**For structural changes (split/merge/table):**
1. WASM returns full page count + affected range
2. Editor adds/removes page divs as needed
3. Re-renders affected pages only

### Step 3: Remove DOM measurement from pagination

**Delete or gut these functions:**
- `domBasedOverflowSplit()` — replaced by WASM fragments
- `applySplitParagraphClipping()` — replaced by WASM fragments
- All `offsetHeight`/`scrollHeight` reads in pagination.js for layout decisions

**Keep DOM measurement ONLY for:**
- Scroll position management (viewport tracking)
- Virtual scroll: which pages are visible (IntersectionObserver)
- Accessibility: screen reader focus management

### Step 4: WASM-driven cursor positioning (future)

**Current:** Cursor uses DOM TreeWalker to find text nodes and set Selection range.
**Target:** WASM provides `getCursorPosition(nodeId, charOffset)` → `{ pageIndex, x, y, height }`

This is the hardest step and can be deferred. The current DOM-based cursor works acceptably once rendering is stable. The main benefit is accuracy near page boundaries and in RTL/complex scripts.

### Step 5: Input path — stop reading DOM, send operations directly

**Current typing flow:**
```
DOM input event → getEditableText(el) → diff against cache → replace_text()
```

**Target typing flow:**
```
DOM input event → compute what changed (beforeinput data) → insert_text_in_paragraph(nodeId, offset, text)
```

The `beforeinput` event provides `inputType` and `data` directly:
- `insertText` + `data:"a"` → `doc.insert_text_in_paragraph(nodeId, cursorOffset, "a")`
- `deleteContentBackward` → `doc.delete_text_in_paragraph(nodeId, cursorOffset-1, 1)`
- `insertParagraph` → `doc.split_paragraph(nodeId, cursorOffset)`

This eliminates the `syncParagraphText` → `getEditableText` → diff → `replace_text` chain entirely. No DOM text reading. No cache management. No diff computation. Just direct WASM operations from input events.

**Caveat:** IME composition needs special handling — buffer composition events and only send the final result.

## Migration Strategy

This is a large change. Do it in stages:

### Stage A: Add `get_page_html()` API (Rust/WASM only, no editor changes)
- Implement in `ffi/wasm/src/lib.rs`
- Uses existing `LayoutDocument` per-page blocks
- Add tests comparing output against current `to_html()` + page map

### Stage B: Wire new rendering path alongside old one
- Add `renderPageFromWasm(pageIndex)` in render.js
- Use it for `renderNodeById` calls (single paragraph edits)
- Keep `renderDocument()` as fallback for structural changes
- A/B test: compare visual output between old and new paths

### Stage C: Replace full render with page-level render
- `renderDocument()` calls `renderAffectedPages([0..pageCount])` instead of to_html + repaginate
- Remove DOM measurement from pagination
- Remove CSS clipping functions

### Stage D: Direct input → WASM operations
- Replace `syncParagraphText` with `beforeinput` event handling
- Remove `syncedTextCache` entirely
- Remove `getEditableText` for typing (keep for copy/paste)

### Stage E: Remove legacy code
- Delete `domBasedOverflowSplit()`
- Delete `applySplitParagraphClipping()`
- Delete `syncParagraphText()` DOM reading path
- Delete `syncedTextCache`

## Files Affected

| File | Change |
|------|--------|
| `ffi/wasm/src/lib.rs` | Add `get_page_html()`, `get_affected_pages()` APIs |
| `crates/s1-layout/src/html.rs` | Add per-page HTML rendering (already has per-page block data) |
| `editor/src/render.js` | New `renderAffectedPages()`, deprecate `renderDocument()` |
| `editor/src/pagination.js` | Remove DOM measurement, CSS clipping |
| `editor/src/input.js` | `beforeinput` → direct WASM ops (Stage D) |
| `editor/src/selection.js` | Cursor from WASM position data (Stage E) |

## Success Criteria

- [ ] Long paragraph spanning 2 pages: no flicker, no duplicate text, no jump
- [ ] Typing in 100-page doc: < 16ms render time (single page update)
- [ ] Structural change (Enter key): only affected pages re-render
- [ ] Page break identical between editor view and PDF export
- [ ] No `offsetHeight`/`scrollHeight` in pagination.js layout path
- [ ] No CSS `overflow:hidden` / negative margins for split paragraphs
