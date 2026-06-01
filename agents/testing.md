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
- Keep helpers local when they construct crate-private types for one module's tests.

## SwiftUI UI Tests

UI tests live in `shell/mac/Tests/JayJayUITests/`.

Each `SceneBase` subclass launches the app against a named fixture. The default fixture is `simple`; override `fixtureName` when a scene needs a different repo state.

Use accessibility identifiers from `shell/mac/Sources/JayJay/Shared/AccessibilityIdentifiers.swift`. Add identifiers at the view body and key them by stable data such as change-id prefix or file path.

If a scene mutates repo state, give it its own fixture. Tests share a filesystem and run alphabetically, so mutations on `simple` leak into later tests. `ui-test-setup` already creates `simple-newchange` for `NewChangeScene`; add sibling copies for new mutating scenes.

## GPUI Tests

GPUI component tests live in `shell/gpui/tests/`.

Use `#[gpui::test]` with `TestAppContext`. Each test should build its own `tempfile::TempDir` fixture through `jj_test::LinearFixture::build()` so tests are hermetic and parallel-safe. Assert state transitions and component behavior; skip pixel-layer assertions.
