# Testing Guide

Load this file before adding fixtures, reorganizing tests, or changing UI test behavior.

## Running Tests

The validation policy and inner-loop commands are in [AGENTS.md](../AGENTS.md#inner-loop); `just test-rust <crate> <filter>` passes extra `cargo test` args after the crate. Select checks for the changed behavior; a Swift-only change may need Swift tests first.

`just test` is `cargo test --workspace` (includes GPUI). Use it for changes spanning crates or publication when focused tests do not cover the affected boundary. `just test-app` and unfiltered `just test-ui` rebuild FFI, the Help Book, and the Xcode app — skip them unless Swift/app behavior changed and a package-scoped Rust test cannot prove it. `just test-gpui` after `just test` is redundant.

Temporary repository fixtures can leave repo-scoped config in jj's user config directory (for example, `~/.config/jj/repos`) after the repositories are deleted. After the task's last fixture test, run `jj config gc` and confirm deletion of the orphaned entries. For unattended agent runs and workspace cleanup, use the [task-completion command](../AGENTS.md#feature-loop).

Choose the exact test target when a name filter could hit the wrong unit or integration binary, and inspect the result count. A green compile with zero intended tests selected, or a skipped optional fixture, is not evidence that the behavior passed. Two name filters go after `--`: `cargo test -p <crate> -- a b`. Report results per the evidence rule in `AGENTS.md` (Task Authority).

## Coverage

- Cover behavior at the lowest layer that proves it. Shell tests should catch independent integration or interaction failures rather than repeat Rust assertions. Combine equivalent input variants; keep distinct failure modes independently diagnosable.
- Unit tests cover core logic, view-model behavior, parsers, serialization boundaries, and regressions.
- UI tests cover user-visible workflows and accessibility identifiers: one scene per workflow.
- Appearance-only corrections follow the [Visual Change Workflow](design.md#visual-change-workflow); do not add tests that freeze font sizes, colors, spacing, or matching element bounds. Keep tests where layout affects interaction, reachability, or state.
- Behavioral bug fixes include the regression test that would have caught the issue. Confirm it fails on the pre-fix code for the intended reason and passes after the fix; a regression test that never failed proves the fixture, not the fix. One regression at the owning layer covers the bug; do not replay the scenario at every layer it crosses.
- Optional live Origin fixture: a sibling `jayjay-origin-smoke` checkout (standalone Cursor Origin repo, not a GitHub mirror). `crates/jayjay-core/tests/integration/pull_requests.rs` uses it when present and skips when it is not. Keep deterministic fixtures as the required gate; report live coverage separately as run, skipped, or blocked.

### Before adding a test

Answer four questions; a missing answer means do not add it yet.

1. Which observable behavior, invariant, or contract does it protect?
2. Which credible regression makes it fail?
3. Why does existing coverage not catch that failure? Each contract has one owning test at the lowest layer that directly proves it. A higher layer earns its own test only for a risk the owner cannot reach, such as UniFFI marshalling, an interaction, or a lifecycle failure. Extend a table-driven case or shared fixture before writing a near-duplicate.
4. Does it need a production seam (a `pub` item, flag, wrapper, or injection hook) that no production caller needs? If so, test at the real boundary instead. The sanctioned seams are the `test-util` and `mock` features below, the working-copy lock hook, and the GPUI exit hook.

A test that would break under a behavior-preserving refactor asserts implementation, not behavior; rewrite it at the owning boundary.

### Low-value patterns

Reject these when authoring. Remove an existing one only when implementation or an audit was requested; a review or investigation that finds one reports it. Keep a test that independently guards a public API, persistence format, migration, security, platform, external tool protocol, or generated cross-language contract:

- assertion-free tests that only exercise code;
- self-comparisons, or expected values produced by the helper or renderer under test;
- tests that restate constants, static palette values, default field choices, shortcut tables, or field-by-field wiring;
- the same contract asserted again with trivially different inputs, or at a second layer with no distinct risk;
- fixtures or fakes that implement the asserted behavior, so the test proves the fake;
- production code, `pub` items, or `#[cfg(test)]` seams whose only callers are tests;
- negative controls that pass for an unrelated reason, such as a rejection from a different guard;
- names or fixtures that promise more than the input exercises.

A source or asset grep is junk when it mirrors an identifier, and the cheapest independent guard when it fails on a user-facing key, byte, or path and survives a rename.

### Auditing existing tests

Discovery is read-only; report evidence before editing, and prefer a few high-confidence candidates over a large inventory. Static or slow is not a deletion reason. Before removing a test, record its name and location, the failure it can detect, the non-test callers of any seam it keeps alive, the stronger remaining proof or why none is needed, why it was added, what its removal unlocks deleting, and the focused validation command. A retained test that fails on the baseline is a possible product bug: reproduce and report it rather than deleting the test, and repair the owner only when implementation was requested. Delete the test-only seams a removed test kept alive instead of preserving aliases, and move retained regressions to their owning layer.

## Rust Test Organization

- **Inline `mod tests`**: small tests tied to private implementation details in the same source file.
- **`src/module/tests.rs` or `src/module/tests/`**: larger module unit tests, local test helpers, or split test files. Keep `mod.rs` thin.
- **`crate/tests/integration/<area>.rs`**: integration tests for public behavior across modules, jj commands, filesystem fixtures, or repo workflows, one module per area declared in `main.rs` so Cargo links one test binary instead of one per file (each links jj-lib). Add an area as a new file plus a `mod` line. Name by behavior, such as `bookmarks.rs` or `working_copy.rs`, not by fixture/helper type.
- **Shared helpers**: put reusable fixtures and command helpers in `crates/jj-test`. Do not use `#[path = "..."]` to stitch helper folders into tests.
- Helpers that implement a crate's own traits cannot live in jj-test — a helper crate linking the crate under test implements different trait types than the unit tests' `crate::` ones. Put them in the defining crate behind a `test-util` feature (see `jayjay-review/src/test_util.rs`) so other crates' tests can dev-depend on the same impls.
- Keep helpers local when they construct crate-private types for one module's tests.
- The core `mock` feature uses an in-memory forge sign-in store. Run `cargo test -p jayjay-core --features mock --lib forge_auth::` to include credential persistence regressions.

## Hung and Racing jj Operations

- To test cancelling a fetch or push, do not fake the network: set the repo's `git.executable-path` to a shell script that touches a marker and sleeps, add an unreachable remote, poll the marker, then cancel. `crates/jayjay-core/tests/integration/remote_operations.rs`, `RepoViewModelSyncCancelTests`, and the `sync-cancel` fixture in `scripts/ui-test-fixtures.sh` share this pattern; dead hosts and timeouts are slow and flaky.
- `jj_test` fixtures pin commit timestamps, so two identical rewrites of `@` collapse into one commit and `is_divergent` stays false even when the code raced. Assert races through the op log — a raced write leaves two op heads, and the next load merges them into an operation with two parents — and confirm the test fails on the old code before trusting it.

## Browser Review Tests

The reusable package lives in `packages/review-web`; `shell/web` is its browser test host. Run `npm ci` at the repository root. Rust adapter behavior is tested with `cargo test --locked -p jj-diff-web`; shared tree behavior lives in `jayjay-primitives::file_tree`. After `npm run build`, run `npm run test:browser` from `shell/web` to exercise the production bundle and real WASM in Chromium and WebKit. Keep screenshots and one-off QA scripts outside source. Native shell tests do not prove this browser boundary.

## Swift Tests

Swift unit tests live in `shell/mac/Tests/JayJayTests/` (`just test-app`). Cover Swift-only behavior; shared logic belongs in Rust tests.

Run `just test-app` and `just test-ui` in the foreground to completion, one at a time; runner recovery and CI debugging are in [Run & Debug](run-debug.md).

## SwiftUI UI Tests

UI tests live in `shell/mac/Tests/JayJayUITests/`.

Each `SceneBase` subclass launches a named fixture. The default fixture is `simple`; override `fixtureName` when a scene needs `complex`, `formats`, `review-notes`, `bookmark-diff`, `conflict`, or `evolog-hide-snapshots`.

Use `complex` when the workflow depends on scale or mixed diff shapes. It has more than 30 changed paths and 1,000 changed lines across additions, rewrites, deletions, renames, binary content, deep paths, and a path containing spaces. Keep narrow scenes on purpose-built fixtures so their assertions stay legible.

Use accessibility identifiers from `shell/mac/Sources/JayJay/Shared/AccessibilityIdentifiers.swift`. Add identifiers at the view body and key them by stable data such as change-id prefix or file path.

The sandboxed XCUITest runner cannot create repositories where the launched app can open them. Mutating scenes therefore use dedicated copies generated from a canonical fixture by `ui-test-fixtures.sh`; name those copies for the workflow, not for their source fixture. Each scene gets an isolated review store.

Pass a test id to run one scene: `just test-ui JayJayUITests/CommandPaletteScene/testOpenAndSearch`.

Each test method launches and terminates the app. Extend a related workflow when it already reaches the required state; combining files alone saves no launches. Preserve separate tests for different startup conditions or failure modes. Keep expensive accessibility diagnostics inside assertion message autoclosures so successful runs do not enumerate the UI for a discarded message.

CI uses `just test-ui-shard <index> <count>` to balance whole scenes across runners by test-method count, since every test pays an app launch. It reads each scene's class name from its file name, so keep one scene class per file in `Scenes/`, named after the file.

Scenes run on the active desktop. Never `typeText`: an active CJK input method composes through it and the failure looks like an app regression. Click the field, then `paste(...)` from `SceneBase`. Query patterns that cost CI rounds: scope alert buttons to `app.sheets` (an app-wide `buttons["Clear"]` matches the Touch Bar); find the Settings window by `AID.Settings.sidebar` and navigate with `selectSettingsPage`, since its title follows the selected page; DAG rows combine their children, so the row text is in `value`, not `label`, and child identifiers disappear — find rows by value, never by child identifier or index.

## External Tool Integration

Use `scripts/test-external-tools.sh` for the real blocking process contract. It creates temporary jj repositories with syntax-highlightable Swift inputs, loads the launcher's own `jayjay config`, then runs `jj diff --tool jayjay`, `jj split --tool jayjay`, or `jj resolve --tool jayjay`. It intentionally does not build, use an Xcode test host, or call `cargo run`. Pass `--launcher /path/to/JayJay.app` to test a specific bundle and `--keep` to inspect the edited repositories afterward.

## GPUI Tests

GPUI component tests live in `shell/gpui/tests/gpui/`, one module per area declared in `main.rs`; CI runs them on Linux and Windows only, while the macOS job checks that the crate compiles, so Cargo links one test binary instead of one per file (each links gpui and jj-lib and weighs hundreds of megabytes with its dSYM). Add a new area as `tests/gpui/<area>.rs` plus a `mod` line; share `crate::harness`. Code that would exit the process (the external tool contract) must go through an injectable hook, because one exit now ends every test.

The GPUI harness installs ephemeral globals with onboarding completed; first-launch tests explicitly set `config.onboarding.completed = false`.

Use `#[gpui::test]` with `TestAppContext`. Each test should build its own `tempfile::TempDir` fixture through `jj_test::LinearFixture::build()` so tests are hermetic and parallel-safe. Assert state transitions and component behavior; skip pixel-layer assertions.

Modules behind `#[cfg(not(target_os = "macos"))]` (the in-window menu bar, the Linux tool launcher, desktop-entry discovery) never compile on macOS, so their tests only run on Linux. On Apple Silicon, install Apple's `container` CLI (`brew install container`) and start its service (`container system start`), then run `just test-linux [cargo test args]`. With no args it runs `cargo test --locked --workspace`; a focused example is `just test-linux -p jayjay-gpui --test gpui settings`.

`scripts/test-linux.sh` builds a native ARM64 Debian trixie image on first use with the Linux libraries, Git, the manifest's minimum Rust version, and a `jj` CLI matching `jj-lib`. The image is rebuilt when its recipe or those versions change. Only the current workspace is mounted, read-only; host Cargo settings and credentials are not mounted. Each invocation copies its current files into a disposable guest source tree, excluding `.git`, `.jj`, and host build output (`target/`, `build/`, SwiftPM `.build/`). This includes uncommitted edits and prevents host VCS metadata from sending Git subprocesses to paths outside the mount. Cargo downloads and build artifacts live in a 16 GiB Linux volume keyed by the workspace's absolute path, so sibling workspaces have separate targets and macOS artifacts stay untouched. Debug info and incremental compilation are disabled to limit disk usage. Each invocation creates and removes its own run container; the image and cache volume persist. The script prints their names; remove that workspace's cache with `container volume delete <printed-volume-name>` when it is no longer needed. Do not use a global prune to clean one workspace.

The test container defaults to 6 CPUs and 10 GiB RAM; override `JAYJAY_LINUX_CPUS` and `JAYJAY_LINUX_MEMORY` for the host. A full `-p jayjay-gpui` run links several large test binaries at once and can get the linker killed (`ld terminated with signal 9`) even with more memory; add `-j 2`. The image builder is managed by Apple's CLI; do not stop or reconfigure it during another project's build. These are headless component tests, not a graphical Linux app session.

Container leftovers are per workspace. `container volume list` shows `jayjay-linux-test-<key>` (this script) and `jayjay-ssh-helper-<key>` (`just ssh-helper` for a Linux target), where the key is the first 16 hex digits of the SHA-256 of the workspace's absolute path (with the platform, e.g. `linux/arm64`, appended for the helper). When a sibling workspace is removed, delete its volumes with `container volume delete`; the `jayjay-linux-test:<hash>` and `rust:<version>-slim-trixie` images are shared across workspaces, so keep them unless a version bump left an old tag behind.

The GPUI CI workflow runs the Rust tests on Windows as well as Linux. Keep fixtures NTFS- and verbatim-path-safe: no `|<>:"?*` in ref names (use `odd&name` for a revset-metachar bookmark), normalize both sides of any path-keyed map through the same canonicalize (`std::fs::canonicalize` yields `\\?\C:\…`, `dunce` yields `C:\…`), and match descriptions in revsets with `subject("…")` because `description("…")` is exact-match including the trailing newline. `DiffHunk.path` stays `/`-separated (see [Architecture](architecture.md#repository-operation-contracts)); only this job catches a separator regression.
