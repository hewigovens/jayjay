# Diff profiling

Profiling is off by default; neither the dependency nor its instrumentation is compiled into ordinary builds. Run from the workspace root, one scenario per command, so cold measurements start in a fresh process:

```sh
just profile diff cold
just profile diff --alloc sbs
cargo bench --locked -p jj-diff --bench profile_diff -- wrap
```

The first form reports p50/p95 timings and call counts per instrumented function, the second counts Rust allocations instead, and the third runs the workload uninstrumented for an elapsed-time comparison. Omitting the scenario runs `cold`. The fixed input is a 6,000-line Rust file with one changed line and long comments:

- `cold`: one collapsed diff, including first-use grammar initialization and empty highlight caches. A single sample cannot establish a latency distribution; repeat the command for more cold observations.
- `warm`: prepare the identical diff before starting the profiler, then compute it 100 times with cached syntax highlights. Line matching and diff construction still run.
- `expand`: prepare a collapsed diff, then reveal ten lines 100 times in the same region. This includes copying the growing result. The first reveal obtains shared cached highlights; later reveals reuse the session's retained highlights.
- `wrap`: prepare a full highlighted diff, then wrap all 6,001 rows at 80 columns 100 times.
- `sbs`: prepare the same full diff, then rebuild its 6,000 side-by-side rows and wrap them at changing widths (40 through 89 columns and back). This follows the Rust operations used by the macOS side-by-side resize handler, without Swift/UniFFI conversion or native text rendering.

Allocation counts cover Rust allocator traffic, not total process memory or native Tree-sitter allocations; capture timings and allocations in separate runs because tracking allocations affects timings. Enabling `hotpath-alloc` in another executable also requires installing the counting allocator there.

For a JSON report, prefix a command with `HOTPATH_OUTPUT_FORMAT=json HOTPATH_OUTPUT_PATH=/tmp/jj-diff-cold.json`, using a distinct path per scenario. Timings of nested functions overlap and must not be summed. Keep the hotpath version fixed when comparing report schemas.

The uninstrumented elapsed time covers workload setup/validation and result disposal, not fixture preparation, profiler startup, or report generation, so it is not comparable to one instrumented function's timing.

This measures the shared diff engine, not shell rendering or end-to-end interaction latency, and does not enable CPU sampling or CI thresholds.
