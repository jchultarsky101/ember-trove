# Guardrails — Ember Trove

Act as a Senior Rust Architect. We follow a **zero-panic, TDD-first** workflow.
Before finalising any file edit, run `cargo check` and `cargo clippy`.
Output only complete, idiomatic Rust. Use `thiserror` for all custom error types.

---

## Performance & Personality

- **Dense Mode**: Minimal conversational fluff; focus on production-ready code.
- **No placeholders** (`// ...`): All code must be complete and compilable.

## Safety & Idioms

- **No Panics**: Never use `.unwrap()` or `panic!`. Use `Result`/`Option` with `?`.
- **Error Handling**: `thiserror` for library errors, `anyhow` for application-level.
- **Ownership**: Follow borrow-checker rules. Prefer owned types initially.
- **Dependencies**: Check `Cargo.toml` before adding crates. Prefer `std`.

## Development Workflow (TDD)

1. **Red** — Write a failing test in `tests/` or a `mod tests` block.
2. **Green** — Implement the minimal logic to pass the test.
3. **Refactor** — `cargo clippy -- -D warnings` + `cargo fmt`.

## Post-Edit Command

```
cargo check && cargo clippy -- -D warnings
```

For the WASM UI crate:

```
cargo check -p ui --target wasm32-unknown-unknown
cargo clippy -p ui --target wasm32-unknown-unknown -- -D warnings
```

---

## Project: Ember Trove

A self-hosted, graph-centric personal knowledge management system.

| Layer    | Technology                         |
|----------|------------------------------------|
| Backend  | Rust · Axum 0.8 · Tokio            |
| Frontend | Leptos 0.8 CSR/WASM · Tailwind v4  |
| Database | PostgreSQL 16 · sqlx 0.8           |
| Storage  | S3-compatible (MinIO / AWS S3)     |
| Auth     | OIDC via Keycloak                  |
| Markdown | pulldown-cmark · ammonia           |
| OpenAPI  | utoipa + Swagger UI                |

```
ember-trove/
├── Cargo.toml       # workspace: members = [api, ui, common]
├── common/          # shared DTOs, error types, ID newtypes
├── api/             # Axum REST backend (port 3000)
├── ui/              # Leptos/Trunk WASM frontend
├── migrations/      # sqlx migrations
└── deploy/          # Dockerfiles, docker-compose, K8s manifests
```

## Git Flow

- Persistent branches: `main` and `develop` only.
- Features: `feature/jc/...` branched from `develop`, worked in
  `.claude/worktrees/<name>/`, merged back with `--no-ff`, worktree + branch
  deleted after merge.
- **Current state**: Phase 1 skeleton in progress.

## Implementation Phases

| Phase | Scope                                    |
|-------|------------------------------------------|
| 1     | Workspace skeleton, DTOs, health route, migrations, deploy |
| 2     | OIDC auth middleware, login/callback/refresh               |
| 3     | Node CRUD + Markdown editor UI                             |
| 4     | Knowledge graph: Edges + Tags                              |
| 5     | Full-text + fuzzy search                                   |
| 6     | Attachments + S3 integration                               |
| 7     | Per-node permissions                                       |
| 8     | Docker multi-stage + K8s deployment                        |
