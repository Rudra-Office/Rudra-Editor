# GEMINI.md - Project Mandates & Core Instructions

This file contains foundational mandates for Gemini CLI and other AI agents working on the Rudra Office project. These instructions take absolute precedence over general defaults.

## 1. The Architectural Constitution (Non-Negotiables)

To scale Rudra Office to production-grade teams, all development must adhere to these eight principles:

1.  **Single Source of Truth:** WASM/Model owns content and layout; CRDT/Server owns collaboration; DOM is a **projection only**. Remove all hybrid ownership.
2.  **No Routine `fullSync`:** Convergence must happen via first-class CRDT operations. `fullSync` is a disaster recovery fallback, not a standard sync mechanism.
3.  **Authoritative Server:** The server must maintain a reliable current snapshot and provide deterministic catch-up for reconnecting peers.
4.  **Range-Aware Mutations:** All editing paths (typing, paste, autocorrect, AI) must use atomic, range-aware edits. Never force-set full paragraph text.
5.  **Engine-Owned Pagination:** Page boundaries and paragraph fragments must be dictated by the WASM engine. No DOM/CSS-based layout simulation.
6.  **Incremental Rendering:** Default to single-node or scoped re-renders. Avoid full document re-renders for localized changes.
7.  **Brutal Correctness Testing:** Every core change must pass the "Trust-focused Regression Suite" (edge cases like concurrent edits at page boundaries and high-latency reconnects).
8.  **Narrow Product Promise:** Prioritize being **DOCX-first**, supporting small teams and moderate document complexity with 100% trust.

## 2. Core Engineering Mandates

- **WASM Ownership:** The Document Engine (Rust/WASM) is the single source of truth. All visual fragments and layout are dictated by the engine.
- **CRDT-Native Operations:** ALL document mutations (text and structural) MUST use the range-aware WASM CRDT API. Never rebuild the collab state from a static model.
- **Atomic Text Sync:** Use the diff-from-truth strategy in `render.js` for all text synchronization.
- **Zero CDN Dependency:** All assets (fonts, icons, scripts) MUST be self-hosted.

## 2. Tracking & Process Mandates

- **Improvement Pipeline:** The `docs/improvement-plan/IMPROVEMENT_TRACKER.md` is the authoritative guide for the project's evolution. Do NOT deviate from this plan without explicit user confirmation.
- **Progress Verification:** After completing a task, update the tracker to 🟢 (Completed) and, after verification, to 🔵 (Verified). Provide the test output as proof of completion.
- **Zero Regression:** Always run existing tests before and after a change to ensure no regression in core editor functionality.

## 2. Collaboration Protocol

- **Monotonic Versioning:** Always respect the server-assigned `roomVersion`.
- **Binary Sync:** Favor binary WebSocket frames for document snapshots (`fullSync`) over Base64 strings to minimize bandwidth.
- **No Data Loss:** Ensure that structural edits (splits/merges) are coordinated with the CRDT layer to prevent character loss during concurrent edits.

## 3. Technology Stack

- **Core:** Rust (Workspace)
- **Editor:** Vanilla JS / Vite (minimize framework overhead)
- **Layout:** Pure Rust (`rustybuzz`, `ttf-parser`, `fontdb`)
- **WASM:** `wasm-bindgen`
- **Server:** Rust (Axio/Tokio)

## 4. Documentation

- Maintain the `docs/improvement-plan/` trackers as the source of truth for project maturity.
- Update `CHANGELOG.md` for every significant architectural shift.
