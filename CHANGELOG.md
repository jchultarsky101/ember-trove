# Changelog

All notable changes to Ember Trove are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

## [1.20.0] - 2026-03-23

### Added
- **Local development workflow**: `docker-compose.yml` now supports a fully self-contained local stack with one command:
  `docker compose -f deploy/docker-compose.yml --env-file deploy/.env.local up --build`
- **`minio-init` service**: auto-creates the `ember-trove` S3 bucket on first boot so attachment uploads work without any manual MinIO setup.
- **`deploy/.env.local.example`**: committed template documenting the three variables that need real values (`OIDC_CLIENT_SECRET`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`).
- **Cognito localhost callback**: registered `http://localhost:8003/api/auth/callback` and `http://localhost:8003` as allowed redirect/logout URLs so Cognito OIDC authentication works in the local Docker stack.

### Fixed
- **`API_EXTERNAL_URL` for local dev**: corrected from `:3003` (direct API port) to `:8003` (nginx proxy) so OIDC redirect URIs match the registered Cognito callback.
- **`cargo audit` paste warning silenced**: `RUSTSEC-2024-0436` (`paste` unmaintained, warning-level only via Leptos transitive dep) added to ignore list — Leptos owns that upgrade path.
- **`tar` 0.4.45 in `Cargo.lock`**: carried forward from v1.19.1 patch.

## [1.19.3] - 2026-03-23

### Fixed
- **Deploy concurrency guard**: added `concurrency: group: production-deploy, cancel-in-progress: true` to `release.yml` so rapid successive tag pushes no longer pile up concurrent Docker builds on the Lightsail VM.

## [1.19.2] - 2026-03-23

### Fixed
- **Production deploy timeout extended to 60 minutes**: Rust rebuild on a cold Lightsail VM regularly exceeded the previous 30-minute SSH timeout, causing deploy failures even when the build was progressing normally.

## [1.19.1] - 2026-03-23

### Fixed
- **Patched `tar` 0.4.44→0.4.45** (RUSTSEC-2026-0067: `unpack_in` symlink chmod; RUSTSEC-2026-0068: PAX size header parsing — both medium severity).

## [1.19.0] - 2026-03-23

### Added
- **`cargo audit` job in CI**: scans `Cargo.lock` against the RustSec advisory database on every push; blocks merges when fixable vulnerabilities are present.
- **Migration validation job in CI**: runs `sqlx migrate run` against an ephemeral Postgres 16 service container on every push to catch SQL errors before deploy.
- **Docker build validation job in CI**: builds both `api` and `ui` images (no push) using GitHub Actions layer cache to catch `Dockerfile` errors in CI.
- **Automated production deploy in `release.yml`**: pushing a version tag now SSHs into the Lightsail server, rebuilds images, restarts services, and verifies health — controlled by the `DEPLOY_ENABLED` repository variable.

### Fixed
- **`release.yml` no longer fails on every branch push**: the `secrets` context is not valid in job-level `if` conditions; switched to `vars.DEPLOY_ENABLED` (repository variables are allowed at job level).
- **"Add to Favorites" dialog now centers on the full screen**: Tailwind's `translate-x-0` left a `transform: translateX(0)` on the sidebar even on desktop, creating a CSS stacking context that trapped `position: fixed` overlays inside the sidebar bounds. Added `md:transform-none` to remove the transform at the desktop breakpoint; mobile slide animation is unaffected.
- **Patched `aws-lc-sys` 0.38→0.39** (RUSTSEC-2026-0048/0044, high severity) and **`rustls-webpki` 0.103.9→0.103.10** (RUSTSEC-2026-0049).

### Changed
- **Rust toolchain pinned to 1.92** via `rust-toolchain.toml` for reproducible CI builds (AWS SDK requires ≥ 1.91.1).
- **GitHub Actions opted into Node.js 24** via `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true`; removes deprecation warnings ahead of GitHub's June 2026 forced migration.

## [1.18.0] - 2026-03-22

### Fixed
- **PKCE (S256) added to OIDC login flow**: Cognito app clients created after November 2024 silently reject token exchanges without PKCE (`invalid_grant`). Login now generates a `code_verifier` (32 random bytes, URL-safe base64), derives `code_challenge = BASE64URL(SHA256(verifier))`, and stores the verifier in a `SameSite=Lax; HttpOnly; Secure; path=/api/auth/callback` cookie consumed once in the callback handler.
- **Blank page after login on production**: Two root causes resolved:
  - CSP `script-src` was missing `'unsafe-inline'`, which silently blocked Trunk's inline `<script type="module">` bootstrap. Added `'unsafe-inline'` to `nginx.prod.conf`.
  - `WebAssembly.instantiateStreaming` hung indefinitely on the nginx reverse-proxy + preload-hints configuration. Added a regular (non-module) `<script>` patch to `ui/index.html` that replaces `instantiateStreaming` with an `arrayBuffer()` fallback before Trunk's module bootstrap runs.

## [1.17.0] - 2026-03-22

### Added
- **`version` and `timestamp` fields on `GET /health`**: health response now includes the running binary version and a UTC timestamp, enabling CI/CD pipelines to verify the deployed version without admin credentials.
- **30-second request timeout**: all API requests now return `408 Request Timeout` if processing exceeds 30 seconds, preventing hung connections under load.
- **`X-Request-Id` middleware**: every response carries a `X-Request-Id` UUID header (generated server-side if not provided by the client) for distributed tracing and log correlation. Header is exposed in CORS so browser clients can read it.

### Changed
- Updated `tower-http` workspace dependency to enable `timeout`, `request-id`, and `propagate-header` features.
- Stale doc comment in `AuthClaims.roles` updated to reference Cognito groups instead of Keycloak realm roles.

## [1.16.0] - 2026-03-21

### Added
- **Unit test coverage**: 27 tests total (up from 9).
  - `common::admin` — 8 tests for `AdminUser::display_name()` and `CreateAdminUserRequest` validation.
  - `common::auth` — 4 tests for `UserInfo::from(AuthClaims)`, serde round-trip, and `#[serde(default)]` on `roles`.
  - `api::wikilink` — 7 edge-case tests (whitespace trimming, empty targets, pipe with empty target, duplicates, adjacent links).

## [1.15.0] - 2026-03-21

### Added
- **Operational metrics endpoint**: `GET /api/metrics` (admin-only) returns a JSON snapshot for monitoring:
  - `version` — API binary version.
  - `uptime_secs` — process uptime since last restart.
  - `db.pool_size` / `db.pool_idle` — PostgreSQL connection pool utilisation.
  - `counts.*` — row counts for `nodes`, `edges`, `tags`, `notes`, `tasks`, `attachments`, `user_favorites`.
- `AppState` now records `started_at: Instant` for uptime tracking.

### Fixed
- Removed unused `post` import from `api/src/routes/favorites.rs`.

## [1.14.0] - 2026-03-21

### Changed
- **Admin user management migrated to Amazon Cognito**: replaced Keycloak Admin REST API client (`keycloak.rs`) with `CognitoAdminClient` (`cognito.rs`) backed by the AWS SDK.
  - All admin endpoints (`GET/POST /api/admin/users`, `DELETE /api/admin/users/{id}`, `PUT /api/admin/users/{id}/roles`, `GET /api/admin/users/roles`) now operate against the Cognito User Pool.
  - Users are identified by email; Cognito groups replace Keycloak realm roles.
  - `CreateAdminUserRequest` no longer requires a separate `username` field — email is used as the Cognito username.
  - Welcome email uses Cognito's built-in `AdminCreateUser` invite flow.
  - Dead `api/src/admin/keycloak.rs` removed.

## [1.13.0] - 2026-03-21

### Added
- **Automated backup script** (`deploy/backup.sh`): streams `pg_dump` output through gzip and uploads directly to S3-compatible object storage in a single pipeline.
  - `list` subcommand shows all stored backups.
  - `restore <file>` subcommand streams a backup from S3 back into PostgreSQL.
  - Auto-prunes oldest backups once count exceeds `BACKUP_RETAIN` (default 30).
  - Reads `deploy/.env.prod` automatically; all vars overridable via environment.
  - Supports custom `S3_ENDPOINT` for MinIO/Lightsail Object Storage.
  - Cron example: `0 2 * * * /home/ubuntu/ember-trove/deploy/backup.sh >> /var/log/ember-backup.log 2>&1`

## [1.12.0] - 2026-03-21

### Added
- **Graph type-filter**: each node type in the legend is now a clickable toggle. Clicking hides/shows all nodes of that type (dims to 40% with a "hidden" badge). Edges are automatically hidden when either endpoint type is filtered out.
- **Graph "Fit" button**: toolbar button (top-right of graph view) resets pan and zoom to the default view, bringing all nodes back into frame.

## [1.11.0] - 2026-03-21

### Added
- **Inline attachment preview**: images (any `image/*` type) and PDFs render inline inside the Attachments panel via a toggle eye-icon button.
  - Images: `<img>` with `max-h-96 object-contain` — respects aspect ratio, fits any width.
  - PDFs: `<iframe>` at 500 px height for in-page browsing.
  - Download and delete buttons remain visible for all attachment types.

### Fixed
- Clippy `collapsible_if` warnings in `favorites_section` resolved.
- "Favorites" section header in dark mode uses `stone-400` for better legibility.

## [1.10.0] - 2026-03-21

### Added
- **Sidebar Favorites**: pin any internal node or external URL to the sidebar for one-click access.
  - Favorites section sits between the search bar and "All Nodes", visible in both expanded and collapsed sidebar modes.
  - Add favorites via an in-modal picker: "Internal Node" tab (live search + select) or "External URL" tab (URL + label inputs).
  - Node favorites navigate to the node's detail view on click; URL favorites open in a new browser tab.
  - Reorder favorites with up/down arrow buttons (visible on hover).
  - Remove any favorite with the trash icon (visible on hover).
  - Favorites are user-scoped and persisted in PostgreSQL (`user_favorites` table, migration `009_favorites.sql`).
  - New API endpoints: `GET /api/favorites`, `POST /api/favorites`, `DELETE /api/favorites/{id}`, `PATCH /api/favorites/reorder`.

## [1.9.2] - 2026-03-19

### Fixed
- **Username display**: sidebar now falls back to `email` before `sub` UUID when the identity provider does not populate the `name` claim (Cognito default behaviour).
- **Cognito logout loop**: logout handler now redirects through Cognito's `end_session_endpoint` with `logout_uri`, clearing the Cognito SSO session cookie so the browser lands on the login page instead of immediately re-authenticating.
- **nginx proxy buffer**: raised `proxy_buffer_size` to 128 KB in `nginx.prod.conf` to accommodate large JWT `Set-Cookie` headers that exceeded the default 4 KB buffer and caused `502 Bad Gateway` on `/api/auth/callback`.

## [1.9.1] - 2026-03-19

### Added
- **Production AWS stack**: `deploy/docker-compose.prod.yml` — four-service compose (postgres, api, ui, nginx proxy) with `COOKIE_SECURE=true` and Cognito / Lightsail Object Storage environment variables.
- **Production nginx config**: `deploy/nginx.prod.conf` — TLS termination (Let's Encrypt), HSTS header, ACME challenge passthrough, and generous proxy buffers for JWT headers.
- **Env template**: `deploy/.env.prod.template` with documented placeholders for all production secrets.
- **AWS deployment guide**: `docs/deploy-aws.md` — step-by-step guide covering Lightsail, Route 53, Cognito, Object Storage, IAM, Certbot, and auto-renewal.

### Changed
- Replaced Keycloak with **Amazon Cognito** as the production identity provider. Local development continues to use Keycloak via `docker-compose.yml`.

## [1.9.0] - 2026-03-18

### Added
- **JWT expiry redirect**: `parse_json` helper now redirects to the login page when both the access token and refresh token are expired, instead of looping on 401.
- **Single-user mode**: node list, tag list, and notes feed return all data regardless of `owner_id`; any authenticated user can add notes to any node.
- **Mobile-responsive layout**: hamburger top bar on narrow viewports; sidebar slides in as a full-height overlay with a backdrop dismiss.

## [1.8.0] - 2026-03-18

### Added
- **Backchannel logout**: Keycloak logout now revokes the refresh token server-side via the OIDC revocation endpoint, preventing token reuse after sign-out.
- **Full-system backup**: admin-only `GET /api/admin/backup` streams the entire database as NDJSON; `POST /api/admin/restore` replays it with a preview/confirm wizard in the UI.
- **Streaming download**: backup endpoint streams response bytes directly from the database without buffering the full payload in memory.

### Fixed
- Search placeholder no longer shows stale text after clearing the search input.
- Logout correctly terminates the Keycloak SSO session via `end_session_endpoint` redirect.
- JWT `aud` claim made optional; Keycloak audience mapper configured in realm export.
- 401 reload loop: app children are lazily instantiated so a failed token refresh does not trigger an infinite reload cycle.

## [1.7.0] - 2026-03-17

### Added
- **Backup / restore UI**: admin panel with a multi-step preview/confirm wizard for full-system backup and restore.
- **Task sync**: task toggle is propagated across My Day and NodeView via a shared `TaskRefresh` context signal.

### Fixed
- Session cookies cleared with correct path on logout.
- `end_session_endpoint` rewritten with `OIDC_EXTERNAL_URL` so the browser receives a browser-reachable Keycloak URL.
- Post-logout redirect URI added to Keycloak client config.

## [1.6.0] - 2026-03-17

### Added
- **Extended search**: full-text and fuzzy search now covers notes and task text in addition to node titles and bodies.

## [1.5.0] - 2026-03-17

### Added
- Collapsible panels in NodeView.
- Dashboard sidebar item renamed for clarity.

### Fixed
- Notes feed expands to full available width.
- My Day and Dashboard empty states are vertically and horizontally centred.

## [1.4.0] - 2026-03-17

### Added
- **Notes**: per-node append-only timestamped notes with a global feed view (`/api/notes/feed`).

## [1.3.0] - 2026-03-17

### Added
- **Tasks**: per-node task lists with create / toggle / delete / My Day scheduling (`/api/nodes/{id}/tasks`).
- **My Day view**: aggregated view of all tasks scheduled for today with focus-date planning.
- **Project Dashboard**: task counts and status summary for Project-type nodes.
- **Node templates**: pre-filled Markdown templates for each node type (article, project, area, resource, reference).

## [1.2.0] - 2026-03-17

### Added
- Quick-capture FAB: floating amber button (bottom-right) opens a modal for rapid node creation with title, type, and optional notes fields; Ctrl+Enter to save, Esc to cancel; navigates to new NodeDetail on success.

### Changed
- **Ember warm theme**: replaced all cool-gray tones with Tailwind `stone` palette and blue accents with `amber`/`orange`, delivering a warm "winter fire" aesthetic consistent across both light and dark modes.
  - Light mode: `stone-50` parchment background, `stone-900` text, `amber-600` primary actions.
  - Dark mode: `stone-950` near-black background, `stone-100` text, `amber-400` links and accents.
  - Graph edges: References use `amber-600`, WikiLinks use `orange-400`.
  - Keycloak login theme updated to match warm ember palette.

## [1.1.0] - 2026-03-17

### Added
- Keycloak login theme: CSS-only dark theme matching app palette.
- Wiki-link `[[title]]` syntax: auto edge creation, UI autocomplete, click navigation, unresolved strikethrough.
- CI/CD: `.github/workflows/ci.yml` (cargo check/clippy/test + WASM job) and `.github/workflows/release.yml` (cargo-dist cross-platform binaries).
- User management UI + Keycloak admin integration.

## [1.0.0] - 2026-03-17

### Added
- Initial production release.
- All 8 implementation phases complete: workspace skeleton, OIDC auth, Node CRUD + Markdown editor, knowledge graph (edges + tags), full-text/fuzzy search, attachments + S3, per-node permissions, Docker multi-stage + K8s deployment.
