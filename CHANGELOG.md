# Changelog

All notable changes to Ember Trove are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

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
