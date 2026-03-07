# Contributing to Ember Trove

Thank you for your interest in contributing! Please read this guide before opening
a pull request.

---

## Code of Conduct

Be respectful and constructive. Harassment of any kind is not tolerated.

---

## Development Setup

### Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install sqlx-cli --features postgres
```

### Local Stack

```bash
docker compose -f deploy/docker-compose.yml up -d postgres minio keycloak
export DATABASE_URL=postgres://ember_trove:ember_trove_dev@localhost/ember_trove
sqlx migrate run --source migrations/
```

---

## Guardrails (see CLAUDE.md)

- **Zero panics** — no `.unwrap()`, no `panic!`. Use `?` everywhere.
- **`thiserror`** for all custom error types.
- **TDD** — write a failing test first.
- Run before every commit:

```bash
cargo check && cargo clippy -- -D warnings && cargo fmt --check
cargo test
```

---

## Git Flow

| Branch type | Pattern                  | Notes                               |
|-------------|--------------------------|-------------------------------------|
| Stable      | `main`                   | Tagged releases only                |
| Integration | `develop`                | Merged features land here           |
| Feature     | `feature/jc/<name>`      | Branch from `develop`; short-lived  |
| Fix         | `fix/jc/<name>`          | Branch from `develop`               |
| Release     | `release/x.y.z`          | Bump version, update CHANGELOG      |

- Merge features back to `develop` with `--no-ff`.
- Never force-push `main` or `develop`.

---

## Pull Request Checklist

- [ ] Tests pass (`cargo test`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt`)
- [ ] CHANGELOG.md updated under `[Unreleased]`
- [ ] New environment variables documented in README

---

## Commit Message Style

```
<type>(<scope>): <short description>

[optional body]

Co-Authored-By: <name> <email>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `ci`.

Example:

```
feat(api): add node CRUD endpoints with permission checks
```
