# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [1.0.0] - 2026-03-17

### Added

#### Infrastructure & Workspace (Phase 1)
- Cargo workspace with `api`, `ui`, and `common` crates.
- `common` crate: UUID newtypes (`NodeId`, `EdgeId`, `TagId`, `AttachmentId`, `PermissionId`),
  full domain DTOs (`Node`, `Edge`, `Tag`, `Attachment`, `Permission`), `EmberTroveError` enum.
- `api` crate: Axum 0.8 REST backend, `Config` (env-based), `AppState`, `ApiError → IntoResponse`,
  health-check route (`GET /api/health`), repository traits, object-store trait.
- `ui` crate: Leptos 0.8 CSR/WASM app shell — dark/light theme toggle (persisted in
  `localStorage`), sidebar navigation, empty component stubs.
- `migrations/001_initial.sql`: full PostgreSQL 16 schema — `nodes`, `edges`, `tags`,
  `node_tags`, `attachments`, `permissions` with all indexes and generated `tsvector` column.
- `deploy/docker-compose.yml`: local dev stack — PostgreSQL 16, MinIO, Keycloak 24.
- `deploy/Dockerfile.api` + `deploy/Dockerfile.ui`: multi-stage Docker builds (Rust → Debian
  trixie-slim runtime; nginx for UI).
- `deploy/k8s/`: Kubernetes manifests — Deployments, Services, Ingress, ConfigMap, Secrets,
  StatefulSets for PostgreSQL and MinIO.

#### Authentication (Phase 2)
- OIDC auth middleware via Keycloak — JWT validation, `realm_access.roles` extraction.
- `GET /api/auth/login` — redirects browser to Keycloak authorisation endpoint.
- `GET /api/auth/callback` — PKCE/code exchange, sets encrypted session cookie.
- `POST /api/auth/refresh` — silent token refresh via encrypted refresh-token cookie.
- `GET /api/auth/me` — returns `UserInfo` for the authenticated session.
- `POST /api/auth/logout` — clears session and refresh cookies.
- `OIDC_EXTERNAL_URL` config: rewrites Keycloak `authorization_endpoint` to a browser-reachable
  URL when the API runs inside Docker (internal URL ≠ browser URL).

#### Node CRUD + Markdown Editor (Phase 3)
- Full node CRUD: `GET/POST /api/nodes`, `GET/PUT/DELETE /api/nodes/{id}`.
- Slug-based lookup: `GET /api/nodes/slug/{slug}`.
- Node status field (`draft` / `published`) with filter on list endpoint.
- Split-pane Markdown editor with live preview (`pulldown-cmark` + `ammonia`).
- Node list view with status badges, tag chips, and inline tag filter.

#### Knowledge Graph — Edges & Tags (Phase 4)
- Edge CRUD: `GET/POST /api/edges`, `DELETE /api/edges/{id}`.
- Node neighbours: `GET /api/nodes/{id}/neighbors`.
- Node backlinks: `GET /api/nodes/{id}/backlinks`.
- Node edges: `GET /api/nodes/{id}/edges`.
- Tag CRUD: `GET/POST /api/tags`, `PUT/DELETE /api/tags/{id}`.
- Tag attachment: `POST/DELETE /api/nodes/{id}/tags/{tag_id}`.
- Node tags: `GET /api/nodes/{id}/tags`.
- Interactive force-directed graph view (SVG, Leptos) with directional arrowheads,
  edge-type colour coding, and node text contrast halo.
- Backlinks panel on node detail page.
- Graph node-position persistence: `GET/PUT /api/graph/positions`, `PUT /api/graph/positions/{node_id}`.

#### Full-Text & Fuzzy Search (Phase 5)
- `GET /api/search?q=…` — PostgreSQL `tsvector` full-text + `pg_trgm` trigram similarity.
- Optional filters: `status=published`, `tag_id=<uuid>`, `tag_ids=<uuid,uuid,…>`.
- Multi-tag AND/OR mode (`and_mode=true`) via a static parameterised SQL HAVING guard.
- Search results view with debounced reactive updates (300 ms, version-guarded).
- Unified sidebar `SearchBar` — triggers `SearchView`; tag-only browse supported
  (empty query + tag filter shows all nodes with that tag).

#### Attachments & S3 (Phase 6)
- File upload: `POST /api/nodes/{id}/attachments` (multipart).
- Attachment list: `GET /api/nodes/{id}/attachments`.
- Download: `GET /api/attachments/{id}/download` (streamed from S3).
- Delete: `DELETE /api/attachments/{id}`.
- S3-compatible backend (MinIO or AWS S3) via `aws-sdk-s3`.
- Attachment panel on node detail page (icon-only collapsed view).

#### Per-Node Permissions (Phase 7)
- Permission grant: `POST /api/nodes/{id}/permissions`.
- Permission list: `GET /api/nodes/{id}/permissions`.
- Owner / editor / viewer ACL backed by JWT claims.
- Permission panel with user-picker (populated from admin user list; falls back to raw UUID input).

#### Docker & Kubernetes Deployment (Phase 8)
- Multi-stage `Dockerfile.api`: dependency cache layer, workspace stub trick for layer reuse,
  `debian:trixie-slim` runtime.
- Multi-stage `Dockerfile.ui`: Trunk WASM build, nginx static file server.
- `deploy/k8s/`: complete Kubernetes manifests (Deployments, Services, HPA, Ingress,
  PersistentVolumeClaims, Secrets).
- nginx reverse-proxy config with `/api/` proxy pass to the API container.

#### Admin / User Management (Phase 8+)
- `GET /api/admin/users` — list Keycloak users (admin role required).
- `POST /api/admin/users` — create user with initial roles.
- `DELETE /api/admin/users/{id}` — delete user.
- `GET /api/admin/users/roles` — list available realm roles.
- `PUT /api/admin/users/{id}/roles` — set user roles.
- `KeycloakAdminClient` with `client_credentials` token caching (30 s buffer).
- Admin UI (`AdminView`): user table, create-user form, two-click delete, inline role editor.
- Sidebar admin link shown only to users with the `admin` realm role.

#### UX Polish
- Tag-browse without a search query: empty `SearchView` + tag filter shows all matching nodes.
- "Browse by tag →" shortcut link in sidebar.
- Clickable tag chips on node cards set global `tag_filter` context signal.
- `fetch_me` 401 guard: prevents infinite reload on genuine session expiry.
- Light / dark mode theme toggle (Tailwind v4, class-based, `localStorage`-persisted).

[Unreleased]: https://github.com/jchultarsky101/ember-trove/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/jchultarsky101/ember-trove/releases/tag/v1.0.0
