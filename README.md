# Ember Trove

[![Build Status](https://img.shields.io/github/actions/workflow/status/jchultarsky101/ember-trove/ci.yml?branch=main&style=flat-square)](https://github.com/jchultarsky101/ember-trove/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/leptos-0.8-purple.svg?style=flat-square)](https://leptos.dev/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-336791.svg?style=flat-square)](https://www.postgresql.org/)

> A self-hosted, graph-centric personal knowledge management system — your second brain, written in Rust.

---

## Overview

Ember Trove is a web-based personal knowledge management (PKM) application where **nodes** (articles, projects, areas, resources, references) are linked by **typed edges**, tagged with flexible metadata, and searchable via full-text + fuzzy search. Markdown is the primary authoring format. Files can be attached to any node and stored in S3-compatible object storage. Access is secured per-node with OIDC (Keycloak).

### Key Features

- **Graph-centric** — nodes and typed directional edges form a navigable knowledge graph with a visual graph view.
- **Markdown-native** — split-pane editor with live preview, rendered via `pulldown-cmark` + `ammonia`.
- **Full-text + fuzzy search** — PostgreSQL `tsvector` full-text search and `pg_trgm` trigram similarity.
- **Per-node permissions** — owner / editor / viewer ACL backed by OIDC (Keycloak).
- **S3 attachments** — file upload / download via MinIO or AWS S3.
- **Light / dark mode** — class-based Tailwind v4 theme toggle persisted in `localStorage`.
- **Self-hosted** — fully Dockerised with a K8s deployment guide.

---

## Tech Stack

| Layer       | Technology                              |
|-------------|-----------------------------------------|
| Backend     | Rust · Axum 0.8 · Tokio                 |
| Frontend    | Leptos 0.8 CSR/WASM · Tailwind CSS v4   |
| Database    | PostgreSQL 16 · sqlx 0.8               |
| File Store  | S3-compatible (MinIO / AWS S3)          |
| Auth        | OIDC via Keycloak                       |
| Markdown    | pulldown-cmark · ammonia               |
| OpenAPI     | utoipa + Swagger UI                     |
| Build       | Trunk (UI) · cargo workspace            |
| Deploy      | Docker multi-stage · Kubernetes         |

---

## Workspace Structure

```
ember-trove/
├── Cargo.toml            # workspace (api, ui, common)
├── common/               # shared DTOs, error types, ID newtypes
├── api/                  # Axum REST backend  — port 3003
├── ui/                   # Leptos/Trunk WASM  — port 8003
├── migrations/           # sqlx migrations (PostgreSQL schema)
└── deploy/
    ├── Dockerfile.api
    ├── Dockerfile.ui
    ├── docker-compose.yml
    └── k8s/              # Kubernetes manifests
```

---

## Getting Started

### Prerequisites

| Tool              | Install                                              |
|-------------------|------------------------------------------------------|
| Rust (stable ≥ 1.82) | [rustup.rs](https://rustup.rs)                  |
| wasm32 target     | `rustup target add wasm32-unknown-unknown`           |
| Trunk             | `cargo install trunk`                                |
| sqlx-cli          | `cargo install sqlx-cli --features postgres`         |
| Docker + Compose  | [docs.docker.com](https://docs.docker.com/get-docker/) |

---

### Quick Start (Phase 1 — UI shell, no auth required)

Phase 1 stubs out auth, S3, and all repo logic. You only need **PostgreSQL** running.

**Step 1 — Start PostgreSQL**

```bash
docker compose -f deploy/docker-compose.yml up -d postgres
```

This starts a `postgres:16` container on the default port 5432 with:
- database: `ember_trove`
- user / password: `ember_trove` / `ember_trove_dev`

**Step 2 — Apply migrations**

```bash
export DATABASE_URL=postgres://ember_trove:ember_trove_dev@localhost/ember_trove
sqlx migrate run --source migrations/
```

**Step 3 — Start the API** (Terminal 1)

```bash
export DATABASE_URL=postgres://ember_trove:ember_trove_dev@localhost/ember_trove
cargo run -p api
```

You should see:
```
INFO ember_trove_api: listening on 0.0.0.0:3003
```

Verify with:
```bash
curl http://localhost:3003/health
# {"status":"ok","service":"ember-trove-api"}
```

**Step 4 — Start the UI dev server** (Terminal 2)

```bash
cd ui
trunk serve --port 8003
```

Trunk compiles the WASM bundle and watches for changes. First build takes ~30 s.

You should see:
```
INFO  📡 server listening at: http://127.0.0.1:8003
```

**Step 5 — Open the browser**

Navigate to **http://localhost:8003**

You'll see the Ember Trove shell:
- Left sidebar with navigation (Articles, Projects, Areas, Resources, References, Graph, Search, Tags)
- Dark / light mode toggle (top-left, persisted in `localStorage`)
- Main panel showing the empty node list

> **Note:** In Phase 1 all data operations return `501 Not Implemented`. Full CRUD, auth, and search are implemented in Phases 2–7.

---

### Full Stack (all services)

To run with Keycloak (OIDC) and MinIO (S3) as well:

```bash
docker compose -f deploy/docker-compose.yml up -d
```

Services started:

| Service   | URL                        |
|-----------|----------------------------|
| PostgreSQL | `localhost:5432`          |
| MinIO      | http://localhost:9000      |
| Keycloak   | http://localhost:8080      |
| API        | http://localhost:3003      |
| UI         | http://localhost:8003      |

### Cargo Check

```bash
# Backend + common (host target)
cargo check && cargo clippy -- -D warnings

# WASM UI
cargo check -p ui --target wasm32-unknown-unknown
cargo clippy -p ui --target wasm32-unknown-unknown -- -D warnings
```

---

## Domain Model

### Node Types

| Type        | Description                                    |
|-------------|------------------------------------------------|
| `article`   | Blog post, essay, or atomic note               |
| `project`   | Active initiative with tasks and references    |
| `area`      | Sphere of responsibility (ongoing)             |
| `resource`  | Reference material, bookmark, or asset         |
| `reference` | Citation, paper, or external source            |

### Edge Types

| Type           | Meaning                                |
|----------------|----------------------------------------|
| `references`   | Node A cites / links to Node B         |
| `contains`     | Node A structurally contains Node B    |
| `related_to`   | Bidirectional semantic relationship    |
| `depends_on`   | Node A requires Node B                 |
| `derived_from` | Node A was derived from Node B         |

---

## API Reference

Interactive docs are served at `/swagger-ui/` when the API is running.

| Method | Path                        | Description                       |
|--------|-----------------------------|-----------------------------------|
| GET    | `/health`                   | Health check                      |
| GET    | `/auth/login`               | Redirect to Keycloak              |
| GET    | `/auth/callback`            | OIDC code exchange                |
| GET    | `/nodes`                    | List nodes (filter, sort, page)   |
| POST   | `/nodes`                    | Create node                       |
| GET    | `/nodes/{id}`               | Get node by ID                    |
| PUT    | `/nodes/{id}`               | Update node                       |
| DELETE | `/nodes/{id}`               | Delete node (cascading)           |
| GET    | `/nodes/{id}/neighbors`     | Linked nodes                      |
| POST   | `/edges`                    | Create edge                       |
| GET    | `/tags`                     | List tags                         |
| GET    | `/search?q=...`             | Full-text + fuzzy search          |

---

## Configuration

All API configuration is loaded from environment variables.
Variables marked **optional\*** are not required until the noted phase.

| Variable             | Default       | Required    | Description                    |
|----------------------|---------------|-------------|--------------------------------|
| `DATABASE_URL`       | —             | Always      | PostgreSQL connection string   |
| `HOST`               | `0.0.0.0`     | Always      | Bind address                   |
| `PORT`               | `3003`        | Always      | Bind port                      |
| `OIDC_ISSUER`        | —             | Phase 2+    | Keycloak realm issuer URL      |
| `OIDC_CLIENT_ID`     | —             | Phase 2+    | OIDC client ID                 |
| `OIDC_CLIENT_SECRET` | —             | Phase 2+    | OIDC client secret             |
| `S3_ENDPOINT`        | —             | Phase 6+    | S3-compatible endpoint URL     |
| `S3_BUCKET`          | —             | Phase 6+    | Bucket name                    |
| `S3_ACCESS_KEY`      | —             | Phase 6+    | S3 access key                  |
| `S3_SECRET_KEY`      | —             | Phase 6+    | S3 secret key                  |
| `S3_REGION`          | `us-east-1`   | Phase 6+    | S3 region                      |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

MIT — see [LICENSE](LICENSE).
