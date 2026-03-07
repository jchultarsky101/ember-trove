# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Cargo workspace with `api`, `ui`, and `common` crates.
- `common` crate: UUID newtypes (`NodeId`, `EdgeId`, `TagId`, `AttachmentId`,
  `PermissionId`), full domain DTOs (`Node`, `Edge`, `Tag`, `Attachment`,
  `Permission`), `EmberTroveError` enum.
- `api` crate skeleton: `Config` (env-based), `AppState`, `ApiError` →
  `IntoResponse`, health-check route (`GET /health`), stub repositories
  and object-store traits, stub route modules.
- `ui` crate skeleton: Leptos 0.8 CSR app shell with dark/light theme toggle,
  layout, sidebar, and empty component stubs.
- `migrations/001_initial.sql`: full PostgreSQL schema — `nodes`, `edges`,
  `tags`, `node_tags`, `attachments`, `permissions` with all indexes and
  generated `tsvector` column.
- `deploy/docker-compose.yml`: local dev stack — PostgreSQL 16, MinIO, Keycloak.
- `deploy/Dockerfile.api` + `deploy/Dockerfile.ui`: multi-stage Docker builds.
- `deploy/k8s/`: Kubernetes manifests — Deployments, Services, Ingress,
  ConfigMap, Secrets, StatefulSets for PostgreSQL and MinIO.
- `CLAUDE.md` guardrails (zero-panic, TDD-first, `thiserror` everywhere).

[Unreleased]: https://github.com/jchultarsky101/ember-trove/compare/HEAD...HEAD
