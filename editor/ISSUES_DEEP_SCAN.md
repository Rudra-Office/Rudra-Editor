# Folio Editor — Deep Scan Issue Tracker

> **Last Updated**: 2026-03-15
> **PO**: Claude (AI)
> **Total Issues Found**: 61 (full audit)
> **Actionable**: 49
> **False Positives**: 12 (verified as already implemented)
> **Resolved**: 49 / 49

---

## Phase 1: Critical / High Priority (Sessions 1-2)

### 1A. Collaboration Broadcasting — All operations must broadcast to peers

| # | Issue | Severity | Status | Files | Resolution |
|---|---|---|---|---|---|
| C-01 | Alignment changes not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'setAlignment', nodeId, alignment })` in `applyAlignment()` after `state.doc.set_alignment()` call |
| C-02 | List format changes not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'setListFormat', nodeId, format, level: 0 })` in `toggleList()` after `state.doc.set_list_format()` call |
| C-03 | Line spacing changes not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'setLineSpacing', nodeId, value })` in line spacing dropdown change handler |
| C-04 | Indent changes not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'setIndent', nodeId, side: 'left', value: newVal })` in `applyIndent()` after `state.doc.set_indent()` |
| C-05 | Heading level (style gallery) not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'setHeading', nodeId, level })` in style gallery click handler after `state.doc.set_heading_level()` |
| C-06 | Table insert not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'insertTable', afterNodeId, rows, cols })` after `state.doc.insert_table()` in table modal submit handler |
| C-07 | HR insert not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'insertHR', afterNodeId })` after `state.doc.insert_horizontal_rule()` |
| C-08 | Page break insert not broadcasting | High | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'insertPageBreak', afterNodeId })` after `state.doc.insert_page_break()` |
| C-09 | Table row/col insert/delete not broadcasting (6 ops) | High | DONE | toolbar-handlers.js | Added `broadcastOp` for all 6 table context menu operations: `insertTableRow` (above/below), `deleteTableRow`, `insertTableColumn` (left/right), `deleteTableColumn` — each with `tableId` and `index` |
| C-10 | Image insert not broadcasting | High | DONE | images.js | Added `broadcastOp({ action: 'insertImage', afterNodeId })` in `insertImage()` after `state.doc.insert_image()` call |
| C-11 | Image delete not broadcasting | High | DONE | images.js | Added `broadcastOp({ action: 'deleteNode', nodeId: imgNodeId })` in `deleteSelectedImage()` after `state.doc.delete_image()` |
| C-12 | Image drag/move not broadcasting | High | DONE | images.js | Added `broadcastOp({ action: 'moveNodeBefore'/'moveNodeAfter', nodeId, beforeId/afterId })` in drag-drop end handler |
| C-13 | Cut (Ctrl+X) not broadcasting delete | High | DONE | input.js | Added `broadcastOp({ action: 'deleteSelection', startNode, startOffset, endNode, endOffset })` in `doCut()` after clipboard write |
| C-14 | Clear formatting not broadcasting | Medium | DONE | toolbar-handlers.js | Added loop broadcasting `broadcastOp({ action: 'formatSelection', ..., key: k, value: 'false' })` for each formatting key (bold, italic, underline, strikethrough, superscript, subscript) in clear formatting handler |
| C-15 | Image resize not broadcasting | High | DONE | images.js | Added `broadcastOp({ action: 'resizeImage', nodeId, width: wPt, height: hPt })` in `deselectImage()` after `state.doc.resize_image()` |
| C-16 | Cell background not broadcasting | Medium | DONE | toolbar-handlers.js | Added `broadcastOp({ action: 'setCellBackground', cellId, color })` after `state.doc.set_cell_background()` in color picker change handler |

### 1B. Collaboration Remote Handlers — Peers must handle received operations

| # | Issue | Severity | Status | Files | Resolution |
|---|---|---|---|---|---|
| R-01 | Missing handler: `setListFormat` | High | DONE | collab.js | Added switch case `'setListFormat'` in `applyRemoteOp()` — calls `state.doc.set_list_format(op.nodeId, op.format, op.level \|\| 0)` then `renderDocument()` |
| R-02 | Missing handler: `setIndent` | High | DONE | collab.js | Added switch case `'setIndent'` — calls `state.doc.set_indent(op.nodeId, op.side, op.value)` then `renderDocument()` |
| R-03 | Missing handler: `setLineSpacing` | High | DONE | collab.js | Added switch case `'setLineSpacing'` — calls `state.doc.set_line_spacing(op.nodeId, op.value)` then `renderDocument()` |
| R-04 | Missing handler: `insertTable` | High | DONE | collab.js | Added switch case `'insertTable'` — calls `state.doc.insert_table(op.afterNodeId, op.rows, op.cols)` then `renderDocument()` |
| R-05 | Missing handler: `insertTableRow` | High | DONE | collab.js | Added switch case `'insertTableRow'` — calls `state.doc.insert_table_row(op.tableId, op.index)` then `renderDocument()` |
| R-06 | Missing handler: `deleteTableRow` | High | DONE | collab.js | Added switch case `'deleteTableRow'` — calls `state.doc.delete_table_row(op.tableId, op.index)` then `renderDocument()` |
| R-07 | Missing handler: `insertTableColumn` | High | DONE | collab.js | Added switch case `'insertTableColumn'` — calls `state.doc.insert_table_column(op.tableId, op.index)` then `renderDocument()` |
| R-08 | Missing handler: `deleteTableColumn` | High | DONE | collab.js | Added switch case `'deleteTableColumn'` — calls `state.doc.delete_table_column(op.tableId, op.index)` then `renderDocument()` |
| R-09 | Missing handler: `insertHR` | High | DONE | collab.js | Added switch case `'insertHR'` — calls `state.doc.insert_horizontal_rule(op.afterNodeId)` then `renderDocument()` |
| R-10 | Missing handler: `insertPageBreak` | High | DONE | collab.js | Added switch case `'insertPageBreak'` — calls `state.doc.insert_page_break(op.afterNodeId)` then `renderDocument()` |
| R-11 | Missing handler: `insertImage` | High | DONE | collab.js | Added switch case `'insertImage'` — image binary synced via full document sync; handler calls `renderDocument()` |
| R-12 | Missing handler: `moveNodeBefore` | High | DONE | collab.js | Added switch case `'moveNodeBefore'` — calls node move before API then `renderDocument()` |
| R-13 | Missing handler: `moveNodeAfter` | High | DONE | collab.js | Added switch case `'moveNodeAfter'` — calls node move after API then `renderDocument()` |
| R-14 | Missing handler: `resizeImage` | High | DONE | collab.js | Added switch case `'resizeImage'` — calls `state.doc.resize_image(op.nodeId, op.width, op.height)` then `renderDocument()` |
| R-15 | Missing handler: `setCellBackground` | Medium | DONE | collab.js | Added switch case `'setCellBackground'` — calls `state.doc.set_cell_background(op.cellId, op.color)` then `renderDocument()` |
| R-16 | Missing handler: `setImageAltText` | Medium | DONE | collab.js | Added switch case `'setImageAltText'` — calls `state.doc.set_image_alt_text(op.nodeId, op.alt)` then `renderDocument()` |

---

## Phase 2: Header/Footer & Rendering

| # | Issue | Severity | Status | Files | Resolution |
|---|---|---|---|---|---|
| H-01 | Headers/footers not rendering (docs without explicit H/F) | Critical | DONE | pagination.js | `updatePageBreaks()` now always renders header area at top (`page.prepend(topHdr)`) and footer at bottom (`page.appendChild(btmFtr)`) even when `headerHtml`/`footerHtml` are empty — provides page margin visualization. Footer defaults to centered page number when no footer HTML exists. |
| H-02 | `docHeaderHtml`/`docFooterHtml` not initialized in state | Medium | DONE | state.js | Added `docHeaderHtml: ''` and `docFooterHtml: ''` to central state object initialization, preventing undefined references in pagination.js |
| H-03 | Header/footer extraction from WASM `to_html()` | High | DONE | render.js | In `renderDocument()`, after WASM `to_html()` call, queries rendered HTML for `.doc-header`/`.doc-footer` elements, extracts their `innerHTML` into `state.docHeaderHtml`/`state.docFooterHtml`, then removes those elements from the page (pagination system re-renders them per page break with page number substitution) |

---

## Phase 3: Paste & Input Fixes

| # | Issue | Severity | Status | Files | Resolution |
|---|---|---|---|---|---|
| P-01 | Paste after delete-all broken (stale `lastSelInfo`) | Critical | DONE | input.js | After delete-all + re-render, `state.lastSelInfo` pointed at non-existent DOM nodes. Fixed by setting `state.lastSelInfo = null` after delete, then re-querying the DOM for the first available `[data-node-id]` paragraph before pasting via `paste_plain_text()` |
| P-02 | Backspace at start not skipping `.editor-header` | Medium | DONE | input.js | Backspace merge logic now skips `.page-break`, `.editor-footer`, and `.editor-header` elements when walking `previousElementSibling` to find the real previous paragraph for `merge_paragraphs()` |
| P-03 | Delete at end not skipping `.editor-header` | Medium | DONE | input.js | Delete-at-end merge logic now skips `.page-break`, `.editor-footer`, and `.editor-header` elements when walking `nextElementSibling` to find the real next paragraph for `merge_paragraphs()` |

---

## Phase 4: UI/UX Polish (Session 3)

| # | Issue | Severity | Status | Files | Resolution |
|---|---|---|---|---|---|
| U-01 | Comment insert uses browser `prompt()` | Medium | DONE | toolbar-handlers.js, index.html | Replaced two `prompt()` calls with `commentModal` overlay. Modal has textarea for comment text, input for author name (defaults to "User"). Stashes selection info in `state._commentSelInfo` before showing. Supports Enter to insert (Ctrl+Enter for newline), Escape to cancel, backdrop click to close. Calls `state.doc.insert_comment(startNodeId, endNodeId, author, text)` on submit. |
| U-02 | Hyperlink insert uses browser `prompt()` | Medium | DONE | toolbar-handlers.js, index.html | Replaced `prompt('Enter URL')` with `linkModal` overlay. URL input with `type="url"` for browser validation. Stashes selection in `state._linkSelInfo`. Validates URL format before insert (red border on invalid). Calls `applyFormat('hyperlinkUrl', url)` on submit. Enter/Escape/backdrop-click handlers. |
| U-03 | Cell background uses browser `prompt()` | Medium | DONE | toolbar-handlers.js, index.html | Replaced `prompt('Enter hex color')` with hidden `<input type="color">` element inside the table context menu item. Click triggers native color picker. On change, calls `state.doc.set_cell_background(cellId, hex)` and broadcasts. |
| U-04 | Image context menu missing | High | DONE | images.js, index.html, main.js | Added `imageContextMenu` div in HTML with 5 items: Align Left, Align Center, Align Right, Alt Text, Delete. New `initImageContextMenu()` function in images.js handles right-click on images (excludes table cells to avoid conflict with table context menu). Align buttons call `state.doc.set_alignment()` + broadcast. Alt text opens `altTextModal`. Delete calls `deleteSelectedImage()`. Called from `main.js` boot sequence. |
| U-05 | Find button (`btnFind`) not wired to handler | Medium | DONE | toolbar-handlers.js | Added `$('btnFind').addEventListener('click', ...)` that opens the find bar (`$('findBar').classList.add('show')`) and focuses the find input field |
| U-06 | Document dirty indicator missing | Medium | DONE | file.js, input.js, styles.css | New `markDirty()` export sets `state.dirty = true` and calls `updateDirtyIndicator()`. New `updateDirtyIndicator()` export toggles CSS class `doc-dirty` on document name input. CSS adds a small gray dot via SVG `background-image` before the document name. Called from all edit paths in input.js. Auto-save clears dirty flag. |
| U-07 | Status bar not showing word/char/para/page count | Low | DONE | file.js, render.js, pagination.js | New `updateStatusBar()` export in file.js. Uses `requestAnimationFrame` debounce. Counts words (`doc.to_plain_text().split(/\s+/)`), characters, paragraphs (DOM `[data-node-id]` elements), pages (`.page-break` count + 1). Displays as "X words · Y characters · Z paragraphs · N pages" in `#statusInfo`. Respects `_userMsg` flag for transient messages like "Auto-saved". Called after render, pagination, and input events. |
| U-08 | Duplicate `updateStatusBar` in pagination.js | Low | DONE | pagination.js | Removed local `updateStatusBar(numPages)` function from pagination.js. Now imports `updateStatusBar as _updateStatus` from file.js and calls `_updateStatus()` at end of `updatePageBreaks()`. Eliminates conflicting status bar logic. |
| U-09 | Escape key doesn't close new modals | Low | DONE | input.js | Extended global `keydown` Escape handler to close `commentModal`, `linkModal`, and `altTextModal` by removing their `show` class. Also closes slash menu, find bar, export/insert menus, and side panels. |
| U-10 | Modal input/textarea focus styling missing | Low | DONE | styles.css | Added `.modal input[type=text]:focus, .modal input[type=url]:focus, .modal textarea:focus { border-color: var(--accent) }` for visible focus indicator on all modal form fields |
| U-11 | Alt text modal for images | Medium | DONE | images.js, index.html | Added `altTextModal` overlay in HTML with text input and Save/Cancel buttons. In images.js, `initImageContextMenu()` wires the Alt Text context menu item to open the modal, pre-fills current alt text from the image's `alt` attribute. Save calls `state.doc.set_image_alt_text(nodeId, alt)` + `broadcastOp({ action: 'setImageAltText', nodeId, alt })`. Escape/backdrop-click to close. |

---

## Audit False Positives (verified as already implemented)

These were flagged by the initial scan but verified to already be working:

| # | Item | Status | Verification |
|---|---|---|---|
| FP-01 | `refreshComments()` | Already implemented | toolbar-handlers.js — full comment panel refresh with threaded replies |
| FP-02 | `refreshHistory()` | Already implemented | toolbar-handlers.js — version history panel with restore/label |
| FP-03 | `initTableContextMenu()` | Already implemented | toolbar-handlers.js — right-click context menu on table cells with row/col operations |
| FP-04 | `.tc-popup` CSS | Already exists | styles.css — track changes accept/reject popup styling |
| FP-05 | `.slash-menu` CSS | Already exists | styles.css — slash command palette dropdown styling |
| FP-06 | `.peer-cursor` CSS | Already exists | styles.css — collaboration peer cursor caret + label styling |
| FP-07 | `.vs-placeholder` CSS | Already exists | styles.css — virtual scrolling placeholder block styling |
| FP-08 | `.find-highlight` CSS | Already exists | styles.css — yellow highlight for find matches |
| FP-09 | Track changes popup | Already implemented | render.js — click on tracked change shows Accept/Reject popup with data-tc-node-id |
| FP-10 | Virtual scrolling | Already implemented | render.js — IntersectionObserver-based, 20-block buffer, placeholder divs |
| FP-11 | Version history | Already implemented | file.js (IndexedDB storage, 5-min timer) + toolbar-handlers.js (history panel UI) |
| FP-12 | Comment threading | Already implemented | toolbar-handlers.js — reply button, inline reply form, indented display, reply deletion |

---

## Remaining Known Limitations (Not bugs — future work)

| # | Item | Priority | Notes |
|---|---|---|---|
| L-01 | Rich HTML paste | Low | Only plain text paste works; HTML paste strips to text. Would require WASM HTML import API. |
| L-02 | Floating image positioning | Low | WASM has anchor positioning but editor doesn't expose it. Images are inline only. |
| L-03 | Image crop tool | Low | No crop UI; resize only. |
| L-04 | Table cell merge UI | Low | WASM `merge_cells()` exists but no UI to select cell ranges. |
| L-05 | Collaborative cursor rendering | Low | Cursor position broadcasts exist but visual rendering is basic. |
| L-06 | Offline collab queue | Low | Offline buffer exists but untested in production scenarios. |

---

## Summary

| Category | Count | Status |
|---|---|---|
| Collaboration Broadcasting (C-01 to C-16) | 16 | All DONE |
| Collaboration Remote Handlers (R-01 to R-16) | 16 | All DONE |
| Header/Footer & Rendering (H-01 to H-03) | 3 | All DONE |
| Paste & Input Fixes (P-01 to P-03) | 3 | All DONE |
| UI/UX Polish (U-01 to U-11) | 11 | All DONE |
| **Total Actionable** | **49** | **49 / 49 DONE** |
| False Positives (FP-01 to FP-12) | 12 | Verified working |
| Known Limitations (L-01 to L-06) | 6 | Future work |
| **Grand Total Tracked** | **67** | — |

---

## Verification

- **Vite build**: Clean, 0 errors, 0 warnings
- **Rust tests**: 1,079+ tests all passing
- **No `document.execCommand()`** calls in the codebase
- **All operations broadcast** to collaboration peers via `broadcastOp()`
- **All remote operations handled** in `applyRemoteOp()` switch cases in collab.js
- **No browser `prompt()`/`alert()` for user input** — all use modal dialogs or native pickers
