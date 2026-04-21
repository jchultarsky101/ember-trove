# Guardrails — Ember Trove

Act as a Senior Rust Architect. We follow a **zero-panic, TDD-first** workflow.
Before finalising any file edit, run `cargo check` and `cargo clippy`.
Output only complete, idiomatic Rust. Use `thiserror` for all custom error types.

**Self-learning resources** (grep before debugging or writing new patterns):
- `.claude/ERRORS.md` — known compile/runtime error patterns and fixes
- `.claude/patterns/` — canonical code patterns (navigate, submit, debounce, double-opt)
- `.claude/rules/leptos.md` — Leptos-specific rules (auto-loaded for ui/ files)
- `.claude/rules/api.md` — API/backend rules (auto-loaded for api/ files)
- `.claude/ROADMAP.md` — current state, backlog, architecture decisions

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

## Post-Edit Commands

After any `.rs` edit in `api/` or `common/`:
```
cargo check && cargo clippy -- -D warnings
```

After any `.rs` edit in `ui/` (WASM):
```
cargo check -p ui --target wasm32-unknown-unknown
cargo clippy -p ui --target wasm32-unknown-unknown -- -D warnings
```

Before any `git commit`:
```
cargo test
```

Run `./scripts/verify.sh` for a full suite (all of the above + git status check).

## Release & CI monitoring (hard rule)

A release is not "shipped" until **every** GitHub Actions workflow on the
pushed ref is green. The pipeline runs `Release` + `CI` concurrently:
`Release` builds and deploys GHCR images, `CI` runs `check` / `clippy` /
`cargo audit` / migrations / docker-build against the same commit. A green
Release alongside a red CI still leaves master broken.

After `git push origin main develop --tags`:

1. Poll `gh run list --limit 6` until all runs for the release commit
   report `completed` (no `queued` / `in_progress` left). Use a Bash
   until-loop or Monitor for this — do not declare done on a partial view.
2. Any `failure` → read `gh run view <id> --log-failed`, fix the root
   cause, and ship a follow-up patch. Do **not** claim success while a
   workflow is red. Transient advisories (cargo-audit) count — suppress
   them explicitly via `--ignore <RUSTSEC-…>` with a dated rationale.
3. Only after every workflow is green is the release really done.

---

## Project: Ember Trove

A self-hosted, graph-centric personal knowledge management system.

| Layer    | Technology                          |
|----------|-------------------------------------|
| Backend  | Rust · Axum 0.8 · Tokio             |
| Frontend | Leptos 0.8 CSR/WASM · Tailwind v4   |
| Database | PostgreSQL 16 · sqlx 0.8            |
| Storage  | S3-compatible (MinIO / AWS S3)      |
| Auth     | OIDC via Cognito (AWS)              |
| Markdown | pulldown-cmark · ammonia            |
| OpenAPI  | utoipa + Swagger UI                 |

```
ember-trove/
├── Cargo.toml       # workspace: members = [api, ui, common]
├── common/          # shared DTOs, error types, ID newtypes
├── api/             # Axum REST backend (port 3003)
├── ui/              # Leptos/Trunk WASM frontend
├── migrations/      # sqlx migrations (auto-applied at API startup)
├── scripts/         # verify.sh, next-version.sh
└── deploy/          # Dockerfiles, docker-compose, K8s manifests
```

## Git Flow

| Branch type | Pattern              | Branched from | Merges into        | Notes                           |
|-------------|----------------------|---------------|--------------------|---------------------------------|
| Feature     | `feature/jc/<name>`  | `develop`     | `develop`          | `--no-ff`; worktree per feature |
| Release     | `release/<version>`  | `develop`     | `main` + `develop` | tag on `main` after merge       |
| Hotfix      | `hotfix/<name>`      | `main`        | `main` + `develop` | tag patch bump on `main`        |

- **Never commit directly to `main` or `develop`** — all changes via branches.
- Use `/release [major|minor|patch]` command for release workflow.
- Use `./scripts/next-version.sh` to compute the next semver automatically.

## Environment Quirks

- **Docker PATH**: `export PATH="$PATH:/Applications/Docker.app/Contents/Resources/bin"` before docker.
- **`cargo` PATH**: `export PATH="$HOME/.cargo/bin:$PATH"` in every Bash tool call.
- **`cat` aliased to `bat`**: Use plain `-m "..."` for git commit messages (not heredoc).
- **`grep`/`tail`/`head`/`rg` not available**: Use Grep tool; Read with offset/limit; `python3 -c` for JSON.
- **`aws` CLI unavailable**: Use `boto3` via `pip3 install boto3` + Python.
- **Worktree cwd resets**: Always use absolute paths in Bash tool calls.
- **Worktree dir deleted → shell broken**: `Write` a `.keep` file at `<path>/.keep` to fix, then `git worktree prune`.
- **Port 8003 conflict**: Check `lsof -i :8003` — stale Trunk process intercepts before Docker.
- **Docker force-recreate**: After image rebuild, use `docker compose up -d --force-recreate <svc>`.
- **BuildKit WASM cache**: `--no-cache` unreliable. Use `trunk build --release` + `docker cp` to bust.

## Leptos Navigation (v1.83.0+)

All navigation uses `leptos_router` 0.8 — browser back/forward works natively.

**NavigateFn is Clone, not Copy**. Wrap in `StoredValue` for reactive contexts:
```rust
let navigate = StoredValue::new(use_navigate());
navigate.get_value()("/path", Default::default());
```

Or clone before each inner `move ||` closure that captures it.
See `.claude/patterns/navigate-reactive.rs` for all patterns.

**Route paths require `path!()` macro**:
```rust
use leptos_router::path;
<Route path=path!("/tasks/inbox") view=|| view!{ <TasksView active=TasksTab::Inbox /> } />
```

**URL mapping**: `/tasks/my-day` · `/tasks/inbox` · `/tasks/calendar` · `/dashboard` · `/graph` · `/search` ·
`/notes` · `/nodes` · `/nodes/new` · `/nodes/:id` · `/nodes/:id/edit` ·
`/tags` · `/templates` · `/admin/users` · `/admin/permissions` · `/admin/backup`

Legacy `/my-day`, `/inbox`, `/calendar` URLs redirect to `/tasks/...` for
bookmarks and PWA shortcuts dating back before v2.3.0.

## Leptos Patterns (Critical)

- **Static `style=` / `title=`**: Always use closures — `style=move || ...` — for reactive attributes.
- **Reactive closure FnOnce**: Moving non-Copy values into inner closures breaks reactivity. See `.claude/ERRORS.md`.
- **Shared submit logic**: Use `RwSignal<bool>` trigger + `Effect::new`. See `.claude/patterns/submit-trigger.rs`.
- **Debounced search**: Version counter + 300ms timeout. See `.claude/patterns/reactive-effect-debounce.rs`.
- **Context newtypes**: `#[derive(Clone,Copy)] struct ShowCapture(pub RwSignal<bool>)` prevents collision.
- **SVG**: z-order = DOM order. Use `style=""` for hyphenated attrs (`stroke-width`, etc.), not `attr:`.
- **Tailwind v4 group-hover**: Unreliable. Use always-visible muted element + `:hover` color.
- **Map closures**: Clone signals/navigate before the inner `move ||` in each map iteration.
- **`MyDayTask` fields**: Accessed via `my_day_task.task.node_id` (nested via `#[serde(flatten)]`).

## PostgreSQL / Axum Patterns

- **`Query<T>` + `Vec<Uuid>`**: Use `Option<String>` + server-side `.split(',')` parse.
- **`node_type` serde**: Lowercase variants — `"article"`, `"project"`, `"area"`, etc.
- **`Option<Option<T>>` PATCH**: Use `deser_double_opt` deserializer. See `.claude/patterns/double-opt-patch.rs`.
- **Static AND/OR tag SQL**: `array_length($n::uuid[], 1) IS NULL` bypass + `HAVING` clause.

## Admin Permission Model

- **`require_role()`**: Returns `Ok(())` immediately for `claims.roles.contains("admin")`.
- **`list_nodes`**: Skips `subject_id` filter for admins.
- **`is_owner`**: `user.sub == n.owner_id || user.roles.contains("admin")`.
- **Admin sub**: `f1eb2590-0091-70e4-d9b3-24e4a23d24d1` (`julian@chultarsky.com`).

## Cognito Hosted UI

- `SetUICustomization` allowlist only — unlisted classes cause `InvalidParameterException`.
- Apply CSS via `boto3.client('cognito-idp').set_ui_customization(...)` (aws CLI unavailable).
- Authoritative CSS: `deploy/cognito.css` + `deploy/logo.png`.
- Pool: `us-east-2_4RQfxhKqn` · Client: `eogq2sehdad3uc8nmar7aneol`

## Production Deployment

- **Server**: `ubuntu@18.221.254.95` (SSH: `~/.ssh/lightsail-ember-trove.pem`)
- **CD pipeline**: tag push → GHA builds GHCR images → EC2 pulls + recreates containers.
- **Prod deploy**: `git push origin main develop --tags` → pipeline handles the rest.
- **Verify**: `curl https://trove.chultarsky.me/api/health`
- **Migrations**: Auto-run at API startup via `sqlx::migrate!()`.
- **Manual override**: `docker compose up -d --force-recreate <svc>` + `nginx -s reload`.

## Browser Testing (mcp__Claude_in_Chrome)

- **Checkbox/select**: Use `find` by description + `form_input` — coordinate clicks miss small targets.
- **API signature changes**: Grep all UI call sites before changing shared `api.rs` functions.
- **Tool timeouts**: Wait and retry — tab remains valid. Fall back to `open "<url>"` for navigation.
