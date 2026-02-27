const CACHE_NAME = 'origa-v3';
const WASM_DB_NAME = 'origa-wasm-cache';
const WASM_STORE_NAME = 'wasm-modules';

function openWasmDb() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(WASM_DB_NAME, 1);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
    request.onupgradeneeded = (e) => {
      e.target.result.createObjectStore(WASM_STORE_NAME);
    };
  });
}

async function getWasmFromDb(url) {
  const db = await openWasmDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(WASM_STORE_NAME, 'readonly');
    const store = tx.objectStore(WASM_STORE_NAME);
    const request = store.get(url);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
  });
}

async function putWasmToDb(url, response) {
  const db = await openWasmDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(WASM_STORE_NAME, 'readwrite');
    const store = tx.objectStore(WASM_STORE_NAME);
    const request = store.put(response, url);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve();
  });
}

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

  if (url.pathname.endsWith('.wasm')) {
    event.respondWith(
      getWasmFromDb(url.href).then(async (cached) => {
        if (cached) {
          return cached;
        }

        const response = await fetch(event.request);
        if (response.ok) {
          const clone = response.clone();
          putWasmToDb(url.href, clone).catch(() => {});
        }
        return response;
      })
    );
    return;
  }

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
