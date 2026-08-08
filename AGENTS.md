# JayJay

Native macOS GUI for Jujutsu version control. Rust core with SwiftUI and GPUI shells.

## Start Here

Keep this file as always-loaded guidance. Load focused docs only when the task touches that area:

- [Release Workflow](agents/release.md) - version bumps, notarization, appcast, GitHub release, Homebrew tap.
- [Testing Guide](agents/testing.md) - Rust/Swift/GPUI test placement, fixtures, UI test rules.
- [Architecture Guide](agents/architecture.md) - workspace crates, dependency rules, MVVM boundaries, core module layout.
- [Storage Guide](agents/storage.md) - persisted user data, file formats and locations, shell-sharing rules, atomic writes, test isolation.
- [Version Control Guide](agents/version-control.md) - JJ history changes, command concurrency, splitting, and bookmarks.
- [Format Projections Guide](agents/format-projections.md) - rich diff projections for notebooks, tables, binary plists, SARIF, and raw/processed behavior.
- [Shell Feature Parity Guide](agents/shell-parity.md) - keeping SwiftUI and GPUI user-visible behavior aligned, with tracked intentional gaps.
- [SwiftUI Shell Guide](agents/swiftui.md) - shell/mac file layout, view-model and caching conventions, presentation surfaces.
- [GPUI Shell Guide](agents/gpui.md) - shell/gpui file layout, state ownership, globals, caches, rendering tips.
- [Review State Guide](agents/review-state.md) - review store, marks, notes, and the reconciliation contract.
- [Design Guide](agents/design.md) - JayJay product context, visual direction, interaction principles.
- [Help Book Guide](agents/help-book.md) - public website (`docs/`), bundled macOS Help Book, web guide reuse, Help Viewer cache, and Apple Help pitfalls.
- [Pull Request Workflow](agents/pull-requests.md) - bookmark-based GitHub and Codeberg PRs, review updates, landing.
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

### Rust Build Cache

Keep Cargo output isolated per JJ workspace; never point concurrent workspace builds at the same `CARGO_TARGET_DIR`.

- Preserve a configured compiler wrapper for Rust-backed `cargo` and `just` commands. Kache is preferred for concurrent workspaces because it normalizes checkout paths and restores cached artifacts into each isolated target with zero-copy filesystem clones when available.
- With Kache, keep each workspace's default `target/` and use normal Cargo commands; do not force a shared target or override incremental settings. For an sccache fallback, use `RUSTC_WRAPPER=sccache CARGO_INCREMENTAL=0` and configure the daemon's `basedirs` with every workspace root.
- Compiler caches do not replace workspace cleanup. Remove completed temporary workspace directories, including their `target/`, only when that cleanup is authorized.
- If a sandbox cannot use the configured wrapper or daemon, use `RUSTC_WRAPPER=""`. Do not change the developer's global Cargo or cache configuration as a workaround.

## Principles

1. **First principles** - Understand the problem before coding. Ask why before how. Do not cargo-cult from git tools; jj's model is different.
2. **KISS and DRY** - Prefer the simplest correct solution. Extract shared logic when duplication is real, not hypothetical.
3. **Single responsibility** - Each file/module/function should have one job.
4. **Cross-platform core** - Business logic belongs in Rust. SwiftUI and GPUI shells render and dispatch actions.
5. **Behavior belongs to types** - Prefer methods/extensions when behavior naturally belongs to a type. In Rust, add inherent methods when the type is in the crate; otherwise use a focused trait. In Swift, prefer extensions and computed properties over free helper functions.
6. **Comments explain the why** - Comment only non-obvious *why*, never restate the code. Keep each comment on a single line — it may run well past 80 columns; we read code in an editor, not a terminal, so don't hard-wrap it to fit.
7. **Test behavior** - New features need focused unit coverage and user-visible flow coverage when behavior reaches the UI. Do not keep tests that only mirror constants, static config, or field-by-field wiring.

## Code Organization

- Keep files under 300 lines. Split by responsibility when a file grows past that.
- One primary type per file, named after the type. Small private helpers used only by that type stay with it; deliberately-cohesive model clusters (a type plus its request/result vocabulary) may share a file.
- Group related files into responsibility folders; don't create folders for singletons.
- Rust: prefer folder modules over long single-file modules. Keep `mod.rs` and `lib.rs` thin: module declarations and `pub use` re-exports only. Put implementation in sibling modules named for the responsibility they own, such as `wrap/cols.rs`, `wrap/unified.rs`, and `wrap/side_by_side.rs`.
- Swift: split oversized types into `TypeName+Responsibility.swift` extension files instead of growing one file.

## Architecture Boundaries

Business logic lives in Rust core. UniFFI bridges types. SwiftUI and GPUI shells render state and dispatch actions.

Load [Architecture Guide](agents/architecture.md) before changing crate or ownership boundaries; load the [SwiftUI](agents/swiftui.md) or [GPUI](agents/gpui.md) shell guide before large file-layout or convention changes in that shell; load [Shell Feature Parity Guide](agents/shell-parity.md) before adding or changing user-visible behavior in one shell that may need parity in the other; load [Review State Guide](agents/review-state.md) before touching review marks or notes.

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

Work in the current JJ workspace by default. Create a sibling `jj workspace` only for a large or long-running session where isolation is materially useful, or when the user explicitly asks for one; do not create a workspace for routine focused work. Forget temporary workspace metadata when that session is finished.

Never run JJ-aware commands concurrently in the same workspace. Even read-only commands such as `jj st`, `jj log`, and `jj diff` may snapshot the working copy; if two commands start from the same operation, JJ can preserve both snapshots as divergent commits with the same change ID.

Serialize `jayjay review ...` and scripts or tools that open the repository through JJ as well. If divergence appears, compare each commit to `@` by commit ID and abandon only snapshots proven stale; never abandon every commit for the shared change ID.

Load [Version Control Guide](agents/version-control.md) before changing history, splitting or describing changes, managing bookmarks, fetching, or pushing. Load [Pull Request Workflow](agents/pull-requests.md) for PR work.

Do not use `git commit`, `git add`, `git push`, `git stash`, `git branch`, or `git rebase -i`; use the jj equivalents.

Do not add AI attribution to commits or PRs — no `Generated with`, `Co-Authored-By`, or assistant/session trailers — unless the user explicitly asks.

## Local Review Notes

Before finalizing issue work, read the current working-copy notes with `jayjay review notes --repo .`, serialized with other JJ-aware commands. Load [Review State Guide](agents/review-state.md) for note statuses, add/resolve commands, and reconciliation rules.

## UI And Design

JayJay is a macOS-native developer tool for jj users. Keep UI changes:

- Native-first and keyboard-friendly.
- Dense enough for repeated developer workflows without clutter.
- Fast and quiet; avoid spinners when a refresh can be silent.
- Jujutsu-native: changes/bookmarks/revsets, not git branches/commits unless referring to interop.

Load [Design Guide](agents/design.md) before changing visual style, copy, interaction patterns, or user-facing workflows.
