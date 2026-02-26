const CACHE_NAME = 'origa-v1';

self.addEventListener('install', (event) => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      );
    })
  );
  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;

  const url = new URL(event.request.url);

  if (url.origin !== location.origin) return;

  event.respondWith(
    caches.match(event.request).then((cachedResponse) => {
      if (cachedResponse) {
        return cachedResponse;
      }

      return fetch(event.request).then((response) => {
        if (!response || response.status !== 200) {
          return response;
        }

        const shouldCache =
          url.pathname === '/' ||
          url.pathname === '/index.html' ||
          url.pathname.endsWith('.css') ||
          url.pathname.endsWith('.js') ||
          url.pathname.endsWith('.wasm') ||
          url.pathname.startsWith('/public/');

        if (shouldCache) {
          const responseToCache = response.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, responseToCache);
          });
        }

        return response;
      });
    }).catch(() => {
      if (url.pathname === '/' || url.pathname === '/index.html') {
        return caches.match('/index.html');
      }
    })
  );
});
