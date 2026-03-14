# s1engine Editor — Full UI/UX Roadmap

> **Created**: 2026-03-15
> **Scope**: Editor UI/UX from current state to production-grade online editor
> **Current State**: 80+ WASM API methods, ~13 JS modules, 47 Playwright tests, Google Docs-inspired UI
> **Target**: Feature-competitive with Google Docs / OnlyOffice for core editing workflows

---

## Phase Overview

### Editor UI/UX Phases (E1-E10)
```
Phase E1: Core Editing Fixes         [P0/P1 bugs that break basic editing]        6 milestones
Phase E2: Selection & Clipboard      [Rich copy/paste, selection polish]           4 milestones
Phase E3: Transaction & Undo UX      [Batched typing, undo groups, history]       4 milestones
Phase E4: Table & Image UX           [Tab navigation, cell editing, image tools]  5 milestones
Phase E5: Collaboration UI           [Share dialog, presence, conflict]           5 milestones
Phase E6: Responsive & Mobile        [Touch support, responsive toolbar]          4 milestones
Phase E7: Accessibility              [WCAG 2.1 AA, keyboard nav, screen readers] 4 milestones
Phase E8: Performance                [Virtual scroll, incremental render]         4 milestones
Phase E9: Advanced Features          [Spell check, templates, equation, shapes]   6 milestones
Phase E10: Polish & Production       [Dark mode, zoom, print preview, deploy]     5 milestones
```

### Engine Gap Phases (F1-F5) — from Deep Scan
```
Phase F1: Text Rendering & Scripts   [BiDi, complex scripts, hyphenation]         3 milestones
Phase F2: Advanced Layout            [Floating images, nested tables, footnotes]   5 milestones
Phase F3: DOCX Fidelity (70%→90%)    [Equations, charts, advanced track changes]  4 milestones
Phase F4: Font & Rendering Accuracy  [Font metrics, fallback chain, canvas mode]  3 milestones
Phase F5: Collaboration Hardening    [CRDT edge cases, autosave safety]           2 milestones
```

**Total: 64 milestones across 15 phases**

---

## Phase E1: Core Editing Fixes

> **Goal**: Fix all P0/P1 bugs that break basic editing workflows.
> **Priority**: CRITICAL — must complete before any other phase.
> **Estimated Milestones**: 8
> **Status**: **7/8 COMPLETE**

### E1.1 — Collapsed Cursor Formatting (P0) ✅ DONE

**Issue**: E-01 — When cursor is collapsed (no selection), clicking Bold formats the entire paragraph instead of setting a "pending format" for the next typed character.

**Implementation**: `pendingFormats` state in `state.js`, `applyFormat()` in `toolbar.js` stores to pending if collapsed, `beforeinput`/`input` handlers in `input.js` apply pending formats to newly typed characters, `selectionchange` clears on cursor move.

---

### E1.2 — UTF-16/Codepoint Selection Mapping (P1) ✅ DONE

**Issue**: E-03 — Selection offset counting must use codepoints consistently.

**Implementation**: All offset calculations in `selection.js` use `Array.from()` for codepoint counting. Verified consistent across `countCharsToPoint()`, `setSelectionRange()`, `getCursorOffset()`, `isCursorAtStart()`, `isCursorAtEnd()`.

---

### E1.3 — Incremental Paragraph Re-render (P1) ✅ DONE

**Issue**: E-04 — Full `renderDocument()` on every change loses cursor/scroll.

**Implementation**: `renderNodeById()` and `renderNodesById()` in `render.js` — re-query DOM fresh before each replacement (E-05 fix), batch multiple nodes in single pass. Format operations use batch render instead of full document re-render.

---

### E1.4 — Superscript/Subscript Exclusion (P1) ✅ DONE

**Issue**: E-06 — Toggling superscript then subscript could overlap render cycles.

**Implementation**: `applyFormatPair()` in `toolbar.js` — applies both format operations (clear opposite + set target) before a single batch re-render. Used for super↔sub mutual exclusion.

---

### E1.5 — Find Highlights Sync (P1) ✅ DONE

**Issue**: E-09 — Find highlights are static DOM overlays. Editing text doesn't update highlight positions.

**Implementation**: `refreshFindIfOpen()` in `find.js` — debounced (300ms) re-search when find bar is open. Registered as `state._onTextChanged` callback, called from `debouncedSync()` and `renderDocument()` in `render.js`. Maintains closest match index across re-searches.

---

### E1.6 — Page Break / HR Deletion Guard (P1) ✅ DONE

**Issue**: E-22 — Page break and HR divs are contenteditable=false but can be selected and deleted via keyboard.

**Implementation**: Guard in `input.js` keydown handler — intercepts Delete/Backspace when cursor is on a page-break, HR, editor-header, or editor-footer element. Prevents default browser deletion. Existing code already skips over these elements when merging paragraphs.

---

### E1.7 — Page Boundary Visuals (P1)

**Issue**: Page height grows with content instead of showing fixed-height paper sheets separated by visible gaps (like Google Docs).

**Deliverables**:
- [x] Improve page-break CSS — white footer/header areas with shadows, 24px gray gap between pages
- [ ] Add page-bottom padding to simulate margin before page break
- [ ] Ensure consistent page height visual between page breaks (1056px for Letter)
- [ ] Tests: visual verification of page boundaries
- [ ] Visual feedback: highlight page break div when cursor is adjacent
- [ ] Tests: 1 Playwright test (Backspace before page break doesn't delete it)

---

## Phase E2: Selection & Clipboard

> **Goal**: Rich copy/paste with formatting preservation, clipboard interop with other editors.
> **Priority**: HIGH — core editing workflow.
> **Estimated Milestones**: 4

### E2.1 — Rich Copy (HTML Clipboard)

**Deliverables**:
- [ ] On `copy` event: extract selection range from WASM model, generate semantic HTML (not the absolutely-positioned layout HTML)
- [ ] Add WASM method `export_selection_html(start_id, start_off, end_id, end_off) → String` that produces clean `<p><span style="...">` HTML
- [ ] Set both `text/plain` and `text/html` on clipboard via `ClipboardItem`
- [ ] Preserve: bold, italic, underline, strikethrough, font size, font family, color, highlight, alignment, lists, hyperlinks
- [ ] Tests: 2 Playwright tests (copy formatted text, paste into external editor)

### E2.2 — Rich Paste (HTML → WASM Model)

**Deliverables**:
- [ ] On `paste` event: check for `text/html` clipboard data
- [ ] Parse HTML into WASM operations — map `<b>` → bold, `<i>` → italic, `<span style="...">` → attributes
- [ ] Add WASM method `paste_html(html: &str, target_id: &str, offset: u32)` that creates runs with formatting
- [ ] Fallback: if HTML parsing fails, fall back to plain text paste via `paste_plain_text()`
- [ ] Handle paste from Google Docs, Word Online, LibreOffice (each has different HTML format)
- [ ] "Paste Special" menu option: paste as plain text, paste as HTML, paste and match formatting
- [ ] Tests: 3 Playwright tests (paste bold from clipboard, paste from Word HTML, paste special plain text)

### E2.3 — Cut Operation

**Deliverables**:
- [ ] On `cut` event: copy selection to clipboard (E2.1), then delete selection via `delete_selection()`
- [ ] Single undo unit: cut should be undoable in one step
- [ ] Tests: 1 Playwright test (cut text, undo restores)

### E2.4 — Drag & Drop Text

**Deliverables**:
- [ ] Support text drag-drop within the editor (select text, drag to new position)
- [ ] On `dragstart`: save selection range and formatted content
- [ ] On `drop`: delete from original position, insert at drop position, all in single transaction
- [ ] Visual feedback: show drop cursor indicator during drag
- [ ] Tests: 1 Playwright test (drag text between paragraphs)

---

## Phase E3: Transaction & Undo UX

> **Goal**: Batch typing into undo groups, show undo history, smart transaction labels.
> **Priority**: HIGH — undo behavior is core UX.
> **Estimated Milestones**: 4

### E3.1 — Typing Transaction Batching ✅ DONE

**Issue**: E-13 — Each character typed = separate WASM operation. Undo undoes one character at a time.

**Implementation**: JS-level batch undo via `state._typingBatch` tracking. `syncParagraphText()` increments batch counter for continuous typing in same paragraph. 500ms timer resets batch on pause. `doUndo()` in `input.js` undoes all batch steps at once. Batch cleared on Enter, format change, or structural operation.

### E3.2 — Undo/Redo History Viewer

**Deliverables**:
- [ ] Add "History" panel in sidebar (toggle via Edit menu → "Show Undo History")
- [ ] List recent transactions with labels: "Typed 'hello world'", "Applied bold", "Inserted table 3×3", "Deleted paragraph"
- [ ] Click entry to jump to that state (undo/redo multiple steps)
- [ ] Highlight affected text/node when hovering over history entry
- [ ] Show timestamp for each entry
- [ ] Cap at 100 visible entries

### E3.3 — Version History UI

**Deliverables**:
- [ ] Complete the version history panel (currently stubbed)
- [ ] Show list of auto-saved versions with timestamps and word counts
- [ ] "Restore" button for each version
- [ ] Side-by-side diff view (original vs selected version) using word-level diff
- [ ] "Name this version" — manually label a version snapshot
- [ ] Tests: 1 Playwright test (save, edit, restore previous version)

### E3.4 — Smart Transaction Labels

**Deliverables**:
- [ ] Auto-generate descriptive labels for all operations:
  - Typing: "Typed 'hello'" / "Typed 12 characters"
  - Formatting: "Applied bold to 'selected text'"
  - Structure: "Split paragraph", "Merged paragraphs", "Inserted table 3×2"
  - Find/Replace: "Replaced 'foo' with 'bar' (5 occurrences)"
- [ ] Show label in undo/redo tooltip: hover over undo button → "Undo: Applied bold"
- [ ] Show label in status bar after undo: "Undid: Applied bold"

---

## Phase E4: Table & Image UX

> **Goal**: Table Tab navigation, image crop/rotate, cell formatting, context menus.
> **Priority**: MEDIUM — table/image editing is expected in any editor.
> **Estimated Milestones**: 5

### E4.1 — Table Tab Navigation

**Deliverables**:
- [ ] Tab key in table cell → move to next cell (left to right, top to bottom)
- [ ] Shift+Tab → move to previous cell
- [ ] Tab in last cell → insert new row and move to first cell of new row
- [ ] Arrow keys: move between cells when at cell boundary
- [ ] Enter in cell: insert line break within cell (not new row)
- [ ] Tests: 2 Playwright tests (Tab traversal, Shift+Tab reverse, Tab in last cell adds row)

### E4.2 — Cell Formatting

**Deliverables**:
- [ ] Select entire cell content on cell click (triple-click selects cell)
- [ ] Apply formatting to entire cell (bold, alignment, color, background)
- [ ] Cell vertical alignment (top, middle, bottom) via context menu
- [ ] Cell padding adjustment via context menu
- [ ] Multi-cell selection: click+drag across cells → format all selected cells
- [ ] Tests: 1 Playwright test (select cell, apply background color)

### E4.3 — Table Properties Dialog

**Deliverables**:
- [ ] "Table Properties" in context menu → modal dialog
- [ ] Table width (auto, fixed percentage, fixed points)
- [ ] Column width adjustment (drag column borders)
- [ ] Table borders (style, color, width) — all/outer/inner presets
- [ ] Table alignment (left, center, right)
- [ ] Row height (auto, fixed)
- [ ] Header row toggle (repeat on every page)

### E4.4 — Image Editing

**Deliverables**:
- [ ] Resize handles: 8 points (corners + midpoints) with aspect ratio lock (Shift)
- [ ] Crop mode: click crop icon → drag handles to crop visible area
- [ ] Rotate: 90° CW/CCW buttons in toolbar, free rotation handle
- [ ] Wrap text: inline, wrap-tight, behind-text, in-front-of-text (via context menu)
- [ ] Image border: add/remove border (style, color, width)
- [ ] Replace image: right-click → "Replace Image" → file picker

### E4.5 — Image Alt Text & Caption

**Deliverables**:
- [ ] Alt text dialog (already exists) — enhance with character count, preview
- [ ] Add "Caption" below image — auto-numbered ("Figure 1:", "Figure 2:")
- [ ] Caption editing: click caption text to edit inline
- [ ] Accessibility warning if alt text is empty (yellow border indicator)

---

## Phase E5: Collaboration UI

> **Goal**: Working collaboration with share dialog, presence indicators, conflict resolution.
> **Priority**: MEDIUM — differentiating feature.
> **Estimated Milestones**: 5

### E5.1 — Share Dialog

**Deliverables**:
- [ ] "Share" button → modal dialog
- [ ] Generate shareable URL (document ID + access token)
- [ ] Permission levels: View only, Comment, Edit
- [ ] Copy link button
- [ ] QR code for mobile sharing
- [ ] Show currently connected peers (avatar, name, cursor color)

### E5.2 — Presence Indicators

**Deliverables**:
- [ ] Show peer cursors in document: colored vertical lines with name labels
- [ ] Cursor label shows peer name + color (fade after 3s of inactivity)
- [ ] Show peer selection ranges as colored highlights (semi-transparent)
- [ ] Avatar list in toolbar: show connected peers with colored dots
- [ ] "User is editing..." indicator when peer is typing in same paragraph
- [ ] Smooth cursor position interpolation (no jumping)

### E5.3 — Conflict Resolution UI

**Deliverables**:
- [ ] Visual indicator when concurrent edit detected (subtle flash on affected paragraph)
- [ ] Status bar: "Synced" / "Syncing..." / "Offline (3 pending changes)"
- [ ] "Changes applied" toast notification when remote changes arrive
- [ ] If text cursor displaced by remote edit, smoothly animate to new position
- [ ] Undo only affects local operations (already implemented in CRDT — surface in UI)

### E5.4 — Comments & Suggestions

**Deliverables**:
- [ ] Comment sidebar: threaded replies (add reply to existing comment)
- [ ] Resolve/unresolve comments
- [ ] "Suggestion" mode: edits shown as tracked changes, owner can accept/reject
- [ ] @mention peers in comments (shows notification)
- [ ] Comment selection highlighting in document (click comment → scroll to highlighted text)
- [ ] Keyboard shortcut: Ctrl+Alt+M to insert comment

### E5.5 — Collaboration Transport

**Deliverables**:
- [ ] Production WebSocket relay server (Cloudflare Workers Durable Objects or self-hosted)
- [ ] Room-based document sessions (join by document ID)
- [ ] Authentication integration (JWT tokens for peer identity)
- [ ] Persistence: save CRDT state to server (not just client IndexedDB)
- [ ] Reconnect with state reconciliation (merge offline changes)
- [ ] Rate limiting and abuse prevention

---

## Phase E6: Responsive & Mobile

> **Goal**: Usable on tablets and phones. Responsive toolbar, touch gestures.
> **Priority**: MEDIUM — 37%+ of users are on mobile/tablet.
> **Estimated Milestones**: 4

### E6.1 — Responsive Toolbar

**Deliverables**:
- [ ] Breakpoint at 1024px: collapse toolbar groups into overflow menu (⋯)
- [ ] Breakpoint at 768px: toolbar becomes a floating bottom bar with essential buttons only
- [ ] Breakpoint at 480px: minimal toolbar (bold, italic, undo, more menu)
- [ ] Toolbar buttons: increase touch target to 44px (WCAG 2.5.5)
- [ ] Swipe left/right on toolbar to reveal more buttons

### E6.2 — Touch Selection

**Deliverables**:
- [ ] Implement touch selection handles (blue circles at selection start/end)
- [ ] Drag handles to adjust selection
- [ ] Double-tap to select word, triple-tap for paragraph
- [ ] Long-press for context menu (cut, copy, paste, format)
- [ ] Pinch-to-zoom on document canvas

### E6.3 — Mobile Keyboard

**Deliverables**:
- [ ] Detect software keyboard open/close, resize editor viewport
- [ ] Floating formatting bar above keyboard (bold, italic, indent, heading)
- [ ] Scroll cursor into view when keyboard opens
- [ ] Handle IME composition events (CJK input methods)

### E6.4 — Responsive Layout

**Deliverables**:
- [ ] Editor canvas: fluid width on mobile (no fixed 816px page width)
- [ ] Optional "mobile view" — continuous scroll without page boundaries
- [ ] Sidebar panels (comments, history) → slide-in from bottom on mobile
- [ ] Menus → full-screen slide-up panels on mobile
- [ ] Tests: 2 Playwright tests (mobile viewport toolbar collapse, touch selection)

---

## Phase E7: Accessibility (WCAG 2.1 AA)

> **Goal**: Full keyboard navigation, screen reader support, WCAG 2.1 AA compliance.
> **Priority**: MEDIUM-HIGH — legal requirement in many jurisdictions.
> **Estimated Milestones**: 4

### E7.1 — Keyboard Navigation

**Deliverables**:
- [ ] Menu bar: arrow keys to navigate menus, Enter to select, Escape to close
- [ ] Toolbar: arrow keys to navigate buttons, Enter/Space to activate
- [ ] Dropdown menus: up/down arrows, type-ahead search
- [ ] Modals: Tab/Shift+Tab to cycle focusable elements, Escape to close
- [ ] Focus trap in modals and panels (focus doesn't escape to background)
- [ ] Skip links: "Skip to editor content" link at top of page
- [ ] Tab order: logical left-to-right, top-to-bottom

### E7.2 — Screen Reader Support

**Deliverables**:
- [ ] `role="document"` on editor canvas
- [ ] `aria-label` on all toolbar buttons (already partially done)
- [ ] `aria-expanded`, `aria-haspopup` on dropdown buttons
- [ ] `aria-checked` / `aria-pressed` on toggle buttons (bold, italic, etc.)
- [ ] Live region announcements: "Bold applied", "Table inserted 3 by 2", "Page 3 of 12"
- [ ] Heading navigation: screen reader can jump between headings
- [ ] Table navigation announcements: "Row 2, Column 3, cell contains: quarterly revenue"

### E7.3 — Visual Accessibility

**Deliverables**:
- [ ] High contrast mode: detect `prefers-contrast: more`, increase borders and outlines
- [ ] Focus indicators: visible focus ring (3px solid blue) on all interactive elements
- [ ] Color contrast: ensure all text meets 4.5:1 ratio (WCAG AA)
- [ ] Don't rely on color alone: icons + labels for formatting state (not just color toggle)
- [ ] Reduced motion: detect `prefers-reduced-motion`, disable animations

### E7.4 — Keyboard Shortcuts Reference

**Deliverables**:
- [ ] Keyboard shortcuts help dialog (Ctrl+/ or F1)
- [ ] Show all shortcuts grouped by category (editing, formatting, navigation, file)
- [ ] Customizable shortcuts: allow user to rebind keys
- [ ] Save custom bindings to localStorage
- [ ] Print-friendly shortcut reference (PDF export)
- [ ] Shortcut hints on toolbar button tooltips

---

## Phase E8: Performance

> **Goal**: Smooth editing for documents up to 10,000 paragraphs / 100 pages.
> **Priority**: MEDIUM — becomes critical at scale.
> **Estimated Milestones**: 4

### E8.1 — Virtual Scrolling

**Issue**: E-07 — Virtual scroll stub exists but is incomplete. Hidden blocks lose text.

**Deliverables**:
- [ ] Implement proper virtual scroll with IntersectionObserver
- [ ] Render only visible paragraphs + 2-page buffer above and below viewport
- [ ] Maintain accurate scroll height by measuring rendered elements and extrapolating for unrendered
- [ ] When paragraph scrolls into view: call `render_node_html()` to populate
- [ ] When paragraph scrolls out: replace with height-preserving placeholder
- [ ] Handle rapid scroll (skip rendering intermediate, jump to target)
- [ ] Tests: 1 Playwright test (scroll 1000-paragraph document, content renders correctly)

### E8.2 — Incremental DOM Patching

**Deliverables**:
- [ ] Replace `innerHTML` replacement with targeted DOM diffing
- [ ] On text edit: only update the affected text node, not entire paragraph HTML
- [ ] On format change: only update the affected span's style attribute
- [ ] Maintain a node ID → DOM element map for O(1) lookups
- [ ] Use `MutationObserver` to detect external DOM changes (browser spell check corrections)
- [ ] Benchmark: < 16ms per edit on 1000-paragraph document

### E8.3 — Layout Caching

**Deliverables**:
- [ ] Cache `to_paginated_html()` result — only re-layout on structural changes
- [ ] Incremental layout: WASM cache already exists (LayoutCache), surface in JS
- [ ] Lazy page rendering: only layout pages visible in "Pages" view
- [ ] Background layout: use Web Worker for layout computation, don't block main thread

### E8.4 — Memory Optimization

**Deliverables**:
- [ ] Profile WASM memory usage for large documents (1000+ pages)
- [ ] Implement WASM memory pooling / object reuse
- [ ] Release image data for off-screen images (reload on scroll into view)
- [ ] Cap undo history at 500 entries (configurable)
- [ ] Warn user when document exceeds performance thresholds (> 50MB, > 500 pages)

---

## Phase E9: Advanced Features

> **Goal**: Features that differentiate from basic editors.
> **Priority**: LOW-MEDIUM — nice-to-have, adds competitive value.
> **Estimated Milestones**: 6

### E9.1 — Spell Check Integration

**Deliverables**:
- [ ] Browser native spell check (already works via contenteditable `spellcheck="true"`)
- [ ] Custom dictionary: add/remove words (persisted in localStorage)
- [ ] Spell check context menu: right-click misspelled word → suggestions from browser
- [ ] Language detection per paragraph (attribute on paragraph node)
- [ ] Auto-correct common mistakes ("teh" → "the") — configurable list

### E9.2 — Page Templates

**Deliverables**:
- [ ] "New from Template" menu option
- [ ] Built-in templates: Blank, Letter, Resume, Report, Meeting Notes, Essay
- [ ] Each template: predefined styles, headers/footers, section layout, placeholder text
- [ ] Custom templates: save current document as template
- [ ] Template gallery with preview thumbnails

### E9.3 — Equation Editor

**Deliverables**:
- [ ] Insert → Equation (Ctrl+Shift+E)
- [ ] LaTeX input mode: type LaTeX, renders as formatted equation
- [ ] Visual equation builder: fraction, superscript, subscript, integral, sigma, matrix
- [ ] WASM: add `NodeType::Equation` with LaTeX source attribute
- [ ] Render equations via MathJax or KaTeX in browser
- [ ] Round-trip: DOCX `<m:oMath>` ↔ LaTeX ↔ visual rendering

### E9.4 — Drawing / Shape Tools

**Deliverables**:
- [ ] Insert → Shape menu: rectangle, oval, line, arrow, text box
- [ ] Click-drag to draw shape on canvas
- [ ] Shape properties: fill color, stroke color/width, text inside shape
- [ ] Move/resize shapes with handles
- [ ] Leverage existing VML/Drawing support in DOCX writer
- [ ] Group/ungroup shapes

### E9.5 — Table of Contents

**Deliverables**:
- [ ] Insert → Table of Contents (already supported in WASM model)
- [ ] Auto-generate from headings (H1-H6)
- [ ] Click TOC entry → scroll to heading
- [ ] "Update TOC" button after editing headings
- [ ] TOC styling options: dots, page numbers, indentation

### E9.6 — Footnotes & Endnotes

**Deliverables**:
- [ ] Insert footnote (Ctrl+Alt+F): auto-numbered superscript in text, footnote at page bottom
- [ ] Insert endnote (Ctrl+Alt+D): collected at document end
- [ ] Footnote area in paginated view
- [ ] Click footnote marker ↔ jumps to footnote content
- [ ] WASM: add `NodeType::Footnote` / `NodeType::Endnote`
- [ ] DOCX round-trip for footnotes/endnotes

---

## Phase E10: Polish & Production

> **Goal**: Production-ready polish for deployment.
> **Priority**: Required before launch.
> **Estimated Milestones**: 5

### E10.1 — Dark Mode

**Deliverables**:
- [ ] CSS custom properties already defined — add dark theme values
- [ ] Detect `prefers-color-scheme: dark` (system preference)
- [ ] Manual toggle: View → Dark Mode (or sun/moon icon in status bar)
- [ ] Persist preference in localStorage
- [ ] Editor canvas: dark page with light text (or white page on dark background — user choice)
- [ ] Toolbar, menus, panels: dark backgrounds with light text

### E10.2 — Zoom (Partially Done)

**Issue**: E-19 — Zoom state exists but CSS not applied.

**Done**:
- [x] Apply `transform: scale(zoomLevel/100)` to editor canvas with `transform-origin: top center` (toolbar-handlers.js)
- [x] Zoom buttons in status bar (zoomIn/zoomOut, 50%-200% range)
- [x] Ctrl+Plus/Minus/Zero for zoom in/out/reset (input.js)
- [x] Ruler scale tracks zoom level (ruler.js)

**Remaining**:
- [ ] Zoom levels: "Fit Width", "Fit Page" presets
- [ ] Pinch-to-zoom on trackpad/touch
- [ ] Persist zoom level per document
- [ ] Status bar: show current zoom percentage with dropdown

### E10.3 — Print Preview

**Deliverables**:
- [ ] File → Print Preview → full-screen paginated view (from `to_paginated_html()`)
- [ ] Show page margins, headers/footers with actual content
- [ ] "Print" button → `window.print()` with print-optimized CSS
- [ ] Page setup dialog: paper size (Letter, A4, Legal), orientation, margins
- [ ] Live preview: changes in dialog update preview immediately

### E10.4 — Onboarding & Help

**Deliverables**:
- [ ] First-run tutorial: highlight key features (formatting, tables, images, collaboration)
- [ ] Tooltip tours: step-by-step highlighting of toolbar buttons
- [ ] Help menu → link to documentation, keyboard shortcuts, report bug
- [ ] "What's New" dialog on version updates
- [ ] In-app search: "How do I insert a table?" → show relevant toolbar button

### E10.5 — Production Deployment

**Deliverables**:
- [ ] Vite production build: minified JS/CSS, WASM async loading, code splitting
- [ ] CDN deployment configuration (static assets)
- [ ] Service Worker for offline support (cache WASM binary + assets)
- [ ] Error tracking integration (Sentry or equivalent)
- [ ] Analytics: track feature usage (which buttons clicked, export format popularity)
- [ ] CSP headers and security hardening
- [ ] Automated Lighthouse audit (target: 90+ on all categories)

---

## Milestone Priority Matrix

| Phase | Milestones | Priority | Dependency |
|-------|-----------|----------|------------|
| **E1** Core Fixes | 6 | CRITICAL | None |
| **E2** Clipboard | 4 | HIGH | E1.1, E1.2 |
| **E3** Undo UX | 4 | HIGH | E1.3 |
| **E4** Table/Image | 5 | MEDIUM | E1.3, E2.2 |
| **E5** Collaboration | 5 | MEDIUM | E3.1 |
| **E6** Mobile | 4 | MEDIUM | E1.*, E7.1 |
| **E7** Accessibility | 4 | MEDIUM-HIGH | E1.* |
| **E8** Performance | 4 | MEDIUM | E1.3, E3.1 |
| **E9** Advanced | 6 | LOW-MEDIUM | E4.*, E5.* |
| **E10** Polish | 5 | REQUIRED | All above |

---

## Recommended Execution Order

```
Sprint 1 (Week 1-2):   E1.1, E1.2, E1.4, E1.5, E1.6    [Core bug fixes]
Sprint 2 (Week 3-4):   E1.3, E2.1, E2.2, E2.3           [Incremental render + clipboard]
Sprint 3 (Week 5-6):   E3.1, E3.4, E7.1                  [Typing batching + keyboard nav]
Sprint 4 (Week 7-8):   E4.1, E4.2, E10.2                 [Table UX + zoom]
Sprint 5 (Week 9-10):  E8.1, E8.2                         [Virtual scroll + DOM patching]
Sprint 6 (Week 11-12): E5.1, E5.2, E5.3                   [Collaboration UI]
Sprint 7 (Week 13-14): E6.1, E6.2, E6.4                   [Mobile + responsive]
Sprint 8 (Week 15-16): E7.2, E7.3, E10.1                  [Screen reader + dark mode]
Sprint 9 (Week 17-18): E3.2, E3.3, E4.3                   [History viewer + table properties]
Sprint 10 (Week 19-20): E9.1, E9.5, E10.3, E10.5         [Spell check + TOC + print + deploy]
```

---

## Competitive Feature Matrix

| Feature | s1engine Editor | Google Docs | OnlyOffice | LibreOffice Online |
|---------|----------------|-------------|------------|-------------------|
| Rich formatting | Yes | Yes | Yes | Yes |
| Tables | Yes (basic) | Yes (full) | Yes (full) | Yes (full) |
| Images | Yes (basic) | Yes (full) | Yes (full) | Yes (full) |
| Track changes | Yes | Yes | Yes | Yes |
| Comments | Partial | Yes (threaded) | Yes (threaded) | Yes |
| Find/Replace | Yes | Yes (regex) | Yes (regex) | Yes (regex) |
| Real-time collab | Beta | Yes | Yes | Yes |
| Offline editing | Partial | Yes (PWA) | No | No |
| Mobile support | No | Yes | Yes | Limited |
| Spell check | Browser only | Custom | Custom | Hunspell |
| Math equations | No | Yes | Yes | Yes |
| Drawing/shapes | No | Yes | Yes | Yes |
| Templates | No | Yes | Yes | Yes |
| Version history | Partial | Yes | Yes | Limited |
| Print preview | No | Yes | Yes | Yes |
| Dark mode | No | No | No | No |
| CRDT collab engine | Yes (Fugue) | Custom | Custom | None (lock-based) |
| Self-hostable | Yes | No | Yes | Yes |
| Open source | Yes (MIT) | No | AGPL | MPL |

**After completing E1-E10**: s1engine would match ~70-80% of Google Docs features for core document editing, with the unique advantage of being a self-hostable, MIT-licensed SDK with built-in CRDT collaboration.

**After completing E1-E10 + F1-F5 (engine gaps below)**: ~90% feature parity for Western-language document editing.

---

## Engine-Level Gap Phases (from Deep Scan)

> These are **engine/library** fixes needed alongside the UI phases above.
> Without these, the editor cannot handle many real-world documents.

### Current Honest Assessment

| Capability | s1engine | OnlyOffice | Collabora Online |
|---|---|---|---|
| Team size | 1 developer + AI | 100+ engineers, 10+ years | 50+ engineers (LibreOffice fork, 20+ years) |
| DOCX fidelity | ~70% of features | ~95% | ~90% |
| BiDi / RTL | Not implemented | Full UAX#9 | Full UAX#9 |
| Complex scripts | Basic (rustybuzz exists, not wired) | Full HarfBuzz | Full HarfBuzz |
| Floating images | Parsed but not rendered in layout | Full anchoring + wrap | Full anchoring + wrap |
| Multi-column | Not implemented | Full support | Full support |
| Footnotes/Endnotes | Not implemented | Full support | Full support |
| Equations | Not implemented | Full MathML + OMML | Full MathML |
| Nested tables | Silently dropped | Full support | Full support |
| Hyphenation | Not implemented | Dictionary-based | Dictionary-based |
| Track changes | Basic ins/del | Full (moves, cell changes) | Full |
| Comments | Basic | Threaded + resolved | Threaded + resolved |
| Font rendering | Fallback averages in WASM | Canvas-based, accurate | Canvas-based, accurate |
| Spreadsheets | None | Full Excel compat | Full Calc |
| Presentations | None | Full PowerPoint compat | Full Impress |

**What s1engine IS good at:**
- Clean Rust architecture (genuinely well-designed, 60K LOC)
- 1,189 tests, zero panics in library code
- CRDT foundation is solid (Fugue algorithm, 182 tests)
- Pure-Rust stack (no C/C++ deps) — ideal for WASM
- Good for Western-language DOCX/ODT documents
- Self-hostable, MIT-licensed (not AGPL like OnlyOffice)

**Realistic position:** s1engine is a strong document processing SDK — comparable to Aspose or docx-rs, not to a full office suite. Getting to OnlyOffice/Collabora level would require 2-5 years by a team of 5-10 engineers. But reaching **Google Docs-level for basic document editing** (no spreadsheets/presentations) is achievable with the phases below.

---

### Phase F1: Text Rendering & Scripts

> **Goal**: Accurate text rendering for all scripts, not just Latin.
> **Impact**: Unlocks Arabic, Hebrew, Hindi, Thai, CJK markets.
> **Estimated effort**: 4-6 weeks

#### F1.1 — BiDi Layout

- [ ] Wire up `unicode-bidi` crate (already a dependency) in layout engine
- [ ] Implement UAX#9 paragraph-level and run-level bidi resolution
- [ ] Layout engine: handle RTL runs with reversed glyph order
- [ ] HTML renderer: emit `dir="rtl"` on RTL paragraphs, `unicode-bidi: embed` on inline runs
- [ ] DOCX parser: read `w:bidi` paragraph property
- [ ] DOCX writer: write `w:bidi` paragraph property
- [ ] Tests: Arabic text layout, Hebrew mixed with English, number handling in RTL context

#### F1.2 — Complex Script Shaping

- [ ] rustybuzz is already used for shaping — ensure it handles complex scripts (Arabic ligatures, Devanagari conjuncts, Thai clusters)
- [ ] Font fallback: when primary font lacks glyphs, cascade to system fonts with correct script coverage
- [ ] Add script-aware line breaking (e.g., Thai has no spaces — use dictionary-based breaking)
- [ ] Test fixtures: Arabic (مرحبا بالعالم), Hindi (नमस्ते दुनिया), Thai (สวัสดีชาวโลก), CJK (你好世界)

#### F1.3 — Hyphenation

- [ ] Integrate `hyphenation` crate (Knuth-Liang algorithm with TeX patterns)
- [ ] Apply hyphenation during Knuth-Plass line breaking (optional hyphenation points)
- [ ] Language-specific patterns (English, German, French, Spanish — cover top 10 languages)
- [ ] DOCX: read/write `w:suppressAutoHyphens`
- [ ] User toggle: View → Hyphenation On/Off

---

### Phase F2: Advanced Layout

> **Goal**: Floating images, multi-column, nested tables, footnotes.
> **Impact**: Handles 90% of real-world DOCX documents without silent content loss.
> **Estimated effort**: 6-8 weeks

#### F2.1 — Floating Image Layout

- [ ] Parse `wp:anchor` positioning (already done in DOCX parser)
- [ ] Layout engine: position floating images relative to page/paragraph/column
- [ ] Text wrapping: square, tight, through, top-and-bottom, behind-text, in-front-of-text
- [ ] Implement text wrapping polygon calculation (flow text around image bounding box)
- [ ] Z-order handling for overlapping images
- [ ] HTML renderer: use `position: absolute` with correct coordinates for floating images
- [ ] Tests: document with floating images and text wrap

#### F2.2 — Nested Tables

- [ ] Modify `collect_body_blocks()` to recurse into table cells for nested tables
- [ ] Layout engine: recursive table layout (table inside cell → layout inner table first, use height for cell)
- [ ] Cap nesting depth at 5 (prevent stack overflow on malicious documents)
- [ ] DOCX parser already handles nested tables in model — just need layout + rendering
- [ ] Tests: 2-level nested table, 3-level nested table

#### F2.3 — Multi-Column Layout

- [ ] Add `ColumnCount` and `ColumnSpacing` attributes to `SectionProperties`
- [ ] DOCX: parse `w:cols` element (count, space, equalWidth)
- [ ] Layout engine: divide content area into N columns, flow text left-to-right across columns
- [ ] Column breaks: handle `w:type="column"` break
- [ ] HTML renderer: use CSS `column-count` or manual positioning
- [ ] Tests: 2-column layout, 3-column with different widths

#### F2.4 — Footnotes & Endnotes

- [ ] Add `NodeType::FootnoteRef`, `NodeType::FootnoteBody`, `NodeType::EndnoteRef`, `NodeType::EndnoteBody`
- [ ] DOCX: parse `w:footnoteReference` / `w:footnotes.xml` / `w:endnotes.xml`
- [ ] Layout engine: reserve space at page bottom for footnotes, flow footnote content
- [ ] Footnote separator line
- [ ] Auto-numbering (1, 2, 3... or i, ii, iii... configurable)
- [ ] Click footnote marker → scroll to footnote, click back → return to marker
- [ ] Tests: document with footnotes, footnotes spanning pages

#### F2.5 — Margin Collapsing

- [ ] Implement CSS-style margin collapsing between adjacent paragraphs
- [ ] `space_after` of paragraph N collapses with `space_before` of paragraph N+1 (larger value wins)
- [ ] Suppress collapsing across page breaks and section breaks
- [ ] Tests: paragraphs with different spacing values

---

### Phase F3: DOCX Fidelity (70% → 90%)

> **Goal**: Handle the remaining 20% of DOCX features that real-world documents use.
> **Estimated effort**: 4-6 weeks

#### F3.1 — Equations (OMML)

- [ ] Parse `<m:oMath>` elements from DOCX
- [ ] Convert OMML to LaTeX (or MathML) for rendering
- [ ] Render equations via KaTeX in browser (lightweight, fast)
- [ ] Round-trip: preserve `<m:oMath>` XML through model (store as raw XML attribute)
- [ ] Tests: inline equation, display equation, matrix, fraction, integral

#### F3.2 — Charts

- [ ] Parse `<c:chart>` references from DOCX
- [ ] Extract chart data (categories, series, values)
- [ ] Render charts via Chart.js or D3.js in browser
- [ ] Round-trip: preserve chart XML through model
- [ ] Supported chart types: bar, line, pie, scatter (cover 80% of use cases)

#### F3.3 — Advanced Track Changes

- [ ] Parse/write `w:moveTo` / `w:moveFrom` (text moves, not just ins/del)
- [ ] Table cell-level track changes
- [ ] Property change tracking (formatting changes)
- [ ] Revision marks with dates and authors
- [ ] "Accept/Reject" with review pane UI showing all changes

#### F3.4 — Drawing Canvas (DrawingML)

- [ ] Parse `<wps:wsp>` (WordprocessingShape) elements
- [ ] Basic shapes: rectangle, oval, line, arrow, text box, callout
- [ ] Shape properties: fill (solid, gradient), stroke, text content, effects (shadow)
- [ ] Render shapes as SVG in browser
- [ ] Group shapes: `<wpg:wgp>` container
- [ ] Round-trip: preserve DrawingML XML

---

### Phase F4: Font & Rendering Accuracy

> **Goal**: Pixel-accurate text rendering matching Word/Docs output.
> **Estimated effort**: 3-4 weeks

#### F4.1 — Font Metrics Accuracy

- [ ] Replace synthesized glyphs (font_size * 0.6) with actual glyph metrics from fallback fonts
- [ ] Embed a minimal fallback font (e.g., subset of Noto Sans) in WASM binary for guaranteed availability
- [ ] Per-character width lookup from font tables (not monospaced average)
- [ ] Kerning pair support (from `kern` / `GPOS` font tables)

#### F4.2 — Font Fallback Chain

- [ ] Extend fallback chain: try all document-referenced fonts → system fonts → script-specific fallbacks → embedded Noto
- [ ] Script detection: identify Unicode blocks, select appropriate fallback per script
- [ ] CSS `@font-face` generation: load document-embedded fonts in browser
- [ ] Font substitution table (map common fonts to available alternatives)

#### F4.3 — Canvas-Based Rendering (Optional)

- [ ] Alternative rendering mode: use HTML5 Canvas instead of DOM spans
- [ ] Pixel-accurate glyph placement (no browser text layout interference)
- [ ] Faster rendering for large documents (Canvas is faster than thousands of DOM elements)
- [ ] Selection overlay: draw selection highlight on Canvas layer
- [ ] Trade-off: lose browser text selection, need custom selection implementation

---

### Phase F5: Collaboration Hardening

> **Goal**: Production-ready collaboration without data loss.
> **Estimated effort**: 2-3 weeks

#### F5.1 — CRDT Edge Cases

- [ ] Audit for potential deadlock in concurrent tree operations (Kleppmann move algorithm)
- [ ] Stress test: 10 replicas, 1000 concurrent ops, verify convergence
- [ ] Handle tombstone accumulation (GC tombstones older than 30 days with all replicas synchronized)
- [ ] Operation deduplication across reconnections
- [ ] Snapshot size limits (compact when > 10MB)

#### F5.2 — Autosave Safety

- [ ] Multi-tab detection: use BroadcastChannel to coordinate saves
- [ ] Atomic write: write to temp key, then swap (prevent partial writes)
- [ ] Checksum verification on load (detect corrupted saves)
- [ ] Server-side persistence: save to backend, not just IndexedDB (for collaboration)
- [ ] Recovery dialog: show when detecting data from crashed session

---

## Engine Gap Priority Matrix

| Phase | Impact | Effort | Priority |
|-------|--------|--------|----------|
| **F1** Text & Scripts | Opens non-Latin markets | 4-6 weeks | HIGH |
| **F2** Advanced Layout | Handles 90% of real DOCX | 6-8 weeks | HIGH |
| **F3** DOCX Fidelity | Professional document support | 4-6 weeks | MEDIUM |
| **F4** Font Accuracy | Visual quality | 3-4 weeks | MEDIUM |
| **F5** Collab Hardening | Data safety | 2-3 weeks | HIGH |

---

## Full Recommended Execution Order (UI + Engine)

```
Sprint 1-2:    E1.* (core editing bugs)
Sprint 3-4:    E2.* (clipboard) + F5.* (collab safety)
Sprint 5-6:    E3.* (undo UX) + F2.2 (nested tables) + F2.5 (margin collapsing)
Sprint 7-8:    E4.* (table/image UX) + F1.1 (BiDi)
Sprint 9-10:   E8.* (performance) + F2.1 (floating images)
Sprint 11-12:  E5.* (collab UI) + F1.2 (complex scripts)
Sprint 13-14:  E6.* (mobile) + F2.3 (multi-column)
Sprint 15-16:  E7.* (accessibility) + F2.4 (footnotes)
Sprint 17-18:  E9.* (advanced features) + F3.1 (equations) + F4.1 (font metrics)
Sprint 19-20:  E10.* (polish) + F3.* (remaining DOCX fidelity)
```

**Total estimated timeline**: ~20 sprints (~40 weeks / 10 months) for one developer.
**With 3-person team**: ~14 weeks (3.5 months) with parallelization.
**With 5-person team**: ~10 weeks (2.5 months).

---

## Success Metrics

| Metric | Current | After E1-E3 | After E1-E10 | After E+F (Full) |
|--------|---------|-------------|--------------|-------------------|
| Playwright tests | 47 | 70 | 150+ | 250+ |
| Rust tests | 1,189 | 1,200 | 1,250 | 1,400+ |
| DOCX fidelity | ~70% | ~72% | ~75% | ~90% |
| Lighthouse Performance | ~75 | 85 | 90+ | 90+ |
| Lighthouse Accessibility | ~65 | 80 | 95+ | 95+ |
| Max document size (smooth) | ~50 pages | ~100 pages | ~500 pages | ~1000 pages |
| BiDi / RTL support | No | No | No | Yes |
| Mobile usable | No | No | Yes | Yes |
| Time to interactive | ~2s | ~1.5s | < 1s | < 1s |
| Edit latency (p99) | ~50ms | ~30ms | < 16ms | < 16ms |
| Comparable to | Mammoth.js | Basic editor | Google Docs (80%) | Google Docs (90%) |

---

## What This Roadmap Will NOT Achieve

Being honest about scope:

1. **Spreadsheets & Presentations** — These are each as complex as the entire document engine. Building Excel/PowerPoint equivalents is a separate multi-year project. OnlyOffice and LibreOffice have dedicated teams for each.

2. **100% DOCX fidelity** — Microsoft Word has 30+ years of accumulated features. Even Google Docs doesn't achieve 100%. Target is 90% which covers virtually all business documents.

3. **Replacing Word/Google Docs** — The goal is to be a **viable self-hosted alternative** for organizations that need data sovereignty, open source, or custom integration — not to win market share from Google.

4. **Native desktop performance** — WASM is fast but will never match a native C++ app like LibreOffice. For 95% of documents, the difference is imperceptible.

---

## Unique Competitive Advantages

Despite the gaps, s1engine has genuine advantages that OnlyOffice/Collabora/Google Docs do not:

| Advantage | Details |
|-----------|---------|
| **MIT License** | OnlyOffice is AGPL (forces open source), Google Docs is proprietary. s1engine is MIT — embed anywhere. |
| **Pure Rust / No C deps** | Entire stack compiles to WASM. No native binary distribution needed. Deploy as static files on a CDN. |
| **Built-in CRDT** | Fugue algorithm with 182 tests. OnlyOffice uses OT (requires central server). Collabora uses lock-based (no true real-time). CRDTs enable peer-to-peer, offline-first collaboration. |
| **Embeddable SDK** | Not just an app — a library. Build custom editors, document processors, mail merge engines, report generators on top. |
| **Self-hostable** | No vendor lock-in. No data leaves your infrastructure. |
| **Single binary** | WASM binary + JS + CSS. No Java runtime (POI), no Docker containers (Collabora), no complex deployment. |
