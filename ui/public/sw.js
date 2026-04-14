// Ember Trove — Service Worker (read-only offline access)
// Strategy:
//   App shell (HTML, WASM, JS, CSS, fonts, icons) → cache-first, updated on install
//   API GET requests → network-first, cached fallback for offline reads
//   API mutations (POST/PUT/PATCH/DELETE) → network-only (no offline writes)

const CACHE_NAME = "ember-trove-v2";

// App shell resources cached on install.
// Trunk hashes WASM/JS/CSS filenames, so we cache "/" (index.html) and let
// the browser follow the hashed references from there.
const SHELL_URLS = ["/", "/manifest.json", "/favicon.svg"];

// ── Install: pre-cache app shell ─────────────────────────────────────────────
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(SHELL_URLS))
  );
  // Activate immediately — don't wait for old tabs to close.
  self.skipWaiting();
});

// ── Activate: clean up old caches ────────────────────────────────────────────
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((names) =>
      Promise.all(
        names
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      )
    )
  );
  // Claim all open tabs so the SW is in control without a reload.
  self.clients.claim();
});

// ── Fetch: routing strategy ──────────────────────────────────────────────────
self.addEventListener("fetch", (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Only handle same-origin requests.
  if (url.origin !== self.location.origin) return;

  // API mutations — network only, never cache.
  if (url.pathname.startsWith("/api/") && request.method !== "GET") return;

  // API GET — network-first with cache fallback for offline reads.
  if (url.pathname.startsWith("/api/")) {
    event.respondWith(networkFirstThenCache(request));
    return;
  }

  // Static assets & app shell — cache-first, fall back to network.
  event.respondWith(cacheFirstThenNetwork(request));
});

// ── Strategies ───────────────────────────────────────────────────────────────

async function cacheFirstThenNetwork(request) {
  const cached = await caches.match(request);
  if (cached) return cached;

  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(CACHE_NAME);
      cache.put(request, response.clone());
    }
    return response;
  } catch {
    // Offline and not in cache — return the shell index for SPA navigation.
    if (request.mode === "navigate") {
      const shell = await caches.match("/");
      if (shell) return shell;
    }
    return new Response("Offline", { status: 503, statusText: "Service Unavailable" });
  }
}

async function networkFirstThenCache(request) {
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(CACHE_NAME);
      cache.put(request, response.clone());
    }
    return response;
  } catch {
    const cached = await caches.match(request);
    if (cached) return cached;
    return new Response(JSON.stringify({ error: "offline" }), {
      status: 503,
      headers: { "Content-Type": "application/json" },
    });
  }
}
