# Testing Guide

Load this file before adding fixtures, reorganizing tests, or changing UI test behavior.

## Coverage

- Unit tests should cover core logic, view-model behavior, parsers, serialization boundaries, and regressions.
- UI tests should cover user-visible workflows and accessibility identifiers.
- Avoid tests that only restate constants, static palette values, simple default field choices, or direct field-by-field wiring.
- Bug fixes should include the regression test that would have caught the issue.

## Rust Test Organization

- **Inline `mod tests`**: small tests tied to private implementation details in the same source file.
- **`src/module/tests.rs` or `src/module/tests/`**: larger module unit tests, local test helpers, or split test files. Keep `mod.rs` thin.
- **`crate/tests/*.rs`**: integration tests for public behavior across modules, jj commands, filesystem fixtures, or repo workflows. Name by behavior, such as `bookmarks.rs` or `working_copy.rs`, not by fixture/helper type.
- **Shared helpers**: put reusable fixtures and command helpers in `crates/jj-test`. Do not use `#[path = "..."]` to stitch helper folders into tests.
- Helpers that implement a crate's own traits cannot live in jj-test — a helper crate linking the crate under test implements different trait types than the unit tests' `crate::` ones. Put them in the defining crate behind a `test-util` feature (see `jayjay-review/src/test_util.rs`) so other crates' tests can dev-depend on the same impls.
- Keep helpers local when they construct crate-private types for one module's tests.

## SwiftUI UI Tests

UI tests live in `shell/mac/Tests/JayJayUITests/`.

Each `SceneBase` subclass launches a named fixture. The default fixture is `simple`; override `fixtureName` when a scene needs `complex`, `formats`, `review-notes`, `bookmark-diff`, or `conflict`.

Use `complex` when the workflow depends on scale or mixed diff shapes. It has more than 30 changed paths and 1,000 changed lines across additions, rewrites, deletions, renames, binary content, deep paths, and a path containing spaces. Keep narrow scenes on purpose-built fixtures so their assertions stay legible.

Use accessibility identifiers from `shell/mac/Sources/JayJay/Shared/AccessibilityIdentifiers.swift`. Add identifiers at the view body and key them by stable data such as change-id prefix or file path.

The sandboxed XCUITest runner cannot create repositories where the launched app can open them. Mutating scenes therefore use dedicated copies generated from a canonical fixture by `ui-test-fixtures.sh`; name those copies for the workflow, not for their source fixture. Each scene gets an isolated review store.

## External Tool Integration

Use `scripts/test-external-tools.sh` for the real blocking process contract. It creates temporary jj repositories with syntax-highlightable Swift inputs, loads the launcher's own `jayjay config`, then runs `jj diff --tool jayjay`, `jj split --tool jayjay`, or `jj resolve --tool jayjay`. It intentionally does not build, use an Xcode test host, or call `cargo run`. Pass `--launcher /path/to/JayJay.app` to test a specific bundle and `--keep` to inspect the edited repositories afterward.

## GPUI Tests

GPUI component tests live in `shell/gpui/tests/`.

Use `#[gpui::test]` with `TestAppContext`. Each test should build its own `tempfile::TempDir` fixture through `jj_test::LinearFixture::build()` so tests are hermetic and parallel-safe. Assert state transitions and component behavior; skip pixel-layer assertions.
