# JayJay

Native macOS GUI for Jujutsu version control. Rust core with SwiftUI and GPUI shells.

## Start Here

Keep this file as always-loaded guidance. Load a focused doc only when the task touches that area:

- [Release Workflow](agents/release.md) - version bumps, notarization, appcast, GitHub release, Homebrew tap.
- [Testing Guide](agents/testing.md) - Rust/Swift/GPUI test placement, fixtures, UI test rules.
- [Architecture Guide](agents/architecture.md) - workspace crates, dependency rules, MVVM boundaries, core module layout.
- [Storage Guide](agents/storage.md) - persisted user data, file formats and locations, shell-sharing rules, atomic writes, test isolation.
- [Version Control Guide](agents/version-control.md) - JJ history, command concurrency, workspaces, Kache, bookmarks.
- [Format Projections Guide](agents/format-projections.md) - rich diff projections for notebooks, tables, binary plists, SARIF, and raw/processed behavior.
- [Shell Feature Parity Guide](agents/shell-parity.md) - keeping SwiftUI and GPUI user-visible behavior aligned, with tracked intentional gaps.
- [SwiftUI Shell Guide](agents/swiftui.md) - shell/mac file layout, view-model and caching conventions, presentation surfaces.
- [GPUI Shell Guide](agents/gpui.md) - shell/gpui file layout, state ownership, globals, caches, rendering tips.
- [Review State Guide](agents/review-state.md) - review store, marks, notes, and the reconciliation contract.
- [Design Guide](agents/design.md) - JayJay product context, visual direction, interaction principles.
- [Help Book Guide](agents/help-book.md) - public website (`docs/`), bundled macOS Help Book, web guide reuse, Help Viewer cache, and Apple Help pitfalls.
- [Pull Request Workflow](agents/pull-requests.md) - bookmark-based GitHub PRs, review updates, landing.
- [Code Review Guide](agents/code-review.md) - repo-specific review setup, adversarial checks, severity, and reporting.

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

Isolate Cargo output per JJ workspace; never share `CARGO_TARGET_DIR` across concurrent workspace builds. Preserve the configured compiler wrapper (Kache) and use normal `cargo`/`just` commands with each workspace's default `target/`. If a sandbox cannot use the wrapper, `RUSTC_WRAPPER=""`. Do not change the developer's global Cargo or cache config. See [Version Control Guide](agents/version-control.md).

## Principles

1. **First principles** - Understand the problem before coding. Ask why before how. Do not cargo-cult from git tools; jj's model is different.
2. **KISS and DRY** - Prefer the simplest correct solution. Extract shared logic when duplication is real, not hypothetical.
3. **Single responsibility** - Each file/module/function should have one job.
4. **Cross-platform core** - Business logic belongs in Rust. UniFFI is a thin SwiftUI bridge. SwiftUI and GPUI shells render state and dispatch actions.
5. **Behavior belongs to types** - Prefer methods/extensions when behavior naturally belongs to a type. In Rust, add inherent methods when the type is in the crate; otherwise use a focused trait. In Swift, prefer extensions and computed properties over free helper functions.
6. **Comments explain the why** - Comment only non-obvious *why*, never restate the code. Keep each comment on a single line — it may run well past 80 columns; we read code in an editor, not a terminal, so don't hard-wrap it to fit.
7. **Test behavior** - New features need focused unit coverage and user-visible flow coverage when behavior reaches the UI. Do not keep tests that only mirror constants, static config, or field-by-field wiring.

## Code Organization

- Keep files under 300 lines. Split by responsibility when a file grows past that.
- One primary type per file, named after the type. Small private helpers used only by that type stay with it; deliberately-cohesive model clusters (a type plus its request/result vocabulary) may share a file.
- Group related files into responsibility folders; don't create folders for singletons.
- Rust: prefer folder modules over long single-file modules. Keep `mod.rs` and `lib.rs` thin: module declarations and `pub use` re-exports only. Put implementation in sibling modules named for the responsibility they own, such as `wrap/cols.rs`, `wrap/unified.rs`, and `wrap/side_by_side.rs`.
- Swift: split oversized types into `TypeName+Responsibility.swift` extension files instead of growing one file.

## Before Finishing

- Run the relevant tests for the behavior or files changed. Use the smallest layer that proves it: Rust unit tests, Swift unit tests (`just test-app`), XCUITest scenes, or GPUI component tests.
- Remove useless tests that only mirror constants, static config, or field wiring.
- Inline helpers that are used once.
- Remove duplication and keep naming, test placement, and module organization consistent with nearby code.
- Before a change is ready to commit, run format and lint. Defer these to the final pass so normal editing does not create unnecessary churn.

## Version Control

This repo uses **Jujutsu (jj)**, not git. Use `jj` for version-control operations (`jj describe`, `jj git push --bookmark`); do not use `git commit`, `git add`, `git push`, `git stash`, `git branch`, or `git rebase -i`.

Never run JJ-aware commands concurrently in the same workspace, including `jj st`, `jj log`, `jj diff`, and `jayjay review ...`. If divergence appears, compare each commit to `@` by commit ID and abandon only snapshots proven stale.

Work in the current workspace. Create a sibling `jj workspace` only when isolation is materially useful or the user asks; forget its metadata when that session ends.

Do not add AI attribution to commits or PRs — no `Generated with`, `Co-Authored-By`, or assistant/session trailers — unless the user explicitly asks.

Load [Version Control Guide](agents/version-control.md) before changing history, bookmarks, fetching, or pushing. Load [Pull Request Workflow](agents/pull-requests.md) for PR work.

## Local Review Notes

Before finalizing issue work, read the current working-copy notes with `jayjay review notes --repo .`, serialized with other JJ-aware commands. Load [Review State Guide](agents/review-state.md) for note statuses, add/resolve commands, and reconciliation rules.

## UI And Design

JayJay is a macOS-native developer tool for jj users. Keep UI changes native-first, keyboard-friendly, dense without clutter, and quiet (no spinner when a refresh can be silent). Use jj words: changes, bookmarks, revsets — not git branches/commits unless referring to interop.

Load [Design Guide](agents/design.md) before changing visual style, copy, interaction patterns, or user-facing workflows.
