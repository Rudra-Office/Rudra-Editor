# Unified Server Architecture — Implementation Plan

> Transform s1-server into a single-container document editing platform.
> Single binary serves: static editor + REST API + WebSocket collab + file sessions.
> Created: 2026-03-19

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    s1-server (single binary)              │
│                                                          │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ Static Files │  │   REST API    │  │   WebSocket    │  │
│  │ (editor UI)  │  │  /api/v1/*    │  │  /ws/edit/{id} │  │
│  │  HTML/JS/CSS │  │  Files, Docs  │  │  Collab rooms  │  │
│  │  WASM bundle │  │  Convert      │  │  per fileId    │  │
│  └─────────────┘  └──────────────┘  └────────────────┘  │
│                           │                    │          │
│                    ┌──────▼────────────────────▼───────┐  │
│                    │         File Session Manager       │  │
│                    │  - Temp files with TTL             │  │
│                    │  - Authoritative doc state          │  │
│                    │  - Editor tracking (who, mode)      │  │
│                    │  - Auto-snapshot every 30s          │  │
│                    │  - Cleanup expired sessions         │  │
│                    └──────────────┬────────────────────┘  │
│                                  │                        │
│                    ┌─────────────▼─────────────────────┐  │
│                    │       Storage Backend              │  │
│                    │  LocalFS / S3 / Memory             │  │
│                    └───────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Modes

### Mode 1: Standalone
- User opens `http://server:8080/`
- Uploads a document → gets `fileId`
- Shares link: `http://server:8080/?file={fileId}`
- Co-editors open the link → join the same editing session
- After all editors leave → file kept for 5 min (configurable) then deleted

### Mode 2: Integrated (like OnlyOffice Document Server)
- Host product generates a JWT:
  ```json
  {
    "fileId": "doc-123",
    "userId": "user-456",
    "userName": "Alice",
    "permissions": "edit",
    "downloadUrl": "https://myapp.com/files/123/download",
    "callbackUrl": "https://myapp.com/s1engine/callback",
    "exp": 1700000000
  }
  ```
- Editor opens with `?token=<jwt>`
- Server validates JWT, fetches file from `downloadUrl`
- On save/close → server POSTs modified file to `callbackUrl`

---

## Implementation Phases

### Phase A: Unified Server (Static Files + File Sessions)

| ID | Task | Test Cases |
|----|------|-----------|
| A-01 | Serve static editor files via `tower_http::ServeDir` | TC: `GET /` returns index.html; `GET /src/styles.css` returns CSS; `GET /nonexistent` returns 404 |
| A-02 | File session manager: create, get, update, join, leave, cleanup | TC: Create session → exists; Join → editor_count=1; Leave → count=0, status=idle; TTL → expired and cleaned |
| A-03 | `POST /api/v1/files` — upload file, create session | TC: Upload TXT → 201 + fileId; Upload DOCX → 201 + word count; No file → 400; Too large → 413 |
| A-04 | `GET /api/v1/files/{id}` — get session info | TC: Valid ID → editors, mode, status; Invalid ID → 404 |
| A-05 | `GET /api/v1/files/{id}/download` — get latest snapshot | TC: Returns current bytes; After update → returns updated bytes; Invalid ID → 404 |
| A-06 | `DELETE /api/v1/files/{id}` — force close session | TC: Removes session; Returns last bytes; Editors disconnected |
| A-07 | `GET /api/v1/files` — list active sessions | TC: Empty list; After upload → 1 item; After close → 0 items |
| A-08 | Background cleanup task (every 60s) | TC: Session with TTL=1s → cleaned after 2s; Active session not cleaned |
| A-09 | Single Dockerfile (Rust build + editor static files) | TC: `docker build` succeeds; `docker run` serves editor + API |
| A-10 | Updated .env.example with unified config | TC: All env vars documented |

### Phase B: WebSocket Editing (Server-Authoritative)

| ID | Task | Test Cases |
|----|------|-----------|
| B-01 | `WS /ws/edit/{fileId}` — WebSocket endpoint with file session | TC: Connect to valid fileId → welcome; Invalid fileId → error + close; editor_count incremented |
| B-02 | On connect: send latest document snapshot to new editor | TC: New editor receives full document bytes; Existing editor's ops don't affect snapshot delivery |
| B-03 | Ops broadcast to all editors in same fileId room | TC: Editor A sends op → Editor B receives it; Editor C in different room doesn't receive it |
| B-04 | Server records ops and updates snapshot every 30s | TC: After ops + 30s → snapshot updated; Download returns updated document |
| B-05 | On disconnect: decrement editor count, start TTL if last | TC: Last editor leaves → status=idle; TTL starts; New editor joins before TTL → session restored |
| B-06 | Editor collab.js connects to `/ws/edit/{fileId}` | TC: Editor opens with ?file=abc → WS connects to /ws/edit/abc |
| B-07 | Remove relay.js dependency from editor | TC: Editor works without relay.js running |

### Phase C: Integration Mode (JWT + Callbacks)

| ID | Task | Test Cases |
|----|------|-----------|
| C-01 | JWT validation in file session creation | TC: Valid JWT → session created with downloadUrl; Invalid JWT → 401; Expired JWT → 401 |
| C-02 | Server fetches file from `downloadUrl` in JWT | TC: Valid URL → file downloaded and session created; Invalid URL → error; Timeout → error |
| C-03 | Permission enforcement from JWT | TC: edit → can modify; view → read-only WS; comment → can add comments only |
| C-04 | Callback on save: POST modified file to `callbackUrl` | TC: On explicit save → POST with file bytes; On force close → POST; Callback failure → retry 3x |
| C-05 | Callback on session end (all editors left + TTL) | TC: After TTL → POST final state to callbackUrl |
| C-06 | `GET /api/v1/files/{id}/info` — editing metadata | TC: Returns: editors list, duration, mode, last modified |
| C-07 | Editor opens with `?token=<jwt>` | TC: Editor extracts fileId from JWT, loads file from server |

### Phase D: Single Docker Image

| ID | Task | Test Cases |
|----|------|-----------|
| D-01 | Multi-stage Dockerfile: Rust build + WASM build + Vite build → slim runtime | TC: Build succeeds; Image < 200MB; Health check passes |
| D-02 | Single docker-compose.yml (one service) | TC: `docker compose up` → editor + API + WS on single port |
| D-03 | TLS support via env vars (cert path + key path) | TC: With certs → HTTPS; Without → HTTP |
| D-04 | .env.example covers all config | TC: All env vars have defaults; Server starts with just .env |

---

## Test Matrix

### Unit Tests (Rust)

```
file_sessions::tests::create_and_get
file_sessions::tests::editor_join_leave
file_sessions::tests::snapshot_update
file_sessions::tests::cleanup_expired
file_sessions::tests::force_close
file_sessions::tests::list_sessions
```

### Integration Tests (HTTP)

```
POST /api/v1/files (upload TXT)           → 201 + fileId
POST /api/v1/files (upload DOCX)          → 201 + wordCount
GET  /api/v1/files                        → list of active sessions
GET  /api/v1/files/{id}                   → session info (editors, mode, status)
GET  /api/v1/files/{id}/download          → document bytes
DELETE /api/v1/files/{id}                 → 204 + session closed
POST /api/v1/convert (TXT→PDF)           → PDF bytes
GET  /health                              → {"status":"ok"}
GET  /                                    → editor HTML
```

### WebSocket Tests

```
WS /ws/edit/{fileId} connect              → welcome message + snapshot
WS /ws/edit/{fileId} send op              → broadcast to other editors
WS /ws/edit/invalid disconnect            → error message
WS /ws/edit/{fileId} 2 editors join       → both get snapshot
WS /ws/edit/{fileId} last editor leaves   → TTL starts
```

### Docker Tests

```
docker build                              → succeeds
docker run + curl /health                 → 200
docker run + curl /                       → editor HTML
docker run + curl /api/v1/info            → server info
docker run + upload + download            → round-trip
```

---

## File Changes

| File | Action |
|------|--------|
| `server/src/main.rs` | Add static file serving, file session routes, cleanup task |
| `server/src/file_sessions.rs` | NEW — session manager with TTL |
| `server/src/routes.rs` | Add file session endpoints (upload, info, download, close, list) |
| `server/src/collab.rs` | Update WS handler to use file sessions for state |
| `server/Cargo.toml` | Add sessions field to AppState |
| `editor/src/collab.js` | Connect to `/ws/edit/{fileId}` instead of relay |
| `editor/src/main.js` | Load file from server API if `?file=` in URL |
| `Dockerfile` | NEW unified single-image build |
| `docker-compose.yml` | Single service, single port |
| `.env.example` | Unified configuration |

---

### Phase E: Admin Panel

| ID | Task | Test Cases |
|----|------|-----------|
| E-01 | Admin auth (username/password from config) | TC: Correct creds → 200; Wrong creds → 401; No creds → 401 |
| E-02 | `GET /admin/` — dashboard HTML (served by Rust) | TC: Shows active sessions, memory, uptime |
| E-03 | `GET /admin/api/sessions` — list all file sessions | TC: Returns JSON with all sessions + editor counts |
| E-04 | `GET /admin/api/rooms` — list all collab rooms | TC: Returns active rooms, peer counts, op counts |
| E-05 | `DELETE /admin/api/sessions/{id}` — force close session | TC: Session removed, editors disconnected |
| E-06 | `GET /admin/api/logs` — recent server logs | TC: Returns last 100 log entries |
| E-07 | `GET /admin/api/stats` — server statistics | TC: Returns uptime, memory, requests/sec, active editors |
| E-08 | `GET /admin/api/config` — current configuration | TC: Returns sanitized config (no secrets) |
| E-09 | Admin credentials in config (TOML + env vars) | TC: S1_ADMIN_USER + S1_ADMIN_PASS env vars work |

## Priority Order

1. **Phase A** (file sessions + static serving) — ✅ DONE
2. **Phase B** (WebSocket editing) — core co-editing
3. **Phase D** (Docker) — deployment
4. **Phase C** (JWT integration) — enterprise features
5. **Phase E** (Admin panel) — ops and monitoring

## Estimated Effort

| Phase | Effort | Status |
|-------|--------|--------|
| Phase A | 2-3 hours | ✅ DONE |
| Phase B | 2-3 hours | TODO |
| Phase D | 1 hour | ✅ DONE (Dockerfile.unified) |
| Phase C | 3-4 hours | TODO |
| Phase E | 2-3 hours | TODO |
| **Total** | **10-14 hours** |
