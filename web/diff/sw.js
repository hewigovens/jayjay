// Built into dist/sw.js with its version set from the build's asset names, so each deploy installs a fresh cache and drops the old one.
const cache = "jayjay-pr-__VERSION__";

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (event) =>
  event.waitUntil(
    (async () => {
      for (const name of await caches.keys())
        if (name.startsWith("jayjay-pr-") && name !== cache) await caches.delete(name);
      await self.clients.claim();
    })(),
  ),
);

self.addEventListener("fetch", (event) => {
  const { request } = event;
  const url = new URL(request.url);
  if (request.method !== "GET" || url.origin !== location.origin) return;
  if (request.mode === "navigate") event.respondWith(page(request));
  // The diff engine keeps its WASM packs in its own cache.
  else if (url.pathname.startsWith("/assets/") && !url.pathname.endsWith(".wasm"))
    event.respondWith(asset(request));
});

async function asset(request) {
  const store = await caches.open(cache);
  const hit = await store.match(request);
  if (hit) return hit;
  const response = await fetch(request);
  if (response.ok) await store.put(request, response.clone());
  return response;
}

// The page is fetched fresh so a deploy shows on the next load; offline, the last one opens.
async function page(request) {
  const store = await caches.open(cache);
  try {
    const response = await fetch(request);
    if (response.ok && response.headers.get("content-type")?.startsWith("text/html"))
      await store.put("/", response.clone());
    return response;
  } catch (error) {
    const hit = await store.match("/");
    if (hit) return hit;
    throw error;
  }
}
