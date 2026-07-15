# JayJay worker (Go)

A single Cloudflare Worker (standard Go → WASM via [`syumai/workers`](https://github.com/syumai/workers)) for JayJay service endpoints. Current routes:

- **`GET /ping`** — SwiftUI and GPUI anonymous statistics event. Logs app version/build + OS version + arch. Returns `200 ok`.
- **`GET /appcast.xml`** — compatibility proxy for older SwiftUI releases. Current releases fetch the signed appcast directly, independently of telemetry.

Stats land in **D1** (SQLite); schema in `schema.sql`. Standard Go, no TinyGo, no wasm-bindgen. Built wasm is ~1.7 MB gzipped — under the 3 MB free-plan limit.

## Privacy

No IP, repository, file, or command data is stored. Each client keeps a random installation secret locally and derives separate SHA-256 identifiers for the current UTC day and month. The server can count DAU and MAU but cannot link an installation across months. The secret itself never leaves the device.

Anonymous build and OS statistics are enabled by default and can be disabled in Settings. Explicit opt-outs persist. Older clients that do not send rotating identifiers fall back to salted network-period hashes and are reported separately as estimates; the raw IP never leaves the request handler.

## Build & deploy

Requires Go 1.24+ and wrangler. The Go assets/shim and wasm are produced by the `[build]` command in `wrangler.toml` (no npm app code):

```bash
wrangler d1 create jayjay_stats                  # one-time; id already in wrangler.toml
wrangler secret put HASH_SECRET                  # one-time; any long random string
just worker::deploy                              # verifies, migrates, rechecks D1, then deploys
```

Local build only: `go run github.com/syumai/workers/cmd/workers-assets-gen -mode=go && GOOS=js GOARCH=wasm go build -o ./build/app.wasm .`

## Query

The Worker does not need to be redeployed to query existing stats. Apply pending
migrations before deploying code that writes new columns:

```bash
just worker::apply-schema
```

Then use the named recipes from the repository root:

```bash
just worker::dau 30
just worker::mau 6
just worker::versions
just worker::platforms
just worker::systems
just worker::recent 20
```

These recipes use `wrangler d1 execute --remote`, so they read production D1
data and require Cloudflare authentication. A Worker deploy is only needed when
`main.go`, `wrangler.toml`, or the deployed route changes.

Aggregate views only include release-looking versions such as `0.3.1`; probe,
test, dev, and missing-version rows are ignored. DAU counts deduplicate by the
day-scoped rotating identifier. MAU and build/OS distributions use each monthly
identifier's latest event so an upgrade is counted once under its newest build.
`exact_client_installs` uses rotating client identifiers; `network_estimates`
is the compatibility count for older releases.

## Older clients and migrations

Older SwiftUI releases may continue to request `/appcast.xml`, and older GPUI
releases may call `/ping` without rotating identifiers. The worker accepts both
payloads and records them as `network_estimates`; raw IP addresses are never
stored. Existing rows remain available for DAU, but exact historical MAU cannot
be reconstructed because those rows have no month-scoped client identifier.

Schema migrations are additive and must be applied before deploying worker code
that writes the new columns. Never edit a migration after it has been applied;
add the next numbered migration and append its SHA-256 to
`migrations/checksums.sha256` instead. `just worker::deploy` verifies every
checksum, applies migrations, confirms the remote ledger matches the local
migration set, and only then deploys worker code.

Raw query example:

```bash
wrangler d1 execute jayjay_stats --remote --command "
  SELECT month, channel, platform, active_installs, exact_client_installs, network_estimates
  FROM monthly_usage ORDER BY month DESC, channel, platform"
```
