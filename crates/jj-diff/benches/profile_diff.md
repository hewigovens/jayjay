# Diff profiling pilot

Run from the workspace root. Profiling is off by default; neither the dependency nor its instrumentation is compiled into ordinary builds. The benchmark also runs without profiling for a workload elapsed-time comparison.

```sh
export HOTPATH_METRICS_SERVER_OFF=1
cargo bench --locked -p jj-diff --bench profile_diff --features hotpath -- cold
cargo bench --locked -p jj-diff --bench profile_diff --features hotpath -- warm
cargo bench --locked -p jj-diff --bench profile_diff --features hotpath -- expand
cargo bench --locked -p jj-diff --bench profile_diff --features hotpath -- wrap
cargo bench --locked -p jj-diff --bench profile_diff --features hotpath -- sbs
```

The target uses a custom harness on stable Rust. Omitting the scenario runs `cold`; pass one scenario per invocation so cold measurements start in a fresh process.

The environment setting disables hotpath's local metrics server; these runs only write reports. The fixed input is a 6,000-line Rust file with one changed line and long comments. Each command starts a separate process:

- `cold`: one collapsed diff, including first-use grammar initialization and empty highlight caches. A single sample cannot establish a latency distribution; repeat the command for more cold observations.
- `warm`: prepare the identical diff before starting the profiler, then compute it 100 times with cached syntax highlights. Line matching and diff construction still run.
- `expand`: prepare a collapsed diff, then reveal ten lines 100 times in the same region. This includes copying the growing result. The first reveal obtains shared cached highlights; later reveals reuse the session's retained highlights.
- `wrap`: prepare a full highlighted diff, then wrap all 6,001 rows at 80 columns 100 times.
- `sbs`: prepare the same full diff, then rebuild its 6,000 side-by-side rows and wrap them at changing widths (40 through 89 columns and back). This follows the Rust operations used by the macOS side-by-side resize handler, without Swift/UniFFI conversion or native text rendering.

For allocation counts, replace `--features hotpath` with `--features hotpath-alloc`. The benchmark installs the counting allocator; enabling this feature in a different executable also requires installing that allocator there. These counts cover Rust allocator traffic, not total process memory or native Tree-sitter allocations. Run timing and allocation captures separately because tracking allocations affects timings.

For a local JSON report, prefix a command with `HOTPATH_OUTPUT_FORMAT=json HOTPATH_OUTPUT_PATH=/tmp/jj-diff-cold.json`, using a distinct output path per scenario. Reports contain p50/p95 and call counts. Timings of nested functions overlap and must not be summed. Keep the hotpath version fixed when comparing report schemas.

Omit `--features hotpath` to run the same workload without instrumentation. The stderr elapsed time includes workload setup/validation inside the scenario and result disposal; it excludes fixture preparation, profiler startup, and report generation. It is not directly comparable to one instrumented function's timing.

This pilot does not enable CPU sampling, app-session profiling, or CI thresholds. It measures the shared diff engine, not shell rendering or end-to-end interaction latency.
