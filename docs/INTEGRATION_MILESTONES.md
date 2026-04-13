# OnlyOffice sdkjs + s1engine WASM Integration

**Process for each milestone:**
1. **Analyse** — Study OnlyOffice source, understand the integration point
2. **Spec** — Write implementation doc with exact files, APIs, data flow
3. **Develop** — Code it
4. **Test** — Verify it works (automated + manual)
5. **Verify** — Re-analyse the complete state, check for regressions
6. **Mark Complete** — Only after all above pass

---

## Milestone 0: Setup & Checkout
**Status**: NOT STARTED

### Analyse
- Clone OnlyOffice sdkjs into `web/onlyoffice/`
- Clone OnlyOffice web-apps (the HTML shell that hosts sdkjs) into `web/shell/`
- Understand the build system (Makefile + Node.js)
- Identify the minimum files needed for a word processor (strip spreadsheet/presentation)
- Map the sdkjs boot sequence: HTML → JS → Canvas init → Document load

### Spec
- Document: which sdkjs files we keep, which we strip
- Document: the boot sequence with exact file:line references
- Document: where sdkjs calls out to the backend (every HTTP/API call)

### Develop
- Clone repos
- Set up build (npm install, make)
- Get the sdkjs word editor running standalone with a blank document
- Verify canvas renders, cursor blinks, typing works (using their built-in model)

### Test
- [ ] sdkjs builds without errors
- [ ] Editor loads in browser (http://localhost:port)
- [ ] Blank document shows
- [ ] Can type text
- [ ] Cursor blinks and follows typed text
- [ ] Can bold/italic/underline text

### Verify
- Re-check: no build errors, no console errors, editor fully functional with OnlyOffice's own model

---

## Milestone 1: WASM Bridge — Document Open
**Status**: NOT STARTED

### Analyse
- Study how sdkjs opens a document (the `openDocument` / `asc_nativeOpenFile` flow)
- Study the internal binary format sdkjs uses (DOCY / JSON serialization)
- Study how sdkjs populates its CDocument model from parsed data
- Map: DOCX bytes → sdkjs expectations (what format, what structure)

### Spec
- Document: exact sdkjs function that receives document data
- Document: data format sdkjs expects (binary DOCY? JSON? raw OOXML?)
- Document: adapter function signature: `openDocxWithWasm(bytes) → sdkjs model data`
- Decide: do we convert our s1engine model to sdkjs's model format, or do we intercept at the OOXML level?

### Develop
- Create `web/adapter/` directory
- Write `adapter.js` — the bridge between sdkjs and s1engine WASM
- Implement `openDocument(docxBytes)`:
  1. Pass DOCX bytes to s1engine WASM `open(bytes)`
  2. Extract document structure from s1engine model
  3. Feed structure into sdkjs's CDocument model
  - OR: pass raw OOXML XML to sdkjs's parser (simpler if sdkjs can parse OOXML directly)

### Test
- [ ] Open a real DOCX file via adapter
- [ ] Document content appears in sdkjs editor
- [ ] Text is correct (compare with Word/LibreOffice)
- [ ] Formatting preserved (bold, italic, headings)
- [ ] Tables render
- [ ] Images show (or placeholder)

### Verify
- Open 5 different DOCX files (simple, complex, tables, images, multilingual)
- Compare rendering with OnlyOffice desktop app
- Check browser console for errors

---

## Milestone 2: WASM Bridge — Document Save
**Status**: NOT STARTED

### Analyse
- Study how sdkjs saves a document (the `asc_DownloadAs` / save flow)
- Study how sdkjs serializes its CDocument model to binary/OOXML
- Map: sdkjs model → DOCX bytes

### Spec
- Document: exact sdkjs function that produces save data
- Document: adapter function: `saveDocxWithWasm() → Uint8Array`
- Decide: extract from sdkjs model → s1engine model → DOCX, or intercept sdkjs's own OOXML output

### Develop
- Implement `saveDocument()` in adapter:
  1. Extract current document state from sdkjs model
  2. Pass to s1engine WASM `export('docx')`
  3. Return DOCX bytes for download

### Test
- [ ] Type text in editor, save as DOCX
- [ ] Open saved DOCX in Word/LibreOffice — content matches
- [ ] Open saved DOCX in our editor — round-trip works
- [ ] Formatting survives save (bold, italic, headings, lists)

### Verify
- Round-trip test: open DOCX → edit → save → re-open → verify content

---

## Milestone 3: Editing Operations Bridge
**Status**: NOT STARTED

### Analyse
- Study how sdkjs handles text insertion (AddToParagraph → model mutation)
- Study how sdkjs handles deletion, formatting, undo/redo
- Map: which model mutations need to sync to s1engine WASM

### Spec
- Document: every sdkjs model mutation that must be mirrored in s1engine
- Document: the sync strategy (immediate sync vs batch sync)
- Document: undo/redo — does sdkjs handle it or does s1engine?

### Develop
- Hook sdkjs's model mutation events
- On each mutation: replay the operation in s1engine WASM
- Keep both models in sync (sdkjs for rendering, s1engine for persistence/export)

### Test
- [ ] Type text — both models have same content
- [ ] Delete text — both models updated
- [ ] Bold/italic — both models updated
- [ ] Undo/redo — both models stay in sync
- [ ] Save after edits — DOCX reflects all changes

### Verify
- Type a full paragraph, apply formatting, save, re-open, verify everything matches

---

## Milestone 4: Collaboration Bridge
**Status**: NOT STARTED

### Analyse
- Study how sdkjs handles collaborative editing (CoAuthoring module)
- Study our CRDT implementation (s1-crdt, Fugue algorithm)
- Map: sdkjs collaboration events ↔ s1engine CRDT operations

### Spec
- Document: collaboration message format (sdkjs vs our relay)
- Document: adapter functions for remote operation application
- Document: conflict resolution strategy

### Develop
- Connect sdkjs collaboration events to our CRDT system
- Connect our WebSocket relay to sdkjs's collaboration layer
- Implement remote op application: relay → s1engine CRDT → sdkjs model update

### Test
- [ ] Two browser tabs, same document
- [ ] Type in tab 1, text appears in tab 2
- [ ] Type in tab 2, text appears in tab 1
- [ ] Concurrent edits don't lose data
- [ ] Cursor indicators show peer positions

### Verify
- 3-way collaboration test: three tabs editing simultaneously

---

## Milestone 5: Server Integration
**Status**: NOT STARTED

### Analyse
- Study how our Axum server serves files and handles conversion
- Study how OnlyOffice's DocumentServer API works (callback URLs, etc.)
- Map: which server endpoints sdkjs needs

### Spec
- Document: server API endpoints for sdkjs
- Document: file storage and session management
- Document: Docker setup for production deployment

### Develop
- Add sdkjs-compatible API endpoints to our Axum server
- Serve the web editor from the server
- Implement file upload/download/conversion via our server
- Update Dockerfile

### Test
- [ ] Docker build succeeds
- [ ] Docker run → editor accessible at http://localhost:8787
- [ ] Upload DOCX → opens in editor
- [ ] Edit → save → download DOCX
- [ ] Collaboration works through Docker instance

### Verify
- Full end-to-end: Docker build → run → open DOCX → edit → save → verify

---

## Milestone 6: Production Polish
**Status**: NOT STARTED

### Analyse
- Performance profiling (document open time, typing latency)
- Missing features compared to standalone OnlyOffice
- Accessibility audit

### Develop
- Fix performance bottlenecks
- Add missing features
- Attribution and licensing compliance

### Test
- [ ] Open 100-page document in < 3 seconds
- [ ] Typing latency < 50ms
- [ ] All toolbar buttons functional
- [ ] Export to PDF, ODT, TXT works
- [ ] Mobile responsive

### Verify
- Test with 10 real-world DOCX files from different sources (Word, Google Docs, LibreOffice)

---

## Progress Tracker

| Milestone | Analyse | Spec | Develop | Test | Verify | Status |
|-----------|---------|------|---------|------|--------|--------|
| M0: Setup | | | | | | NOT STARTED |
| M1: Doc Open | | | | | | NOT STARTED |
| M2: Doc Save | | | | | | NOT STARTED |
| M3: Edit Bridge | | | | | | NOT STARTED |
| M4: Collab | | | | | | NOT STARTED |
| M5: Server | | | | | | NOT STARTED |
| M6: Polish | | | | | | NOT STARTED |
