# Phase 1: Hardening (Stability & Fidelity)

## Status: Mostly Complete — remaining gaps: concurrent E2E tests, checksum enforcement

## Goal
Eliminate data loss, prevent formatting collapse during typing, and stabilize the collaboration handshake.

## Completed Items

| ID | Task | Evidence |
|---|---|---|
| H-01 | `replace_text` diff-based sync in `syncParagraphText` | render.js:867 — prefix/suffix diff → `replace_text()` |
| H-02 | `requestCatchup` on reconnect (not `sync-req`) | collab.js:698 |
| H-03 | Targeted unicast for catch-up ops (`_target` field) | collab.rs:621, filtering at line 369 |
| H-04 | Unified `access`/`mode` semantics | collab.js:371 sends both from single `accessLevel` |
| H-06 | Cursor position saved/restored across re-renders | render.js:131-175 |
| H-07 | `renderNodeById` as default for paragraph edits | input.js: 13 call sites |
| H-08 | `applySplitParagraphClipping` wired into repaginate | pagination.js:441 |
| H-09 | 15 missing co-editing op handlers added | collab.js:1240-1750 |
| H-10 | `normalizeRemoteOp` for field name mismatches | collab.js:1001 |
| S6-05 | AI/Autocorrect use `replace_text` | ai-inline.js:453, ai-panel.js:640, input.js:2342 |

## Remaining Gaps

### 1. First-Edit Cache Miss (H-01) — FIXED
`syncedTextCache` is now pre-populated during `renderPageFromWasm()` using `getEditableText()`. Every paragraph has a cache entry before the user types.

### 2. ~109 `renderDocument()` Calls (S5-03) — MITIGATED
`renderDocument()` now delegates to `renderDocumentFromWasm()` which uses per-page WASM HTML — no full DOM rebuild. 6 collab op handlers and paste/split/merge in input.js use `renderAffectedPages()` for incremental updates. The remaining callers still invoke `renderDocument()` but it's now a fast WASM-authority path.

### 3. No Concurrent Editing E2E Tests (H-05 incomplete)
90 Playwright tests exist but all are single-user. No tests for:
- Two users typing in the same paragraph
- Reconnect during active editing
- Offline buffer replay ordering
- Page-boundary typing with remote edits

**Impact:** Collab regressions go undetected.
**Fix needed:** Add multi-tab or multi-browser Playwright tests using the relay server.

### 4. Checksum Validation Warns But Doesn't Reject (S7-07 incomplete)
Checksum is computed and sent during fullSync (collab.js:964) but the receive side only logs a warning at line 1753 and still applies the sync.

**Impact:** Corrupt snapshots can still propagate.
**Fix needed:** Change warning to rejection + request retransmission.
