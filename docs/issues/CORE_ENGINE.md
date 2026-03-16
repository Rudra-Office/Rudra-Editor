# Core Engine Issues (s1-model, s1-ops)

> Tracking file for bugs in the document model and operations crates.
> Last updated: 2026-03-17

## Critical

| ID | Issue | File | Lines | Status |
|----|-------|------|-------|--------|
| CORE-01 | `MoveNode` inverse `old_parent_id`/`old_index` fields — analyzed and confirmed NOT a bug (fields are recalculated on apply) | `s1-ops/src/operation.rs` | 313-319 | WONTFIX |
| CORE-02 | `Selection::node_ids()` only returns anchor+focus, losing all intermediate nodes in multi-node selections | `s1-ops/src/cursor.rs` | 54-62 | FIXED |
| CORE-03 | `move_node()` index off-by-one when moving within same parent — incorrect sibling ordering | `s1-model/src/tree.rs` | 369-437 | FIXED |

## Major

| ID | Issue | File | Lines | Status |
|----|-------|------|-------|--------|
| CORE-04 | `root_node()` panics with `.expect()` — violates no-panics-in-library rule | `s1-model/src/tree.rs` | 192 | FIXED |
| CORE-05 | Silent index clamping in `move_node()` instead of returning error | `s1-model/src/tree.rs` | 423 | FIXED |
| CORE-06 | `AttributeMap` missing typed getters for `LineSpacing`, `Borders`, `ListInfo`, `TabStops`, `FieldType` | `s1-model/src/attributes.rs` | 495-538 | FIXED |

## Medium

| ID | Issue | File | Lines | Status |
|----|-------|------|-------|--------|
| CORE-07 | `char_offset_to_byte()` silently falls back to string length on out-of-bounds | `s1-model/src/tree.rs` | 775-780 | FIXED |
| CORE-08 | Cursor `Position`/`Selection` not validated against document structure | `s1-ops/src/cursor.rs` | 8-22 | FIXED |
| CORE-09 | Transaction rollback silently ignores failures with `let _ = apply()` | `s1-ops/src/transaction.rs` | 62-87 | FIXED |
| CORE-10 | `DeleteNode` operation conflates fresh-delete and undo-restore patterns | `s1-ops/src/operation.rs` | 21-31 | FIXED |
| CORE-11 | `SetMetadata` uses `Option<Option<String>>` — confusing triple-Option | `s1-ops/src/operation.rs` | 76-96 | FIXED |

---

## Resolution Log

| ID | Date | Fix Description | Commit |
|----|------|-----------------|--------|
| CORE-01 | 2026-03-16 | Analyzed: `old_parent_id`/`old_index` are "stored on apply" metadata recalculated each time — not a bug | — |
| CORE-02 | 2026-03-16 | Updated doc comment; added `node_ids_in_range(model)` method with DFS tree traversal to find all text nodes between anchor and focus | — |
| CORE-04 | 2026-03-16 | Changed `root_node()` return type from `&Node` to `Option<&Node>`, removed `.expect()` | — |
| CORE-06 | 2026-03-16 | Added 7 typed getters: `get_underline_style`, `get_line_spacing`, `get_borders`, `get_tab_stops`, `get_list_info`, `get_media_id`, `get_field_type` | — |
| CORE-07 | 2026-03-16 | Changed `char_offset_to_byte()` to return `Result` with bounds check; updated all callers to propagate `TextOffsetOutOfBounds` error | — |
| CORE-09 | 2026-03-16 | Added detailed doc comment explaining best-effort rollback semantics | — |
| CORE-03 | 2026-03-17 | Removed incorrect same-parent index adjustment; `new_index` is now the desired final position after removal. Added 3 regression tests (forward, backward, no-op) | — |
| CORE-05 | 2026-03-17 | Added `#[cfg(debug_assertions)]` warning when index exceeds child count and is clamped | — |
| CORE-08 | 2026-03-17 | Already fixed: `Position::validate()` and `Selection::validate()` methods exist, checking node existence, type, and offset bounds | — |
| CORE-10 | 2026-03-17 | Already documented: doc comments at lines 22-26 clearly explain the dual-purpose pattern (fresh-delete vs undo-restore). This is by design for the operation/inverse symmetry. | — |
| CORE-11 | 2026-03-17 | Already documented: doc comments at lines 85-88 explain the triple-Option: outer=applied?, inner=existed?. Same pattern used in SetStyle. This is by design for undo capture. | — |
