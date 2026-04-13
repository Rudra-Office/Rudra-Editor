# ADR-001: Editing Authority — s1engine-first

**Status**: Accepted
**Date**: 2026-04-14
**Decision**: s1engine is the source of truth for the document model.

## Context

The web editor uses OnlyOffice sdkjs for canvas rendering, input capture, cursor management, and formatting UI. The document backend is s1engine (Rust/WASM) which handles format conversion, CRDT collaboration, layout, and text shaping.

The question: which runtime owns the document model?

## Decision

**s1engine-first.**

- s1engine owns the document model, undo/redo history, collaboration state, and format conversion.
- OnlyOffice is a rendering and editing facade. It displays the document and captures user input.
- The adapter layer translates user edits from OnlyOffice into s1engine operations, and s1engine model state into OnlyOffice rendering commands.

## Consequences

### Open path
DOCX bytes → s1engine WASM parse → s1engine model → adapter maps to OnlyOffice runtime objects → OnlyOffice renders

### Edit path
User types/formats in OnlyOffice → adapter captures edit → s1engine operation applied → s1engine model updated → adapter pushes changes back to OnlyOffice for re-render (if needed)

### Save path
s1engine model → s1engine export → DOCX bytes

### Undo/redo
s1engine owns undo/redo history via its operation/transaction system. OnlyOffice undo/redo UI triggers s1engine undo/redo, then the adapter updates OnlyOffice display state.

### Collaboration
s1engine CRDT (Fugue algorithm) is the collaboration backend. Remote operations are applied to s1engine model, then the adapter pushes updates to OnlyOffice for rendering.

## What this means for implementation

1. The adapter must intercept OnlyOffice edit events and translate them to s1engine operations.
2. The adapter must be able to rebuild OnlyOffice display state from s1engine model state.
3. OnlyOffice's internal document model (CDocument, CParagraph, ParaRun) is a rendering cache, not the source of truth.
4. Save always reads from s1engine, never from OnlyOffice's internal model.
5. Undo/redo always goes through s1engine's transaction system.

## Alternatives considered

- **OnlyOffice-first**: s1engine is only import/export. Rejected because it makes s1engine's CRDT, layout engine, and operation system redundant.
- **Hybrid**: Phased ownership. Rejected as unnecessarily complex — s1engine already has the full document model.
