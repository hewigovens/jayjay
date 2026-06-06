# JayJay

Native macOS GUI for Jujutsu version control. Rust core with SwiftUI and GPUI shells.

## Start Here

Keep this file as always-loaded guidance. Load focused docs only when the task touches that area:

- [Release Workflow](agents/release.md) - version bumps, notarization, appcast, GitHub release, Homebrew tap.
- [Testing Guide](agents/testing.md) - Rust/Swift/GPUI test placement, fixtures, UI test rules.
- [Architecture Guide](agents/architecture.md) - MVVM boundaries, file layout, review state, presentation surfaces.
- [Design Guide](agents/design.md) - JayJay product context, visual direction, interaction principles.
- [Pull Request Workflow](agents/pull-requests.md) - bookmark-based GitHub and Codeberg PRs, review updates, landing.

When a change spans multiple areas, load each relevant doc before editing.

## Build

```bash
just build      # Build debug app
just run        # Build and launch
just lint       # Clippy + SwiftLint
just test       # Rust unit tests across the workspace
just test-app   # Swift unit tests
just test-ui    # XCUITest scenes
just test-gpui  # GPUI component tests
just release    # Sign, notarize, package; read agents/release.md first
```

## Principles

1. **First principles** - Understand the problem before coding. Ask why before how. Do not cargo-cult from git tools; jj's model is different.
2. **KISS and DRY** - Prefer the simplest correct solution. Extract shared logic when duplication is real, not hypothetical.
3. **Single responsibility** - Each file/module/function should have one job.
4. **Cross-platform core** - Business logic belongs in Rust. SwiftUI and GPUI shells render and dispatch actions.
5. **Behavior belongs to types** - Prefer methods/extensions when behavior naturally belongs to a type. In Rust, add inherent methods when the type is in the crate; otherwise use a focused trait. In Swift, prefer extensions and computed properties over free helper functions.
6. **Terse comments** - Comment only non-obvious why. Keep comments short; do not restate code.
7. **Test behavior** - New features need focused unit coverage and user-visible flow coverage when behavior reaches the UI. Do not keep tests that only mirror constants, static config, or field-by-field wiring.

## Code Organization

- Keep files under 300 lines. Split by responsibility when a file grows past that.
- Prefer folder modules over long single-file modules.
- Keep `mod.rs` and `lib.rs` thin: module declarations and `pub use` re-exports only.
- Put implementation in sibling modules named for the responsibility they own, such as `wrap/cols.rs`, `wrap/unified.rs`, and `wrap/side_by_side.rs`.

## Architecture Boundaries

Business logic lives in Rust core. UniFFI bridges types. SwiftUI and GPUI shells render state and dispatch actions.

Load [Architecture Guide](agents/architecture.md) before changing ownership boundaries, review state, presentation surfaces, or large file layout.

## Testing

Use the smallest test layer that proves the behavior:

- Rust unit tests for core logic and view-model behavior.
- Swift unit tests for Swift-only behavior.
- XCUITest scenes for SwiftUI user-visible workflows.
- GPUI component tests for GPUI shell state transitions.

Load [Testing Guide](agents/testing.md) before adding fixtures, reorganizing Rust tests, or changing UI test behavior.

## Before Finishing

- Run the relevant tests for the behavior or files changed.
- Remove useless tests that only mirror constants, static config, or field wiring.
- Inline helpers that are used once.
- Remove duplication and keep naming, test placement, and module organization consistent with nearby code.
- Before a change is ready to commit, run format and lint. Defer these to the final pass so normal editing does not create unnecessary churn.

## Version Control

This repo uses **Jujutsu (jj)**, not git. All version-control operations should use `jj`.

Key differences:

- No staging area; jj auto-snapshots the working copy.
- Changes are identified by change IDs, not git commit hashes.
- `@` is the working copy and `@-` is its parent.
- History is mutable.

Common commands:

```bash
jj st
jj log --limit 10
jj diff
jj describe -m "message"
jj commit -m "message"
jj squash
jj split --paths FILE -m "msg"
jj edit <rev>
jj bookmark set <name> -r <rev>
jj git fetch
jj git push --bookmark <name>
jj fix
```

For PR work, load [Pull Request Workflow](agents/pull-requests.md). Use a pushed bookmark and JayJay's **Pull Request on GitHub** or **Pull Request on Codeberg** action.

Do not use `git commit`, `git add`, `git push`, `git stash`, `git branch`, or `git rebase -i`; use the jj equivalents.

## UI And Design

JayJay is a macOS-native developer tool for jj users. Keep UI changes:

- Native-first and keyboard-friendly.
- Dense enough for repeated developer workflows without clutter.
- Fast and quiet; avoid spinners when a refresh can be silent.
- Jujutsu-native: changes/bookmarks/revsets, not git branches/commits unless referring to interop.

Load [Design Guide](agents/design.md) before changing visual style, copy, interaction patterns, or user-facing workflows.
