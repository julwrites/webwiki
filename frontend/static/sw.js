// Bump this version string whenever static assets change.
// All clients will evict the old cache and re-fetch everything on next load.
const CACHE_NAME = 'vimwiki-v2';

// Assets pre-cached on install (app shell only — no wasm, handled at runtime).
const ASSETS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/icon.svg',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS))
  );
  // Activate immediately without waiting for existing tabs to close.
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  // Delete every cache that isn't the current version.
  event.waitUntil(
    caches.keys().then((cacheNames) =>
      Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      )
    )
  );
  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // Bypass SW entirely for API calls and non-GET requests.
  if (url.pathname.startsWith('/api/') || event.request.method !== 'GET') {
    return;
  }

  // Network-first strategy: always try the network so updates land immediately.
  // Only fall back to cache when offline.
  event.respondWith(
    fetch(event.request)
      .then((networkResponse) => {
        // Cache a fresh copy if the response is valid.
        if (networkResponse && networkResponse.status === 200 && networkResponse.type === 'basic') {
          const toCache = networkResponse.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(event.request, toCache));
        }
        return networkResponse;
      })
      .catch(() => {
        // Offline fallback: serve from cache if available.
        return caches.match(event.request);
      })
  );
});

