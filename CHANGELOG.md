# Changelog

All notable changes to Ember Trove are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

## [1.75.12] - 2026-04-06

### Fixed
- **Graph view: Auto-arrange now persists positions to the database** — added a batch `PUT /graph/positions` endpoint so all node positions are saved in a single transaction after auto-arrange runs.

---

## [1.75.11] - 2026-04-06

### Changed
- **Housekeeping**: removed dead `force_layout()` function, updated module doc comment, bumped `api` version to match current release, standardized `edition = "2024"` across all crates, added CHANGELOG gap note for versions 1.52.0–1.75.3.

---

## [1.75.10] - 2026-04-05

### Changed
- **UI: unified save/cancel buttons across all sections** — replaced text-label buttons with consistent icon-only buttons (`check` for save, `close` for cancel) everywhere:
  - Task panel, My Day view, Note panel, Links panel, Tag Manager, Templates view
- **UI: unified add/cancel toggles** — section header "Add" buttons now use icon-only (`add` ↔ `close`) in Task panel, Note panel, Tag Manager, and Templates view
- All icon buttons share the same visual language: `p-1.5 rounded-lg`, green hover for save, stone hover for cancel

---

## [1.75.9] - 2026-04-05

### Fixed
- **Graph view: Auto-arrange now centers the graph in the viewport** — removed the force simulation that was pushing nodes far apart. The hierarchical BFS layering alone produces a clean, non-overlapping layout instantly.
  - Nodes are now centered in the viewport after auto-arrange (not anchored to a corner)
  - Minimum zoom is 0.5x so nodes stay readable at any graph size
  - Disconnected components are tiled in a grid with proper spacing
  - Computation is now near-instant (no 300-iteration force loop)

---

## [1.75.8] - 2026-04-05

### Changed
- **Graph view: unified toolbar design** — all controls (Add Edge, Fit, Auto-arrange, zoom) are now in a single cohesive container with consistent height, dividers, and visual treatment.
- **Graph view: manual zoom input** — the zoom percentage is now an editable number field. Type any value (e.g. `100` for 100%) and press Enter to set it exactly. The field syncs bidirectionally with wheel and pinch-to-zoom gestures.

---

## [1.75.7] - 2026-04-05

### Fixed
- **Graph view: tighten auto-arrange spacing** — nodes now cluster closer together with reduced spacing constants (120→80px horizontal, 110→90px vertical), stronger edge attraction, and weaker repulsion. Layout anchored to upper-left corner instead of centered for immediate visibility.

---

## [1.75.6] - 2026-04-05

### Added
- **Graph view: Auto-arrange button** — smart layout algorithm that re-arranges all nodes to eliminate overlap (shapes + titles + tag dots) with optimal spacing for readability.
  - **Hierarchical placement** — root nodes (no incoming edges) placed in a top row, then BFS layers fan out below; hubs sorted toward the center of each layer.
  - **Multi-component support** — disconnected subgraphs are arranged in a grid, each independently laid out.
  - **Enhanced force refinement** — envelope-based repulsion prevents text overlap, same-type nodes get extra separation, component separation force keeps subgraphs apart.
  - **Auto-fit viewport** — after layout, pan and zoom automatically adjust to frame all nodes.
  - **Progress spinner** — full-screen overlay with animated spinner and message during computation.

---

## [1.75.5] - 2026-04-05

### Changed
- **Graph view: significantly expanded work area** — virtual canvas enlarged from 1000×700 to 3000×2000 (~6× more space) with proportionally scaled margins and minimap.
- **Graph view: auto-grow canvas** — force layout bounds now dynamically expand based on node count (up to 4× for 200+ nodes), so the canvas grows with your database.
- **Graph view: "Re-layout" button** — new toolbar button that re-runs the force-directed simulation to spread nodes apart when the graph gets crowded.
- **Graph view: wider zoom range** — zoom out to 0.05× (was 0.1×) and zoom in to 16× (was 8×) for finer control over large graphs.

---

## [1.75.4] - 2026-04-05

### Changed
- **CI/CD: migrate GitHub Actions to Node.js 24-compatible versions** — upgraded `actions/checkout` v4→v6, `docker/build-push-action` v6→v7, `docker/login-action` v3→v4, `docker/setup-buildx-action` v3→v4 to eliminate Node.js 20 deprecation warnings.

---

<!-- Note: versions 1.52.0–1.75.3 (24 releases) are documented in git commit history: https://github.com/jchultarsky101/ember-trove/tags -->

---

## [1.51.0] - 2026-03-29

### Added
- **Calendar view** — new sidebar entry (between My Day and Dashboard) showing a month grid of tasks that have a due date. Navigate forward/backward by month with chevron buttons or jump to the current month with "Today". Each day cell shows colour-coded chips (priority tint + text) for its tasks; done/cancelled tasks are struck through. Clicking a chip opens the node detail view. Today's cell is highlighted with an amber ring. The grid is Mon–Sun with leading blank cells for offset days.
- **`GET /api/calendar?year={y}&month={m}`** endpoint — returns `Vec<MyDayTask>` for tasks whose `due_date` falls within the given calendar month. Accessible to any authenticated user; results scoped to the caller's own tasks.

---

## [1.50.1] - 2026-03-29

### Fixed
- **Task edit form consistency** — the inline edit form in `TaskPanel` previously only allowed changing the title. It now also exposes a priority `<select>` (Low / Medium / High) and a `<input type="date">` for the due date, matching the fields available when creating a task. All three fields are saved in a single `UpdateTaskRequest`.

---

## [1.50.0] - 2026-03-29

### Fixed
- **My Day carry-over** — tasks previously disappeared from "My Day" when the date rolled over to a new day. The query now returns tasks whose `focus_date` is on or before today, unless the task is already `done` or `cancelled`. Incomplete tasks from prior days are carried forward automatically until marked done or removed from My Day. A small history-icon badge shows the original focus date for carried-over tasks.

---

## [1.49.1] - 2026-03-29

### Fixed
- **Admin `is_owner` in NodeView** — `is_owner` is now `true` when the authenticated user carries the `"admin"` role, regardless of who created the node. Previously admin users saw no "Add note", "Edit permissions", or "Pin" controls on nodes they did not own. Computed as `user.sub == n.owner_id || user.roles.contains("admin")` using the `roles: Vec<String>` field already present in `UserInfo`.

---

## [1.49.0] - 2026-03-29

### Added
- **Drag-and-drop image upload in Markdown editor** — drag one or more image files onto the editor textarea to upload them inline. The file is sent to the existing `POST /nodes/{id}/attachments` endpoint and the resulting URL is inserted as `![filename](url)` at the cursor position. A `![uploading-N…]()` placeholder is inserted immediately while the upload is in-flight and replaced (or removed on failure) once the request completes. An amber inset ring appears on the textarea during drag-over. Only `image/*` MIME types are accepted; non-image files are silently skipped.
- **Clipboard paste image upload** — `Ctrl+V` / `Cmd+V` with an image on the clipboard (e.g. a screenshot) triggers the same upload pipeline. `ev.prevent_default()` is called only when at least one image item is found in the clipboard data, so text paste is unaffected.
- A "Uploading image…" spinner badge appears in the top-right corner of the editor pane while any upload is in progress (`img_uploading: RwSignal<bool>`).

---

## [1.48.2] - 2026-03-29

### Fixed
- **Admin sees all nodes in list view** — `list_nodes` was always setting `params.subject_id = Some(claims.sub)`, which restricts results to nodes the caller owns or holds an explicit permission row for. Admin users now skip this filter (`subject_id` left as `None`), causing the SQL `IN (SELECT node_id FROM permissions …)` clause to be omitted entirely and all nodes to be returned.

---

## [1.48.1] - 2026-03-29

### Fixed
- **Admin bypasses per-node permission check** — `require_role()` in `api/src/auth/permissions.rs` now returns `Ok(())` immediately when the caller's JWT contains `"admin"` in its `roles` claim (populated from Cognito `cognito:groups`). Previously an admin user received 403 when opening any node they had not explicitly been granted a permission row for.

---

## [1.48.0] - 2026-03-27

### Added
- **Graph minimap** — small 160×112 px overview panel fixed at the bottom-right corner of the graph view. Shows all node positions as colour-coded dots (matching the node-type fill colours), faint edge lines, and an amber viewport indicator rect that reflects the current pan/zoom state. Clicking anywhere on the minimap pans the main graph to centre on that graph coordinate. The panel is hidden while the graph is loading or empty. Implemented using four new constants (`MINI_W`, `MINI_H`, `MINI_SCALE_X`, `MINI_SCALE_Y`) and a reactive `{move || {}}` block; the viewport rect updates via inner reactive closures so pan/zoom changes update only those SVG attributes without re-rendering the full minimap.

---

## [1.47.0] - 2026-03-27

### Added
- **Graph edge delete** — hovering an edge now shows a red "Delete edge" button at the bottom of the hover card. Clicking it calls `DELETE /api/edges/{id}` and removes the edge from the graph reactively without a page reload.
- **Add Edge mode in graph** — new "Add Edge" toolbar button (top-right, amber when active). Click it to enter edge-create mode (cursor → crosshair). Click a source node (amber dashed ring appears), then a target node to open a type-picker popup (edge type select + optional label). Confirm to create the edge immediately. Node dragging is disabled while in this mode; Cancel or clicking the toolbar button again exits.
- **Edge count badge on node cards** — nodes that participate in at least one edge now show a `link` icon + count badge below the date in the card's top-right corner. `Node` DTO gains `edge_count: u32`; the `list_nodes` SQL query uses a `LEFT JOIN` subquery to count edges (source OR target) per node.

---

## [1.46.0] - 2026-03-27

### Added
- **Template picker in quick-capture modal** — the FAB / `n`-shortcut modal now shows a "Template (optional)" select alongside the Type select. Choosing a template pre-fills the Notes textarea and sets the node type to match; `template_id` is passed in `CreateNodeRequest` for activity-log attribution.
- **Template picker in node editor (create mode)** — a compact "— Template —" select appears in the node editor header only when creating a new node. Selecting a template overwrites body and type. Both pickers use `LocalResource<Vec<NodeTemplate>>` mirrored into an `RwSignal` for untracked reads in `on:change` closures.

---

## [1.45.3] - 2026-03-27

### Changed
- **Node card body preview expanded to 3 lines** — CSS class changed from `truncate` (1 line) to `line-clamp-3`; `body_preview` character cap raised from 120 to 300 to ensure 3 lines of text are available at typical card widths.

---

## [1.45.2] - 2026-03-27

### Changed
- Documentation update: README, CHANGELOG, `docs/deploy-aws.md`, and `CLAUDE.md` updated with session learnings (boto3 Cognito CSS application, SVG z-order, `pointer-events`, newtype context pattern, Cognito CSS allowed-class list).

---

## [1.45.1] - 2026-03-27

### Changed
- **`n` keyboard shortcut now opens quick-capture modal** — previously `n` navigated to the full NodeEditor (`View::NodeCreate`); now it opens the same lightweight `CreateNodeModal` as the FAB, making both entry points consistent. `ShowCapture` context signal lifted to the App root so the keyboard handler and Layout share state without prop-drilling.

---

## [1.45.0] - 2026-03-27

### Added
- **Graph tag filter** — clicking a coloured tag dot on any graph node filters the graph to show only nodes that share that tag (and their connecting edges). The active dot renders larger with an amber stroke. A "Tag filter active · ×" row appears in the legend panel to clear the filter. Clicking the same dot again also clears it. Tag filter combines with the existing type-filter toggles.

---

## [1.44.1] - 2026-03-27

### Fixed
- **Graph tag dots hidden by title pill** — tag dots were rendered at `cy+27`, inside the title background pill (`cy+22` to `cy+36`), causing the pill to paint over them. Fixed by moving dots to `cy+42` (below the pill's bottom edge) and rendering the dot block after the title `<text>` element in SVG order so they always paint on top.

---

## [1.44.0] - 2026-03-27

### Added
- **Node-type icons on graph shapes** — Material Symbols Outlined ligature centred on each node shape (white, semi-transparent, `pointer-events: none`). Uses the same `type_icon()` helper as the sidebar and node lists. SVG `style=` attribute used to avoid Leptos 0.8 `attr:` prefix serialisation bug.

---

## [1.43.0] - 2026-03-27

### Added
- **Graph view tag colour overlay** — up to 5 small filled dots (r=3.5, white outline) rendered below each node shape, one per tag, using the tag's hex colour. Dots are horizontally centred and spaced 9 px apart. No backend changes required.

---

## [1.42.0] - 2026-03-27

### Added
- **Collapsible markdown preview in node editor** — the live preview pane can be toggled via a visibility icon button in the editor header. Initial visibility is determined from `window.innerWidth` (≥ 768 px → visible; mobile → hidden by default). Toggle state stored in `show_preview: RwSignal<bool>`. Amber styling on the button when preview is active.

---

## [1.41.0] - 2026-03-27

### Added
- **Saved search presets** — migration 017 adds `search_presets` table (owner-scoped). New DTOs: `SearchPresetId`, `SearchPreset`, `CreateSearchPresetRequest` in `common`. New repo: `SearchPresetRepo` / `PgSearchPresetRepo`. Routes: `GET /api/search-presets`, `POST /api/search-presets`, `DELETE /api/search-presets/{id}`. UI: "Presets ▾" dropdown in the SearchView filter bar — load a preset to restore all filters, delete with ×, or save the current search via an inline form. Total tests: 55.

---

## [1.40.0] - 2026-03-27

### Added
- **Node tagging from list view** — each node card in the list view now has a tag-picker dropdown. All tags are fetched once per list render; per-card `show_picker: RwSignal<bool>` controls visibility. Dropdown shows a colour swatch, tag name, and an amber checkmark for applied tags. Clicking attaches or detaches the tag immediately and refreshes the list. Fixes attachment drop-zone compile error by adding `DragEvent` and `DataTransfer` to web-sys features.

---

## [1.39.0] - 2026-03-27

### Added
- **Graph pinned-node highlight** — an amber hollow ring (`stroke: #f59e0b`, r=29) is drawn behind the node shape for pinned nodes, making them visually distinct in the graph view.

---

## [1.38.0] - 2026-03-27

### Added
- **`p` keyboard shortcut to toggle pin** — pressing `p` while a node detail is open toggles the node's pinned state (same as the pin button in the toolbar). `current_node_pinned: RwSignal<bool>` context is provided from the App root; `NodeView` writes it on load and keeps it in sync. Toast feedback. `ShortcutsModal` updated.

---

## [1.37.0] - 2026-03-27

### Changed
- **Attachment bulk upload** — the single-file picker is replaced by a drag-and-drop drop zone accepting multiple files simultaneously. Files are uploaded sequentially with a live `n/total` progress counter. A clear button resets the pending queue. No backend changes.

---

## [1.36.0] - 2026-03-27

### Added
- **Node pinning** — migration 016 adds `pinned BOOLEAN DEFAULT FALSE` to the `nodes` table. `PUT /api/nodes/{id}/pin` toggles pin state (owner-only). Node list sorted `pinned DESC, updated_at DESC`. Amber `push_pin` icon on pinned cards. Pin toggle button in the node-detail header.

---

## [1.35.0] - 2026-03-27

### Changed
- **Search ranking improvements** — `ts_rank_cd` now uses length normalisation (`|1`) so long documents do not unfairly dominate results. Fuzzy (ILIKE-only) body matches receive a 0.05 rank floor to distinguish them from zero-score results. The `12%` raw relevance figure in SearchView is replaced with a 3-bar visual indicator.

---

## [1.34.0] - 2026-03-27

### Fixed
- **Notes panel scrolling** — notes list now has `max-h-[28rem] overflow-y-auto` so long note histories scroll within the panel instead of expanding the page. A note-count badge is shown next to the panel header.
- **CI test stability** — `AppState` in tests now uses `..Config::default()` to avoid compilation failures when `Config` gains new fields.

---

## [1.33.0] - 2026-03-27

### Added
- **Bulk permission management** — new "Bulk Permissions" view in the admin sidebar. Groups all permission rows across all nodes; supports inline role-change and revoke; resolves Cognito usernames for display; filter input for large permission lists; owner rows are read-only.

---

## [1.32.0] - 2026-03-27

### Added
- **Node templates** — migration 015 adds `node_templates` table. CRUD routes at `/api/templates`. `TemplatesView` in sidebar with inline Markdown editor and "Use" button. `TemplatePrefill` context pre-fills `NodeEditor` when creating a node from a template. Activity action `CreatedFromTemplate` recorded on use.

---

## [1.31.0] - 2026-03-27

### Added
- **Keyboard shortcuts help modal** — pressing `?` toggles an overlay listing all global shortcuts. Escape also closes it. Rendered via Leptos `<Portal>` (`ShortcutsModal` component).

---

## [1.30.0] - 2026-03-27

### Added
- **Node version history** — migration 014 adds `node_versions` table. `NodeVersionRepo` / `PgNodeVersionRepo` snapshot the node body on every save (fire-and-forget). Routes: `GET /api/nodes/{id}/versions`, `POST /api/nodes/{id}/versions/{vid}/restore`. `VersionPanel` collapsible timeline UI in the node-detail view.

---

## [1.29.0] - 2026-03-27

### Added
- **Activity / audit log** — migration 013 adds `node_activity` table. `ActivityAction` enum with 10 variants (Created, Updated, Published, Archived, TagAttached, TagDetached, PermissionGranted, PermissionRevoked, AttachmentUploaded, AttachmentDeleted). `GET /api/nodes/{id}/activity` returns a timestamped log. `ActivityPanel` collapsible timeline UI in the node-detail view. All mutating route handlers instrumented.

---

## [1.28.0] - 2026-03-25

### Added
- **Node export** — `GET /nodes/{id}/export?format=markdown|json` returns a file download. Markdown includes YAML front-matter (title, type, status, tags, timestamps). JSON serialises the full Node DTO. A download icon in the node-view toolbar triggers the browser's native save dialog.
- **Public sharing links** — owners can generate opaque share tokens (`POST /nodes/{id}/share`). Sharing a token URL (`/share/<token>`) renders a read-only public node view with no login required. Tokens can be listed and revoked from the new "Public Links" panel in the node view. Migration 012 adds the `share_tokens` table (with optional `expires_at`).

## [1.27.0] - 2026-03-25

### Added
- **SES invite notification** — when an existing Cognito user is granted access to a node, an HTML+text email is sent via AWS SES v2 with the node title, role, and a direct link. New users continue to receive only the Cognito welcome email (no duplicate). Controlled by the optional `SES_FROM_EMAIL` env var; if unset the invite still works, the email is simply skipped. Send failures are logged as warnings and do not affect the API response.
- **Global keyboard shortcuts** — `n` new node · `g` graph · `/` search · `Esc` back to node list. Suppressed inside inputs, textareas, selects, contenteditable elements, and when Ctrl/Meta/Alt is held.

## [1.26.0] - 2026-03-25

### Added
- **GitHub CD automation** — `LIGHTSAIL_HOST`, `LIGHTSAIL_SSH_KEY` secrets and `DEPLOY_ENABLED=true` repository variable are now set. Every push of a `v*.*.*` tag triggers the existing `release.yml` workflow: creates a GitHub Release, SSH-builds the Docker images on the EC2 host, force-recreates the containers, and health-checks the API. No more manual deploy steps.

### Fixed
- **Permission panel ownership gating** — `PermissionPanel` now accepts `is_owner: bool`; the invite button, role-change dropdown, and revoke button are hidden for viewers and editors (they only see a read-only role badge).
- **`is_owner` computation** — `node_view.rs` previously treated every authenticated user as owner. It now correctly compares `auth.sub == node.owner_id`.
- **Revoke button visibility** — Replaced the unreliable `opacity-0 group-hover:opacity-100` pattern (broken in Tailwind v4) with an always-visible muted `text-stone-300 hover:text-red-500` style, consistent with the note-edit button fix in v1.24.1.

## [1.24.1] - 2026-03-24

### Fixed
- **Note edit button always visible** — Replaced `opacity-0 group-hover:opacity-100` CSS pattern (unreliable in Tailwind v4 due to `@media (hover:hover)` scoping) with an always-rendered button in muted `stone-300` that brightens to `amber-500` on hover. The pencil icon is now permanently visible on every note card.

## [1.24.0] - 2026-03-24

### Added
- **Editable notes** — Notes can now be edited after creation. Each note in the panel shows a pencil icon on hover (owner only); clicking it switches to an inline textarea with Save / Cancel controls and Ctrl+Enter shortcut. The API gains `PATCH /notes/:id` (owner-scoped); the `Note` DTO gains `updated_at`; notes display a `· edited` badge when `updated_at` differs from `created_at` by more than 2 seconds. Migration `010_notes_updated_at.sql` adds the column + trigger and back-fills existing rows from `created_at`.
- **Editable task titles** — Each task row gains an edit pencil icon in its hover-action strip. Clicking it replaces the title with an inline input; Enter saves via `PATCH /tasks/:id`, Escape cancels. All reactive closures capture only `Copy` signal types to stay `FnMut`-compatible with Leptos 0.8.

### Changed
- Notes are returned newest-first by the API (`ORDER BY created_at DESC`) — the panel now displays them in that order (most recent at the top).

## [1.23.0] - 2026-03-24

### Fixed
- **Portal modals** — `DeleteConfirmModal` and `LinkPickerModal` now use Leptos `<Portal>` (same fix as v1.22.0 for `AddFavoriteModal`). Both were rendered inside ancestor elements that could carry a CSS `transform`, trapping their `position:fixed` backdrops.

### Changed
- **Permission panel — inline role editing** — Each permission row in the "Sharing" section now shows an inline `<select>` dropdown (owner / editor / viewer) instead of a static badge. Changing the role calls `PUT /permissions/{id}` immediately, with a "saving…" state while the request is in flight. The `update_permission` API helper was added to `ui/src/api.rs`.

### Added
- **API integration tests** — `api/src/tests.rs` contains 36 router-level integration tests run via `tower::ServiceExt::oneshot` with stub repositories and a lazy pool (no live database required). Tests cover: health endpoint shape, route registration for every domain (nodes, edges, tags, search, graph, notes, favorites, permissions — standalone and per-node), auth-guard behaviour, and permission DTO serialisation. Total test count: **63** (41 API + 22 common).

## [1.22.0] - 2026-03-24

### Fixed
- **Add-Favorite dialog confined to sidebar**: The "Add to Favorites" modal was rendered inside the sidebar's `<aside>` DOM node, which carries a CSS `translate-x-*` transform for the mobile slide-in animation. Even with `md:transform-none`, the transform created a new stacking context that trapped `position:fixed` children inside the sidebar's bounding box (~230 px wide), making the dialog unusable — especially in collapsed mode. Fixed by wrapping the modal backdrop in Leptos 0.8's `<Portal>`, which teleports the DOM nodes to `<body>`, completely bypassing any ancestor stacking context.

## [1.21.2] - 2026-03-23

### Fixed
- **Health-check tooling missing from runtime image**: `debian:trixie-slim` does not include `wget`; `docker exec deploy-api-1 wget …` always exited non-zero, causing every production deploy to fail at the verification step. Added `wget` to the `apt-get install` list in the API runtime stage so the deploy health-check command works as intended.

## [1.21.1] - 2026-03-23

### Fixed
- **Health endpoint rate-limiting**: `/api/health` is now exempt from the `tower_governor` rate-limit layer. Monitoring tools and the deploy health-check (`wget` inside the API container) connect directly without nginx headers, which caused the rate-limiter key extraction to fail and return 500, making every production deploy appear unhealthy. The health route is now handled by a separate sub-router that does not pass through `GovernorLayer`.

## [1.21.0] - 2026-03-24

### Added
- **Standalone permission routes**: `GET /api/permissions[?node_id=<uuid>]` lists all grants (optionally filtered to a node); `PUT /api/permissions/{id}` updates the role on an existing grant; `DELETE /api/permissions/{id}` revokes a grant by ID directly — complementing the existing nested routes under `/api/nodes/{id}/permissions`.
- **`UpdatePermissionRequest` DTO** and **`PermissionListParams` DTO** added to the `common` crate.
- **`list_all` and `update` methods** added to `PermissionRepo` trait and `PgPermissionRepo`.
- **Rate limiting** via `tower_governor 0.8`: 10 requests/second per peer IP (burst cap 100) applied globally to all API routes. A background task prunes stale IP entries every 60 seconds.
- **Unit test suite expansion**: 16 new tests — permission repo helper round-trips, governor config validity, and DTO serde/validation in `common`.

## [1.20.2] - 2026-03-24

### Fixed
- **502 Bad Gateway on login in local Docker stack**: nginx's default 4 KB `proxy_buffer_size` was too small for the `/api/auth/callback` response, which sets large `Set-Cookie` headers containing JWT access/id/refresh tokens. Increased `proxy_buffer_size` and `proxy_buffers` to 32 KB in `deploy/nginx.conf`.

## [1.20.1] - 2026-03-24

### Fixed
- **Production deploy health check**: replaced fixed `sleep 10` with a 5 s × 12 retry loop (up to 60 s total). The API container starts quickly but OIDC discovery and database migrations take 5–15 s; the fixed sleep was not sufficient, causing false-negative deploy failures even when the deployment itself succeeded.

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
