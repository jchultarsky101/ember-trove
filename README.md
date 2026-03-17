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
- **Multi-tag filtering** — AND/OR tag filters across node list and search results.
- **Per-node permissions** — owner / editor / viewer ACL backed by OIDC (Keycloak).
- **S3 attachments** — file upload / download via MinIO or AWS S3.
- **User management** — admin UI backed by Keycloak Admin REST API.
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

## Local Development — Step-by-Step

This section walks you through building and running every service manually on your local machine so you can access the app in a browser.

### Prerequisites

Install the following tools before proceeding:

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable ≥ 1.91.1 | [rustup.rs](https://rustup.rs) |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| Trunk | latest | `cargo install trunk` |
| sqlx-cli | latest | `cargo install sqlx-cli --features postgres` |
| Docker Desktop | latest | [docs.docker.com/get-docker](https://docs.docker.com/get-docker/) |

> **Note:** `aws-sdk-s3` requires Rust ≥ 1.91.1. Run `rustup update stable` if your toolchain is older.

---

### Step 1 — Start the backing services (PostgreSQL, MinIO, Keycloak)

The docker-compose file provides pre-configured containers for all three backing services.

```bash
docker compose -f deploy/docker-compose.yml up -d postgres minio keycloak
```

Wait ~15 seconds for Keycloak to finish starting, then verify all three are healthy:

```bash
docker compose -f deploy/docker-compose.yml ps
```

Expected output:

```
NAME                    STATUS
deploy-postgres-1       running (healthy)
deploy-minio-1          running (healthy)
deploy-keycloak-1       running
```

Service URLs once running:

| Service    | URL                          | Credentials                         |
|------------|------------------------------|--------------------------------------|
| PostgreSQL | `localhost:5432`             | `ember_trove` / `ember_trove_dev`    |
| MinIO UI   | http://localhost:9001        | `ember_trove` / `ember_trove_dev`    |
| Keycloak   | http://localhost:8180        | `admin` / `admin` (master realm)     |

---

### Step 2 — Configure Keycloak

Keycloak uses an in-memory dev store (`KC_DB: dev-file`). You must recreate the realm and client after every Keycloak container restart.

Run the following commands to set up the realm, client, and a test user. All commands execute inside the Keycloak container using `kcadm.sh`.

**2a. Authenticate kcadm as the master admin:**

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh config credentials \
    --server http://localhost:8080 \
    --realm master \
    --user admin \
    --password admin
```

**2b. Create the `ember-trove` realm:**

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh create realms \
    -s realm=ember-trove \
    -s enabled=true
```

**2c. Create the `ember-trove-api` OIDC client:**

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh create clients \
    -r ember-trove \
    -s clientId=ember-trove-api \
    -s enabled=true \
    -s publicClient=false \
    -s secret=change-me \
    -s 'redirectUris=["http://localhost:3003/api/auth/callback","http://localhost:8003/api/auth/callback"]' \
    -s directAccessGrantsEnabled=true
```

**2d. Disable PKCE on the client** (required for the confidential client flow):

First, get the client's internal UUID:

```bash
CLIENT_UUID=$(docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh get clients \
    -r ember-trove \
    --fields id,clientId \
    -q clientId=ember-trove-api \
  | python3 -c "import sys,json; print(json.load(sys.stdin)[0]['id'])")
echo "Client UUID: $CLIENT_UUID"
```

Then clear the PKCE attribute:

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh update clients/$CLIENT_UUID \
    -r ember-trove \
    -s 'attributes={"pkce.code.challenge.method":""}'
```

**2e. Create the `admin` realm role:**

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh create roles \
    -r ember-trove \
    -s name=admin
```

**2f. Create a test user:**

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh create users \
    -r ember-trove \
    -s username=testuser \
    -s email=test@example.com \
    -s firstName=Test \
    -s lastName=User \
    -s enabled=true
```

**2g. Set the test user's password:**

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh set-password \
    -r ember-trove \
    --username testuser \
    --new-password Ember2026
```

**2h. Assign the `admin` role to the test user** (optional — needed to access the Admin UI):

```bash
docker exec deploy-keycloak-1 \
  /opt/keycloak/bin/kcadm.sh add-roles \
    -r ember-trove \
    --uusername testuser \
    --rolename admin
```

Verify by navigating to **http://localhost:8180** → Ember-Trove realm → Users → testuser → Role Mappings.

---

### Step 3 — Apply database migrations

With PostgreSQL running, apply the SQL schema:

```bash
export DATABASE_URL=postgres://ember_trove:ember_trove_dev@localhost/ember_trove
sqlx migrate run --source migrations/
```

Expected output:

```
Applied 1/migrate initial (Xms)
```

---

### Step 4 — Create the MinIO bucket

The API requires the `ember-trove` bucket to exist before uploading attachments.

```bash
docker exec deploy-minio-1 mc alias set local http://localhost:9000 ember_trove ember_trove_dev
docker exec deploy-minio-1 mc mb local/ember-trove
```

If the bucket already exists you will see `Bucket created successfully. \`local/ember-trove\`` or a `Your previous request to create the named bucket succeeded` error — both are fine.

Alternatively, log into the MinIO console at **http://localhost:9001** (credentials: `ember_trove` / `ember_trove_dev`), navigate to **Buckets → Create Bucket**, and enter `ember-trove`.

---

### Step 5 — Build and run the API

Set the required environment variables and start the API server:

```bash
export DATABASE_URL=postgres://ember_trove:ember_trove_dev@localhost/ember_trove
export S3_ENDPOINT=http://localhost:9000
export S3_BUCKET=ember-trove
export S3_ACCESS_KEY=ember_trove
export S3_SECRET_KEY=ember_trove_dev
export S3_REGION=us-east-1
export OIDC_ISSUER=http://localhost:8180/realms/ember-trove
export OIDC_CLIENT_ID=ember-trove-api
export OIDC_CLIENT_SECRET=change-me
export FRONTEND_URL=http://localhost:8003
export API_EXTERNAL_URL=http://localhost:3003
export KEYCLOAK_ADMIN_USER=admin
export KEYCLOAK_ADMIN_PASSWORD=admin
export COOKIE_KEY=00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
export RUST_LOG=info

cargo run -p api
```

> **COOKIE_KEY** must be exactly 128 hex characters (64 bytes). The all-zeros value above is safe for local development; **change it in any shared or production environment**.

Successful start output:

```
INFO ember_trove_api: database migrations complete
INFO ember_trove_api: Keycloak admin client enabled (realm: ember-trove)
INFO ember_trove_api: ember-trove-api listening on 0.0.0.0:3003
```

Verify with:

```bash
curl http://localhost:3003/api/health
# {"status":"ok","service":"ember-trove-api","database":"ok"}
```

Interactive API docs (Swagger UI) are available at **http://localhost:3003/swagger-ui/**.

---

### Step 6 — Build and run the UI dev server

In a second terminal:

```bash
trunk serve --config ui/Trunk.toml
```

Trunk compiles the WASM bundle, starts a dev server with hot-reload, and proxies `/api/*` requests to the API. The first build takes ~60 s.

Successful start output:

```
INFO  📡 server listening at: http://127.0.0.1:8003
```

---

### Step 7 — Open the app in a browser

Navigate to **http://localhost:8003**.

You will be redirected to Keycloak. Log in with:

- **Username:** `testuser`
- **Password:** `Ember2026`

After login you are redirected back to the Ember Trove main view.

**What you can do:**

| Feature | How to access |
|---------|--------------|
| Create a node | Click **+** in the sidebar next to any node type |
| Edit Markdown | Click a node → split-pane editor |
| Tag nodes | Open a node → Tags section |
| Browse by tag | Click **Browse by tag →** in the sidebar |
| Search | Type in the search bar (top of sidebar) → Enter |
| View graph | Click **Graph** in the sidebar |
| Upload attachment | Open a node → Attachment panel (paperclip icon) |
| Manage users | Click **Admin** in the sidebar (admin role required) |

---

## Running the Full Docker Stack

To run everything (API + UI + all services) with a single command:

```bash
docker compose -f deploy/docker-compose.yml up --build
```

> **Important:** After `docker compose up`, Keycloak starts with an empty database. You must re-run the Keycloak setup commands in [Step 2](#step-2--configure-keycloak) after every fresh container start.

Services and ports:

| Service    | URL                    | Notes                              |
|------------|------------------------|------------------------------------|
| UI (nginx) | http://localhost:8003  | Entry point — open this in browser |
| API        | http://localhost:3003  | `/api/health`, Swagger at `/swagger-ui/` |
| Keycloak   | http://localhost:8180  | Admin: `admin` / `admin`           |
| MinIO API  | http://localhost:9000  | S3-compatible endpoint             |
| MinIO UI   | http://localhost:9001  | `ember_trove` / `ember_trove_dev`  |
| PostgreSQL | `localhost:5432`       | `ember_trove` / `ember_trove_dev`  |

To rebuild a single service after a code change:

```bash
docker compose -f deploy/docker-compose.yml build api
docker compose -f deploy/docker-compose.yml up -d api
```

---

## Configuration Reference

All API configuration is provided via environment variables.

| Variable                 | Default       | Required  | Description                                         |
|--------------------------|---------------|-----------|-----------------------------------------------------|
| `DATABASE_URL`           | —             | Always    | PostgreSQL connection string                        |
| `COOKIE_KEY`             | —             | Always    | 128 hex chars (64 bytes) for cookie encryption      |
| `FRONTEND_URL`           | —             | Always    | Browser-facing URL of the UI (used for CORS)        |
| `API_EXTERNAL_URL`       | —             | Always    | Browser-facing URL of the API (used for OIDC redirect URI) |
| `HOST`                   | `0.0.0.0`     | No        | Bind address                                        |
| `PORT`                   | `3003`        | No        | Bind port                                           |
| `RUST_LOG`               | `info`        | No        | Log level (`error`, `warn`, `info`, `debug`, `trace`) |
| `OIDC_ISSUER`            | —             | Auth      | Keycloak realm issuer URL, e.g. `http://localhost:8180/realms/ember-trove` |
| `OIDC_CLIENT_ID`         | —             | Auth      | OIDC client ID (`ember-trove-api`)                  |
| `OIDC_CLIENT_SECRET`     | —             | Auth      | OIDC client secret                                  |
| `OIDC_EXTERNAL_URL`      | —             | Docker    | Browser-reachable Keycloak base URL; rewrites internal discovery URL |
| `S3_ENDPOINT`            | —             | S3        | S3-compatible endpoint URL (e.g. `http://localhost:9000`) |
| `S3_BUCKET`              | —             | S3        | Bucket name                                         |
| `S3_ACCESS_KEY`          | —             | S3        | S3 access key                                       |
| `S3_SECRET_KEY`          | —             | S3        | S3 secret key                                       |
| `S3_REGION`              | `us-east-1`   | No        | S3 region                                           |
| `KEYCLOAK_ADMIN_USER`    | —             | Admin API | Keycloak master realm admin username                |
| `KEYCLOAK_ADMIN_PASSWORD`| —             | Admin API | Keycloak master realm admin password                |

> **OIDC_EXTERNAL_URL** is needed only when the API runs inside Docker and Keycloak is mapped to a non-default host port (e.g. `8180`). OIDC discovery returns `authorization_endpoint` with the internal Docker hostname; this variable rewrites it to the browser-reachable URL.

---

## Cargo Build & Check Commands

```bash
# Backend + common (host target)
cargo check
cargo clippy -- -D warnings
cargo test

# WASM UI
cargo check -p ui --target wasm32-unknown-unknown
cargo clippy -p ui --target wasm32-unknown-unknown -- -D warnings
```

---

## API Reference

All routes are nested under `/api`. Interactive docs at `/swagger-ui/` when the API is running.

### Auth (public — no JWT required)

| Method | Path                   | Description                          |
|--------|------------------------|--------------------------------------|
| GET    | `/api/auth/login`      | Redirect to Keycloak login           |
| GET    | `/api/auth/callback`   | OIDC code exchange; sets session cookie |
| POST   | `/api/auth/refresh`    | Silent token refresh                 |

### Auth (protected — JWT required)

| Method | Path                   | Description                          |
|--------|------------------------|--------------------------------------|
| GET    | `/api/auth/me`         | Current user info and roles          |
| POST   | `/api/auth/logout`     | Clear session and refresh cookies    |

### Health

| Method | Path           | Description                              |
|--------|----------------|------------------------------------------|
| GET    | `/api/health`  | Service health + database connectivity   |

### Nodes

| Method | Path                              | Description                              |
|--------|-----------------------------------|------------------------------------------|
| GET    | `/api/nodes`                      | List nodes (`status`, `tag_id`, `tag_ids`, pagination) |
| POST   | `/api/nodes`                      | Create node                              |
| GET    | `/api/nodes/{id}`                 | Get node by UUID                         |
| GET    | `/api/nodes/slug/{slug}`          | Get node by slug                         |
| PUT    | `/api/nodes/{id}`                 | Update node                              |
| DELETE | `/api/nodes/{id}`                 | Delete node (cascading)                  |
| GET    | `/api/nodes/{id}/neighbors`       | Linked neighbour nodes                   |
| GET    | `/api/nodes/{id}/backlinks`       | Nodes that link to this node             |
| GET    | `/api/nodes/{id}/edges`           | All edges involving this node            |
| GET    | `/api/nodes/{id}/tags`            | Tags attached to this node               |
| POST   | `/api/nodes/{id}/tags/{tag_id}`   | Attach a tag to a node                   |
| DELETE | `/api/nodes/{id}/tags/{tag_id}`   | Detach a tag from a node                 |
| GET    | `/api/nodes/{id}/attachments`     | List attachments                         |
| POST   | `/api/nodes/{id}/attachments`     | Upload attachment (multipart)            |
| GET    | `/api/nodes/{id}/permissions`     | List permissions                         |
| POST   | `/api/nodes/{id}/permissions`     | Grant permission to a user               |

### Edges

| Method | Path               | Description        |
|--------|--------------------|--------------------|
| GET    | `/api/edges`       | List all edges     |
| POST   | `/api/edges`       | Create edge        |
| DELETE | `/api/edges/{id}`  | Delete edge        |

### Tags

| Method | Path               | Description        |
|--------|--------------------|--------------------|
| GET    | `/api/tags`        | List all tags      |
| POST   | `/api/tags`        | Create tag         |
| PUT    | `/api/tags/{id}`   | Update tag         |
| DELETE | `/api/tags/{id}`   | Delete tag         |

### Search

| Method | Path                                         | Description                            |
|--------|----------------------------------------------|----------------------------------------|
| GET    | `/api/search?q=…`                            | Full-text + fuzzy search               |
| GET    | `/api/search?q=…&status=published`           | Filter by node status                  |
| GET    | `/api/search?q=…&tag_id={uuid}`              | Filter by single tag                   |
| GET    | `/api/search?q=…&tag_ids={uuid,uuid}`        | Filter by multiple tags (OR mode)      |
| GET    | `/api/search?q=…&tag_ids={uuid,uuid}&and_mode=true` | Multi-tag AND mode               |

### Attachments

| Method | Path                             | Description                     |
|--------|----------------------------------|---------------------------------|
| GET    | `/api/attachments/{id}/download` | Stream attachment bytes from S3 |
| DELETE | `/api/attachments/{id}`          | Delete attachment                |

### Graph

| Method | Path                               | Description                         |
|--------|------------------------------------|-------------------------------------|
| GET    | `/api/graph/positions`             | List saved node positions           |
| PUT    | `/api/graph/positions/{node_id}`   | Save / update a node position       |

### Admin (admin role required)

| Method | Path                           | Description                        |
|--------|--------------------------------|------------------------------------|
| GET    | `/api/admin/users`             | List Keycloak users                |
| POST   | `/api/admin/users`             | Create user                        |
| DELETE | `/api/admin/users/{id}`        | Delete user                        |
| GET    | `/api/admin/users/roles`       | List available realm roles         |
| PUT    | `/api/admin/users/{id}/roles`  | Set roles for a user               |

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

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

MIT — see [LICENSE](LICENSE).
