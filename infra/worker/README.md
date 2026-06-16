# JayJay worker (Go)

A single Cloudflare Worker (standard Go → WASM via [`syumai/workers`](https://github.com/syumai/workers)) for JayJay service endpoints. Current routes:

- **`GET /appcast.xml`** — macOS/Sparkle opt-in proxy. Logs aggregate request stats, then proxies the real appcast from `APPCAST_ORIGIN`. The appcast's EdDSA signature is verified in-app, so the proxy can't tamper with updates.
- **`GET /ping`** — GPUI (Linux/Windows) opt-in daily ping. Logs app version + OS + arch. Returns `200 ok`.

Stats land in **D1** (SQLite); schema in `schema.sql`. Standard Go, no TinyGo, no wasm-bindgen. Built wasm is ~1.7 MB gzipped — under the 3 MB free-plan limit.

## Privacy

No IP or personal data is stored. The daily-unique counter is a salted SHA-256 of `(IP + day + HASH_SECRET)`, stored only as `unique_key`; the raw IP never leaves the request handler. Telemetry is opt-in (GPUI: `telemetry.enabled = true`; macOS: the "Send anonymous usage stats" Settings toggle, which routes update checks through the appcast proxy).

## Build & deploy

Requires Go 1.24+ and wrangler. The Go assets/shim and wasm are produced by the `[build]` command in `wrangler.toml` (no npm app code):

```bash
wrangler d1 create jayjay_stats                  # one-time; id already in wrangler.toml
wrangler d1 execute jayjay_stats --remote --file=schema.sql
wrangler secret put HASH_SECRET                  # one-time; any long random string
wrangler deploy                                  # runs workers-assets-gen + go build, then uploads
```

Local build only: `go run github.com/syumai/workers/cmd/workers-assets-gen -mode=go && GOOS=js GOARCH=wasm go build -o ./build/app.wasm .`

## Query

```bash
wrangler d1 execute jayjay_stats --remote --command "
  SELECT day, channel, os, arch, version, COUNT(DISTINCT unique_key) AS dau
  FROM pings GROUP BY day, channel, os, arch, version ORDER BY day DESC LIMIT 50"
```
