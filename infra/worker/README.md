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
just worker::apply-schema
wrangler secret put HASH_SECRET                  # one-time; any long random string
wrangler deploy                                  # runs workers-assets-gen + go build, then uploads
```

Local build only: `go run github.com/syumai/workers/cmd/workers-assets-gen -mode=go && GOOS=js GOARCH=wasm go build -o ./build/app.wasm .`

## Query

The Worker does not need to be redeployed to query existing stats. To use the
views in `schema.sql`, apply the schema to the remote D1 database once:

```bash
just worker::apply-schema
```

Then use the named recipes from the repository root:

```bash
just worker::daily 30
just worker::versions
just worker::platforms
just worker::systems
just worker::recent 20
```

These recipes use `wrangler d1 execute --remote`, so they read production D1
data and require Cloudflare authentication. A Worker deploy is only needed when
`main.go`, `wrangler.toml`, or the deployed route changes.

Aggregate views only include release-looking versions such as `0.3.1`; probe,
test, dev, and missing-version rows are ignored.

Raw query example:

```bash
wrangler d1 execute jayjay_stats --remote --command "
  SELECT day, channel, os, arch, version, COUNT(DISTINCT unique_key) AS dau
  FROM pings GROUP BY day, channel, os, arch, version ORDER BY day DESC LIMIT 50"
```
