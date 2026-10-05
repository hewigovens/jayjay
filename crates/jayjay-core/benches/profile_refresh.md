# Refresh profiling

Replays the core calls behind a macOS repository refresh against an existing repository: the default-revset graph and its layout, bookmarks and revset vocabulary, the selected change, and the status-bar context. The working-copy snapshot is left out because it writes.

```sh
just profile refresh <repo-path> [refresh|graph|bookmarks]
just profile refresh --alloc <repo-path> graph
cargo bench --locked -p jayjay-core --bench profile_refresh -- <repo-path>
```

`refresh` (default) runs every step; `graph` and `bookmarks` run one group each. Each of five iterations reopens the repository, so every pass is a first load with cold caches. The `just` forms add per-function timings (or allocation counts) for core and jj-diff; the plain `cargo bench` form prints only per-step times.

Opening a repository with concurrent operation heads merges them into a new operation, so the benchmark refuses such a repository; profile an isolated copy instead. Otherwise it creates no operations, though loading can extend jj's index cache as any jj command does.

Caches that outlive an operation, such as commit emptiness, only pay off on the refreshes that follow a snapshot. To measure that, profile a disposable copy (`cp -c -R` clones a repository on APFS) and snapshot between refreshes; never point a writing experiment at a real checkout.
