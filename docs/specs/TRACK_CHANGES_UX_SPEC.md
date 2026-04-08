# Track Changes UX Specification — M13.3

**Reference**: OnlyOffice `track-revisions-manager.js`, `review-info.js`, `RevisionsChange.js`
**Status**: Specification
**Current State**: Engine complete (parse/write/accept/reject via WASM). Editor has basic panel. Needs display modes + visual marks.

---

## Current Implementation (What Exists)

### Engine (s1-format-docx + ffi/wasm)
- Parse `<w:ins>`, `<w:del>`, `<w:moveTo>`, `<w:moveFrom>` from DOCX
- Store RevisionType, RevisionAuthor, RevisionDate, RevisionId as node attributes
- Write revisions back to DOCX on export
- WASM API: `tracked_changes_count()`, `tracked_changes_json()`, `accept_change(nodeId)`, `reject_change(nodeId)`, `accept_all_changes()`, `reject_all_changes()`

### Editor (toolbar-handlers.js)
- Track changes panel (#tcPanel) with change cards
- Accept/Reject buttons per change card
- Accept All / Reject All buttons
- Change navigation (previous/next)
- Edit mode toggle (editing/suggesting/viewing)

---

## What Needs Adding

### 1. Display Modes

Three modes for how tracked changes appear:

**Markup (default)**
- Insertions: underlined in reviewer color
- Deletions: strikethrough in reviewer color
- Formatting changes: indicated by sidebar balloon
- All content visible (both inserted and deleted text)

**Final**
- Insertions shown as normal text (no marks)
- Deletions hidden entirely
- Document appears as if all changes were accepted
- Read-only rendering mode (no edits while in Final view)

**Original**
- Deletions shown as normal text (no marks)
- Insertions hidden entirely
- Document appears as if all changes were rejected
- Read-only rendering mode

### 2. Visual Marks (Markup Mode)

#### Canvas Rendering
During text rendering phase:
1. Check each run's `data-tc-type` attribute (or revision attributes)
2. If `Insert`: draw colored underline below baseline
3. If `Delete`: draw colored strikethrough at mid-height
4. If `MoveFrom`: draw double strikethrough
5. If `MoveTo`: draw double underline
6. Color: deterministic from author hash (see Reviewer Colors below)

#### DOM Rendering (fallback)
- Insert: `<span class="tc-insert" style="text-decoration: underline; text-decoration-color: {color}">`
- Delete: `<span class="tc-delete" style="text-decoration: line-through; text-decoration-color: {color}">`
- In Final mode: Delete spans get `display: none`
- In Original mode: Insert spans get `display: none`

### 3. Reviewer Colors

Deterministic color assignment from author name/ID:

```javascript
const REVIEWER_COLORS = [
  '#4472C4', '#ED7D31', '#A5A5A5', '#FFC000', '#5B9BD5',
  '#70AD47', '#264478', '#9B57A0', '#636363', '#EB7E33',
  '#2F5597', '#BF9000', '#44546A', '#C00000', '#00B0F0',
];

function getReviewerColor(authorName) {
  let hash = 0;
  for (let i = 0; i < authorName.length; i++) {
    hash = ((hash << 5) - hash + authorName.charCodeAt(i)) | 0;
  }
  return REVIEWER_COLORS[Math.abs(hash) % REVIEWER_COLORS.length];
}
```

### 4. Sidebar Balloons

Each tracked change in the sidebar panel shows:
- Reviewer name (colored circle + name)
- Timestamp (relative: "2 hours ago" or absolute: "Apr 8, 2026 3:45 PM")
- Change type label ("Inserted", "Deleted", "Formatting changed", "Moved")
- Content preview (first 80 chars)
- Accept / Reject buttons
- Highlight in document when hovering over balloon

### 5. Mode Selector UI

Add dropdown to Review toolbar section:
```
[Markup ▼] [← Prev] [Next →] [Accept ▼] [Reject ▼]
                                ├─ Accept Change
                                ├─ Accept All Changes
                                └─ Accept All & Stop Tracking
```

### 6. Status Bar Integration

Add track changes indicator to status bar:
- "Track Changes: ON" / "Track Changes: OFF"
- Click to toggle
- Show count: "5 changes"
- Color indicator when tracking is on

### 7. Collaboration Integration

When track changes is enabled during collaboration:
- Remote changes tagged with remote user's info
- Reviewer colors consistent across all peers
- Accept/reject operations broadcast via CRDT
- Display mode is per-user (each user chooses their view)

---

## Implementation Order

1. Visual marks in DOM rendering (immediate visual impact)
2. Display mode selector (Markup/Final/Original)
3. Reviewer color assignment
4. Enhanced sidebar balloons
5. Status bar integration
6. Canvas rendering of marks (when canvas becomes default)
7. Collaboration integration
