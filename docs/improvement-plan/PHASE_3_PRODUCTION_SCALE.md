# Phase 3: Production Scale (Performance & Features)

## Status: Not Started

## Goal
Optimize for large documents (100+ pages) and teams (20+ concurrent editors). Make collaboration trustworthy enough that users don't second-guess what they see.

## Current Problems

| Symptom | Root Cause | Where |
|---------|-----------|-------|
| Pressing Enter in collab triggers full document re-upload | Structural ops (split/merge) have no CRDT representation, fall back to fullSync | collab.js broadcastOp → scheduleDebouncedFullSync |
| Large doc collab is slow | fullSync uses Base64 JSON (~33% overhead) | collab.js btoa(), collab.rs Message::Text |
| New joiner sees stale content | Server stores snapshot only when peers send fullSync, no periodic refresh | file_sessions.rs update_snapshot |
| "Why is my screen different?" | No server-side document reconstruction, depends entirely on peer snapshots | No server-side engine |
| Checksum sent but corruption not blocked | Receive-side logs warning but applies sync anyway | collab.js:1753 |

## Key Objectives

### 1. Binary WebSocket Protocol (S-01)
**Current:** `btoa(String.fromCharCode(...bytes))` for every fullSync. Server only handles `Message::Text`.

**Required:**
- Send raw DOCX bytes as Binary WebSocket frames
- Header: `[4 bytes: payload length][JSON header: type/version/checksum][raw bytes]`
- Fallback to Base64 for environments without binary frame support
- Server handles both `Message::Text` and `Message::Binary`
- Expected improvement: ~33% reduction in sync bandwidth

### 2. Structural CRDTs (S-02)
**Current:** `SplitNode` and `MergeNode` are not CRDT operations. Pressing Enter broadcasts a raw `splitParagraph` op + schedules fullSync.

**Required:**
- Add `SplitNode(nodeId, offset)` and `MergeNode(nodeId1, nodeId2)` to `s1-crdt` crate
- These ops must be commutative and conflict-resolvable
- Remove fullSync trigger for Enter/Backspace in CRDT mode
- This is the hardest item in the entire roadmap — requires CRDT theory work

### 3. Server-Side Document State (S-03)
**Current:** Server stores bytes from peer fullSync. Cannot validate, reconstruct, or serve guaranteed-fresh state.

**Required:**
- Server runs a lightweight `s1engine::Engine` instance per active room
- Applies operations to maintain live document state
- New joiners get server-reconstructed snapshot (no peer dependency)
- Snapshot freshness is guaranteed, not dependent on peer timing
- This requires compiling s1engine for server-side use (already a dependency)

### 4. Checksum Enforcement (S7-07 completion)
**Current:** Checksum computed and sent (FNV-1a 32-bit) but receive-side only warns.

**Required:**
- Reject fullSync with mismatched checksum
- Request retransmission from sender
- Log corruption events for admin visibility

### 5. Snapshot Freshness (S-04)
**Current:** No `last_snapshot_at` tracking. No periodic refresh.

**Required:**
- Track when snapshot was last updated
- When a new peer joins and snapshot is older than 30s, request fresh fullSync from active peers
- Server-side document state (S-03) would eliminate this need entirely

## Dependencies
- S-01 (Binary WS) is independent — can start now
- S-02 (Structural CRDTs) is independent but high complexity
- S-03 (Server state) depends on having s1engine available in server binary (already is)
- S-04 (Snapshot freshness) is a stopgap until S-03 is done
