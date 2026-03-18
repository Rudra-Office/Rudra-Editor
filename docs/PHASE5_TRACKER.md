# Phase 5: Auth, Permissions & Multi-tenancy

> Goal: Add authentication, document-level permissions, and tenant isolation.
> Created: 2026-03-19 | Depends on: Phase 3 (complete)

## Milestone 5.1 — Authentication

| ID | Task | Status |
|----|------|--------|
| P5-01 | JWT validation middleware (HS256 + RS256) | OPEN |
| P5-02 | API key validation middleware | OPEN |
| P5-03 | Anonymous access mode (configurable) | OPEN |

## Milestone 5.2 — Permissions

| ID | Task | Status |
|----|------|--------|
| P5-04 | Document permission model (owner/admin/editor/commenter/viewer) | OPEN |
| P5-05 | Permission enforcement on all API endpoints | OPEN |
| P5-06 | Sharing API (POST/DELETE/GET /documents/:id/share) | OPEN |

## Milestone 5.3 — Multi-tenancy

| ID | Task | Status |
|----|------|--------|
| P5-07 | Tenant isolation (tenant_id from JWT, scoped queries) | OPEN |
| P5-08 | Per-tenant configuration (quotas, rate limits) | OPEN |

---

## Resolution Log

| ID | Date | Description |
|----|------|-------------|
