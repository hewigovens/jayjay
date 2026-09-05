# Testing Guide

Load this file before adding fixtures, reorganizing tests, or changing UI test behavior.

## Running Tests

The inner-loop commands are in `AGENTS.md` (Feature Loop → Inner loop); `just test-rust <crate> <filter>` passes extra `cargo test` args after the crate. Do not start with `just test`, `just test-app`, `just lint`, or `just build`.

`just test` is `cargo test --workspace` (includes GPUI). Use it when publishing, not as the inner loop. `just test-app` and unfiltered `just test-ui` rebuild FFI, the Help Book, and the Xcode app — skip them unless Swift/app behavior changed and a package-scoped Rust test cannot prove it. `just test-gpui` after `just test` is redundant.

Choose the exact test target when a name filter could hit the wrong unit or integration binary, and inspect the result count. A green compile with zero intended tests selected, or a skipped optional fixture, is not evidence that the behavior passed. Two name filters go after `--`: `cargo test -p <crate> -- a b`. Report results per the evidence rule in `AGENTS.md` (Task Authority).

## Coverage

- Essentials and regressions only. Each behavior gets one focused test at the lowest layer that proves it; a behavior proven in Rust is not re-proven in Swift or a UI scene, and a property proven for one input is not re-proven per permutation — fold variants (line endings, EOF newline, whitespace) into one test.
- Unit tests cover core logic, view-model behavior, parsers, serialization boundaries, and regressions.
- UI tests cover user-visible workflows and accessibility identifiers: one scene per workflow.
- Avoid tests that only restate constants, static palette values, simple default field choices, or direct field-by-field wiring.
- Bug fixes include the regression test that would have caught the issue.
- Optional live Origin fixture: a sibling `jayjay-origin-smoke` checkout (standalone Cursor Origin repo, not a GitHub mirror). `crates/jayjay-core/tests/integration/pull_requests.rs` uses it when present and skips when it is not. Keep deterministic fixtures as the required gate; report live coverage separately as run, skipped, or blocked.

## Rust Test Organization

- **Inline `mod tests`**: small tests tied to private implementation details in the same source file.
- **`src/module/tests.rs` or `src/module/tests/`**: larger module unit tests, local test helpers, or split test files. Keep `mod.rs` thin.
- **`crate/tests/integration/<area>.rs`**: integration tests for public behavior across modules, jj commands, filesystem fixtures, or repo workflows, one module per area declared in `main.rs` so Cargo links one test binary instead of one per file (each links jj-lib). Add an area as a new file plus a `mod` line. Name by behavior, such as `bookmarks.rs` or `working_copy.rs`, not by fixture/helper type.
- **Shared helpers**: put reusable fixtures and command helpers in `crates/jj-test`. Do not use `#[path = "..."]` to stitch helper folders into tests.
- Helpers that implement a crate's own traits cannot live in jj-test — a helper crate linking the crate under test implements different trait types than the unit tests' `crate::` ones. Put them in the defining crate behind a `test-util` feature (see `jayjay-review/src/test_util.rs`) so other crates' tests can dev-depend on the same impls.
- Keep helpers local when they construct crate-private types for one module's tests.

## Hung and Racing jj Operations

- To test cancelling a fetch or push, do not fake the network: set the repo's `git.executable-path` to a shell script that touches a marker and sleeps, add an unreachable remote, poll the marker, then cancel. `crates/jayjay-core/tests/integration/remote_operations.rs`, `RepoViewModelSyncCancelTests`, and the `sync-cancel` fixture in `scripts/ui-test-fixtures.sh` share this pattern; dead hosts and timeouts are slow and flaky.
- `jj_test` fixtures pin commit timestamps, so two identical rewrites of `@` collapse into one commit and `is_divergent` stays false even when the code raced. Assert races through the op log — a raced write leaves two op heads, and the next load merges them into an operation with two parents — and confirm the test fails on the old code before trusting it.

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

Scenes run on the active desktop. Never `typeText`: an active CJK input method composes through it and the failure looks like an app regression. Click the field, then `paste(...)` from `SceneBase`. Query patterns that cost CI rounds: scope alert buttons to `app.sheets` (an app-wide `buttons["Clear"]` matches the Touch Bar); the Settings window's accessibility title is the selected tab name, not "Settings"; DAG rows combine their children, so the row text is in `value`, not `label`, and child identifiers disappear — find rows by value, never by child identifier or index.

## External Tool Integration

Use `scripts/test-external-tools.sh` for the real blocking process contract. It creates temporary jj repositories with syntax-highlightable Swift inputs, loads the launcher's own `jayjay config`, then runs `jj diff --tool jayjay`, `jj split --tool jayjay`, or `jj resolve --tool jayjay`. It intentionally does not build, use an Xcode test host, or call `cargo run`. Pass `--launcher /path/to/JayJay.app` to test a specific bundle and `--keep` to inspect the edited repositories afterward.

## GPUI Tests

GPUI component tests live in `shell/gpui/tests/gpui/`, one module per area declared in `main.rs`; CI runs them on Linux and Windows only, while the macOS job checks that the crate compiles, so Cargo links one test binary instead of one per file (each links gpui and jj-lib and weighs hundreds of megabytes with its dSYM). Add a new area as `tests/gpui/<area>.rs` plus a `mod` line; share `crate::harness`. Code that would exit the process (the external tool contract) must go through an injectable hook, because one exit now ends every test.

The GPUI harness installs ephemeral globals with onboarding completed; first-launch tests explicitly set `config.onboarding.completed = false`.

Use `#[gpui::test]` with `TestAppContext`. Each test should build its own `tempfile::TempDir` fixture through `jj_test::LinearFixture::build()` so tests are hermetic and parallel-safe. Assert state transitions and component behavior; skip pixel-layer assertions.

Modules behind `#[cfg(not(target_os = "macos"))]` (the in-window menu bar, the Linux tool launcher, desktop-entry discovery) never compile on macOS, so their tests only run on Linux. `just test-linux [cargo test args]` runs `cargo test --workspace` inside the OrbStack machine `gpui-test`, which mounts the repo at the same path: create it once with `orb create ubuntu:jammy gpui-test`, then install `build-essential clang pkg-config libfontconfig-dev libwayland-dev libxkbcommon-x11-dev libx11-xcb-dev git`, rustup, and the pinned `jj` (fixtures shell out to both). A cold build takes a couple of minutes; the machine may serve a just-edited file a second or two late, so re-run if a change seems missing.

The GPUI CI workflow runs the Rust tests on Windows as well as Linux. Keep fixtures NTFS- and verbatim-path-safe: no `|<>:"?*` in ref names (use `odd&name` for a revset-metachar bookmark), normalize both sides of any path-keyed map through the same canonicalize (`std::fs::canonicalize` yields `\\?\C:\…`, `dunce` yields `C:\…`), and match descriptions in revsets with `subject("…")` because `description("…")` is exact-match including the trailing newline. `DiffHunk.path` stays `/`-separated (see [Architecture](architecture.md#repository-operation-contracts)); only this job catches a separator regression.
