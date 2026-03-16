# Editor UI/UX Issues

> Tracking file for bugs in the Folio editor frontend.
> Last updated: 2026-03-17

## High

| ID | Issue | File | Lines | Status |
|----|-------|------|-------|--------|
| UI-01 | Image context menu ignores scroll offset — uses clientX/Y, menu appears at wrong position | `images.js` | 318-324 | FIXED |

## Medium

| ID | Issue | File | Lines | Status |
|----|-------|------|-------|--------|
| UI-02 | Modal dialogs don't trap focus or restore selection on close | `toolbar-handlers.js` | 206-289 | FIXED |
| UI-03 | Z-index chaos — inconsistent hierarchy (100, 200, 10000) no strategy | `styles.css` | 69-1832 | FIXED |
| UI-04 | Ruler drag loses cursor position — selection not restored after drag | `ruler.js` | 194-200 | FIXED |
| UI-05 | Find bar Tab key escapes instead of cycling through controls | `find.js` | 47-53 | FIXED |
| UI-06 | Only first page gets `role="textbox"` — subsequent pages miss it | `pagination.js` | 426 | WONTFIX |
| UI-07 | Collab indicators missing `aria-label` — `title` not reliably announced | `collab.js` | 854-859 | FIXED |
| UI-08 | Mobile find bar positioning broken — left/right don't stretch correctly | `styles.css` | 2183-2188 | FIXED |
| UI-09 | Toolbar touch targets 28px — WCAG 2.5.5 requires 44px minimum | `styles.css` | 75 | FIXED |
| UI-10 | `pendingFormats` not cleared on blur — stale formats may apply later | `state.js` | 45 | FIXED |
| UI-11 | Inconsistent modal backdrop-click-to-close behavior | `toolbar-handlers.js` | 235-357 | FIXED |
| UI-12 | Image resize handles missing `aria-hidden` — screen readers announce them | `images.js` | 141-152 | FIXED |
| UI-13 | Comment resolved state not announced to screen readers | `toolbar-handlers.js` | 1954 | FIXED |
| UI-14 | Slash menu missing `role="listbox"` on parent container | `input.js` | 2582 | FIXED |
| UI-15 | Unused CSS variables — hardcoded shadows instead of `var(--shadow-*)` | `styles.css` | 36-337 | FIXED |

---

## Resolution Log

| ID | Date | Fix Description | Commit |
|----|------|-----------------|--------|
| UI-01 | 2026-03-16 | Replaced hardcoded menu size with `getBoundingClientRect()` measurement; added viewport bounds clamping | — |
| UI-03 | 2026-03-16 | Added 7 CSS custom properties (`--z-toolbar` through `--z-toast`); replaced 15 hardcoded z-index values | — |
| UI-05 | 2026-03-16 | Added Tab/Shift+Tab key handling to cycle focus through find bar inputs and buttons | — |
| UI-06 | 2026-03-16 | Verified: `createPageElement()` already sets `role="textbox"` and `aria-multiline="true"` on all pages | — |
| UI-07 | 2026-03-16 | Added `aria-label` alongside `title` on connection status indicator and peer dots | — |
| UI-10 | 2026-03-16 | Added capturing `blur` event listener on `pageContainer` to clear `state.pendingFormats` | — |
| UI-12 | 2026-03-16 | Added `aria-hidden="true"` and `role="presentation"` to all image resize handles | — |
| UI-14 | 2026-03-16 | Added programmatic `role="listbox"` and `aria-label` on slash menu container in `openSlashMenu()` | — |
| UI-02 | 2026-03-17 | Added `aria-modal="true"`, `role="dialog"` on open; filter disabled/hidden focusable elements; Tab/Shift+Tab cycling, Escape close, focus return already existed | — |
| UI-08 | 2026-03-17 | Changed mobile `.find-bar` from `position:absolute;left:8px;right:8px` to `position:fixed;left:0;right:0;width:100%` for full-width stretching | — |
| UI-09 | 2026-03-17 | Changed `.tb-btn` from 30px to 44px (min-width/min-height); also `.tb-select` and `.tb-input` to 44px — meets WCAG 2.5.5 minimum touch target | — |
| UI-11 | 2026-03-17 | Added centralized backdrop-click-to-close handler to all 16 modals via `initModalFocusTrap()` | — |
| UI-13 | 2026-03-17 | Added `announce()` calls for comment resolve/reopen; added `role="article"`, `aria-label`, `aria-pressed` on resolve button, and `<span class="sr-only" role="status">` | — |
| UI-15 | 2026-03-17 | Replaced 16 hardcoded `box-shadow` values with `var(--shadow-sm)`, `var(--shadow-md)`, or `var(--shadow-lg)` CSS variables | — |
| UI-04 | 2026-03-17 | Save selection range in `startDrag()`, restore in `endDrag()` after indent applied — cursor position preserved across ruler drag | — |
