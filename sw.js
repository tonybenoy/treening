const CACHE_NAME = 'treening-__CACHE_VERSION__';

const PRECACHE_URLS = [
    './',
    './index.html',
    './manifest.json',
    './icons/icon-192.svg',
    './icons/icon-512.svg',
];

self.addEventListener('install', event => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => cache.addAll(PRECACHE_URLS))
            .then(() => self.skipWaiting())
    );
});

self.addEventListener('activate', event => {
    event.waitUntil(
        caches.keys().then(keys =>
            Promise.all(
                keys.filter(key => key !== CACHE_NAME)
                    .map(key => caches.delete(key))
            )
        ).then(() => self.clients.claim())
    );
});

self.addEventListener('fetch', event => {
    // Skip non-GET requests and PeerJS signaling server
    if (event.request.method !== 'GET') return;
    const url = new URL(event.request.url);
    if (url.hostname.includes('peerjs.com') || url.hostname === '0.peerjs.com') return;

    if (url.origin === location.origin) {
        // App assets: cache first
        event.respondWith(
            caches.match(event.request).then(cached => {
                if (cached) return cached;
                return fetch(event.request).then(response => {
                    if (response.ok) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
                    }
                    return response;
                });
            })
        );
    } else {
        // External (Tailwind CDN etc): network first, cache fallback
        event.respondWith(
            fetch(event.request)
                .then(response => {
                    if (response.ok) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
                    }
                    return response;
                })
                .catch(() => caches.match(event.request))
        );
    }
});
