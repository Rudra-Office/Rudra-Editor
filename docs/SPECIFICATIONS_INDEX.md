# Rudra Code Specification Index

> Each feature area has its own specification document.
> Process: Research → Spec → Break → Fix → Implement → Test → Verify
> Last updated: 2026-03-30

## Specifications

| # | Area | Document | Status |
|---|------|----------|--------|
| 1 | **Collaboration Protocol** | [COLLABORATION_PROTOCOL.md](COLLABORATION_PROTOCOL.md) | v3.1 — Complete (22 edge cases, full checklists) |
| 2 | **Editor UX** | [specs/EDITOR_UX_SPEC.md](specs/EDITOR_UX_SPEC.md) | v1.0 — Cursor, selection, clipboard, images, undo |
| 3 | **Rendering Engine** | [specs/RENDERING_SPEC.md](specs/RENDERING_SPEC.md) | v1.0 — Incremental render, pagination, performance |
| 4 | **DOCX Format Fidelity** | [specs/DOCX_FIDELITY_SPEC.md](specs/DOCX_FIDELITY_SPEC.md) | v1.0 — Feature matrix, round-trip, compatibility |
| 5 | **ODT Format Fidelity** | [specs/ODT_FIDELITY_SPEC.md](specs/ODT_FIDELITY_SPEC.md) | v1.0 — Feature matrix, limitations |
| 6 | **PDF Export** | [specs/PDF_EXPORT_SPEC.md](specs/PDF_EXPORT_SPEC.md) | v1.0 — Pipeline, targets, edge cases |
| 7 | **Server API** | [specs/SERVER_API_SPEC.md](specs/SERVER_API_SPEC.md) | v1.0 — REST + WebSocket + auth |
| 8 | **Security Model** | [specs/SECURITY_SPEC.md](specs/SECURITY_SPEC.md) | v1.0 — Auth, authz, input validation, DOCX security |
| 9 | **Performance** | [specs/PERFORMANCE_SPEC.md](specs/PERFORMANCE_SPEC.md) | v1.0 — Targets, benchmarks, profiling |
| 10 | **Admin Panel** | [specs/ADMIN_PANEL_SPEC.md](specs/ADMIN_PANEL_SPEC.md) | v1.0 — Dashboard, errors, health, enhancement plan |
| 11 | **E2E Test Plan** | [specs/E2E_TEST_PLAN.md](specs/E2E_TEST_PLAN.md) | v1.0 — Automated + manual scenarios |
| 12 | **Spreadsheet Formats** | [specs/SPREADSHEET_SPEC.md](specs/SPREADSHEET_SPEC.md) | v1.0 — XLSX, ODS, CSV, TSV, data model, formula engine |
| 13 | **Canvas Editor High-Level Architecture** | [CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md](CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md) | Draft — canvas-first editor architecture baseline |
| 14 | **Canvas Editor Low-Level Design** | [CANVAS_EDITOR_LOW_LEVEL_DESIGN.md](CANVAS_EDITOR_LOW_LEVEL_DESIGN.md) | Draft — rendering, input, scene, WASM API design |
| 15 | **Canvas Editor Elements Spec** | [CANVAS_EDITOR_ELEMENTS_SPEC.md](CANVAS_EDITOR_ELEMENTS_SPEC.md) | Draft — element-by-element ownership and behavior |
| 16 | **Canvas Editor Implementation Roadmap** | [CANVAS_EDITOR_IMPLEMENTATION_ROADMAP.md](CANVAS_EDITOR_IMPLEMENTATION_ROADMAP.md) | Draft — phased migration plan and exit criteria |
| 17 | **Canvas Editor WASM API Contract** | [CANVAS_EDITOR_WASM_API_CONTRACT.md](CANVAS_EDITOR_WASM_API_CONTRACT.md) | Draft — scene, geometry, edit, and composition boundary |
| 18 | **Canvas Editor Frontend Module Plan** | [CANVAS_EDITOR_FRONTEND_MODULE_PLAN.md](CANVAS_EDITOR_FRONTEND_MODULE_PLAN.md) | Draft — browser module split and migration map |
| 19 | **Canvas Editor Fidelity Validation Spec** | [CANVAS_EDITOR_FIDELITY_VALIDATION_SPEC.md](CANVAS_EDITOR_FIDELITY_VALIDATION_SPEC.md) | Draft — layout, render, interaction, and format fidelity gates |
| 20 | **Canvas Editor Fidelity Phase Checklist** | [CANVAS_EDITOR_FIDELITY_PHASE_CHECKLIST.md](CANVAS_EDITOR_FIDELITY_PHASE_CHECKLIST.md) | Draft — release gates and evidence checklist per canvas phase |

## Trackers

| Document | Purpose |
|----------|---------|
| [issues/COMPREHENSIVE_ISSUE_TRACKER.md](issues/COMPREHENSIVE_ISSUE_TRACKER.md) | Master issue tracker (92 items) |
| [issues/PHASED_ROADMAP.md](issues/PHASED_ROADMAP.md) | 6-phase roadmap with status |
| [issues/PHASE4_ENTERPRISE_TRACKER.md](issues/PHASE4_ENTERPRISE_TRACKER.md) | Enterprise features (46 steps) |
| [issues/PHASE5_ADVANCED_FORMATS_TRACKER.md](issues/PHASE5_ADVANCED_FORMATS_TRACKER.md) | Advanced format support (30 steps) |
| [issues/PHASE6_MULTI_APP_TRACKER.md](issues/PHASE6_MULTI_APP_TRACKER.md) | Multi-app suite (20 steps) |
| [issues/ZIP_PRESERVATION_TRACKER.md](issues/ZIP_PRESERVATION_TRACKER.md) | ZIP entry round-trip preservation |

## Development Process

```
1. RESEARCH    → Study standard + existing implementations
2. SPECIFY     → Write spec: happy path + edge cases + errors + targets
3. BREAK       → Adversarial thinking: what breaks this?
4. FIX SPEC    → Update spec to handle breaks
5. IMPLEMENT   → Code to spec (spec items = test cases)
6. TEST        → Unit + integration + E2E for every spec item
7. VERIFY      → Real documents from various office applications
```
