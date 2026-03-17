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

Follows standard Git Flow. `v1.0.0` is the first production tag on `main`.

| Branch type | Pattern              | Branched from | Merges into          | Notes                              |
|-------------|----------------------|---------------|----------------------|------------------------------------|
| Feature     | `feature/jc/<name>`  | `develop`     | `develop`            | `--no-ff`; worktree per feature    |
| Release     | `release/<version>`  | `develop`     | `main` + `develop`   | tag on `main` after merge          |
| Hotfix      | `hotfix/<name>`      | `main`        | `main` + `develop`   | tag bump on `main` after merge     |

- Persistent branches: `main` (production) and `develop` (integration).
- Features: `feature/jc/...` branched from `develop`, worked in
  `.claude/worktrees/<name>/`, merged back with `--no-ff`, worktree + branch
  deleted after merge.
- Releases: `release/<semver>` branched from `develop`; after QA, merge into
  `main` (`--no-ff`), tag (`v<semver>`), merge back into `develop`, delete branch.
- Hotfixes: `hotfix/<name>` branched from `main`; after fix, merge into `main`
  (`--no-ff`), tag patch bump, merge back into `develop`, delete branch.
- **Never commit directly to `main` or `develop`** — all changes via branches.
- **Current state**: v1.0.0 released. `develop` is the active integration branch.

## Environment Quirks

- **Docker PATH**: Binary at `/Applications/Docker.app/Contents/Resources/bin/docker`; always
  `export PATH="$PATH:/Applications/Docker.app/Contents/Resources/bin"` before any `docker` call.
- **`cargo` PATH**: Not on default shell PATH; always `export PATH="$HOME/.cargo/bin:$PATH"` in
  Bash tool calls.
- **`cat` aliased to `bat`**: Heredoc git commit messages (`-m "$(cat <<'EOF'...)"`) silently produce
  empty messages in this shell. Use plain multi-line `-m "..."` strings for all commits.
- **Docker build output**: BuildKit output does not stream to the task file in real-time; `tail` of
  the output file shows only the initial lines while building. Use `/bin/ps aux | grep docker` to
  confirm the build is still alive.
- **Stray Docker containers**: Old `docker compose` runs (e.g. `partorbital-*`) leave containers on
  a different network from `deploy-*`. Run `docker ps` and stop orphans before troubleshooting
  networking between services.
- **Keycloak usernames are read-only**: `kcadm.sh update users/<id> -s username=...` →
  `error-user-attribute-read-only`. Delete and recreate the user to rename.
- **Keycloak `set-password`**: `--temporary false` flag removed in recent KC — omit it entirely.
- **Worktree cwd resets**: Bash cwd resets to the session's worktree root between tool calls; always use
  absolute paths (e.g. `cd /Users/julian/projects/ember-trove && git ...`).
- **Worktree directory deleted → shell broken**: If the session worktree directory is deleted (e.g. by
  `rm -rf .claude/worktrees/`), the shell snapshot fails to `cd` there and **every subsequent Bash
  command silently fails** (exit code non-zero, only the cd error printed). Fix: use the `Write` tool
  to create a placeholder file at `<worktree-path>/.keep` — this recreates the directory and unblocks
  the shell immediately. Never delete the current session's worktree directory.
- **Docker single-service rebuild**: `docker compose -f deploy/docker-compose.yml build <svc> && docker compose -f deploy/docker-compose.yml up -d <svc>` — rebuilds one container without restarting others.
- **Verify merge state first**: At session start, run `git log --oneline -5` on `develop` to confirm what's already merged before re-doing work in a worktree.

## Leptos Patterns

- **Reactive Effect + async race**: `Effect::new` fires on every signal change (each keystroke).
  Any `spawn_local` inside must use a monotonic version counter (`RwSignal<u32>`) to discard stale
  responses, plus `gloo_timers::future::TimeoutFuture::new(300).await` debounce before the API call.
- **Shared context signals**: Lift `RwSignal<T>` to the App root, `provide_context(sig)` there,
  `use_context::<RwSignal<T>>()` in children. No prop-drilling. Example: `search_query` written by
  sidebar `SearchBar`, read by `SearchView`.
- **SearchBar suppress on Search view**: When `current_view == View::Search`, return early in
  `trigger_search` to suppress the dropdown — but still call `search_query.set(...)` first so the
  `SearchView` `Effect` fires and auto-searches.
- **Context signal type**: Carry full DTOs (e.g. `RwSignal<Option<Tag>>`) in context rather than
  just IDs — avoids extra fetches and lets any child render name/colour without a lookup.
- **move closure + String ownership**: In `map()` closures, clone String fields into named
  variables *before* the `view!` macro (e.g. `let name = tag.name.clone(); let title = format!("…{name}");`).
  The first use inside `view!` moves the String; a second use (e.g. in `title=`) will fail to compile.
- **Clippy `too_many_arguments`**: Private helper fns with ≥8 args trigger this. Annotate with
  `#[allow(clippy::too_many_arguments)]` when a params struct would be excessive.
- **SVG `attr:` prefix bug**: Leptos 0.8 writes `attr:foo=val` as `setAttribute("attr:foo", val)`
  (keeps the prefix!) for SVG elements. **Rule**: use `style="foo: val"` for ALL SVG presentation
  attributes (stroke-width, fill-opacity, text-anchor, font-size, marker-end, paint-order, etc.).
  Regular named attributes without hyphens (`stroke`, `fill`, `d`, `cx`, `cy`) work fine without `attr:`.
- **Unknown SVG elements (`<marker>`, `<defs>` content)**: Not in Leptos's element list. Create via
  `web_sys::Document::create_element_ns(Some("http://www.w3.org/2000/svg"), "marker")` and
  `set_attribute`. Inject after first render with `spawn_local(async { TimeoutFuture::new(50).await; inject(); })`.
- **Event delegation: `stop_propagation()` ineffective between Leptos handlers**: All Leptos
  handlers are registered at the document root. `ev.stop_propagation()` inside a child's handler
  does NOT prevent a parent's co-registered Leptos handler from firing. **Fix:** use a signal
  guard in the outer handler that is set by the inner handler (inner handlers fire first in
  bubbling order). Example: SVG pan guard `if drag_node.get_untracked().is_none()`.
- **Drag-vs-click disambiguation**: Use `RwSignal<bool> did_drag` — set `true` in `on:mousemove`
  during drag, check+clear in `on:click` to suppress the post-mouseup click event.
- **SVG marker re-injection guard**: In `spawn_local` marker injectors, check
  `svg.query_selector("defs marker").is_ok_and(|m| m.is_none())` before inserting — reactive
  signals can re-fire and duplicate markers if unguarded.

## Browser Testing (mcp__Claude_in_Chrome)

- **Checkbox clicks**: Coordinate-based clicks miss small checkboxes. Use `mcp__Claude_in_Chrome__find`
  to locate by description, then `left_click` via the returned `ref`.
- **`<select>` dropdowns**: Coordinate clicks don't open native selects. Use
  `mcp__Claude_in_Chrome__find` to get the `ref`, then `mcp__Claude_in_Chrome__form_input` with
  the option's value string to select an option reliably.
- **API signature grep before changing**: When adding a parameter to a shared API function
  (e.g. `search_nodes()`), grep all UI source files for the old call-site count before committing —
  missed callers cause a compile failure on the next check.

## PostgreSQL / Axum Patterns

- **`Query<T>` + `Vec<Uuid>`**: `axum::extract::Query` uses `serde_urlencoded` which cannot
  deserialize repeated query params into `Vec<T>`. Use `Option<String>` (comma-separated UUIDs)
  and parse server-side with a helper (`s.split(',').filter_map(|v| v.parse().ok()).collect()`).
- **Static AND/OR tag SQL**: Avoid dynamic query building by using
  `array_length($n::uuid[], 1) IS NULL` as a bypass guard (empty array → skip filter) combined
  with `HAVING (NOT $and_mode) OR COUNT(DISTINCT tag_id) = array_length($n::uuid[], 1)` to
  switch AND/OR logic — all in a single static parameterised query.

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
