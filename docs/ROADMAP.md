# Development Roadmap

## Engine (COMPLETE)

```
Phase 0: Planning           ████████████████████  COMPLETE
Phase 1: Foundation         ████████████████████  COMPLETE
Phase 2: Rich Documents     ████████████████████  COMPLETE
Phase 3: Layout & Export    ████████████████████  COMPLETE
Phase 4: Collaboration      ████████████████████  COMPLETE
Phase 5: Production Ready   ████████████████████  COMPLETE (WASM + C FFI)
Phase 6-12: Formats/Layout  ████████████████████  COMPLETE (1,589 tests passing)
```

The s1engine document engine is complete and production-ready.

## Web Editor (IN PROGRESS)

**Approach**: Integrate OnlyOffice sdkjs as the editor frontend, with s1engine WASM as the document backend.

```
Phase 16: OnlyOffice Integration  ░░░░░░░░░░░░░░░░░░░░  IN PROGRESS
```

### Phase 16: OnlyOffice sdkjs + s1engine WASM

**Goal**: Production-quality web document editor using OnlyOffice's battle-tested rendering/input system backed by our Rust document engine.

1. Set up OnlyOffice sdkjs in `web/` directory
2. Build adapter layer between sdkjs API and s1engine WASM
3. Document open: DOCX bytes → s1engine WASM parse → sdkjs document model
4. Document save: sdkjs model → s1engine WASM export → DOCX bytes
5. Collaboration: sdkjs ↔ s1engine CRDT ↔ WebSocket relay
6. Server integration: Axum server + relay.js
