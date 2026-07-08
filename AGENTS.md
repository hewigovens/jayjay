# JayJay

Native macOS GUI for Jujutsu version control. Rust core with SwiftUI and GPUI shells.

## Start Here

Keep this file as always-loaded guidance. Load focused docs only when the task touches that area:

- [Release Workflow](agents/release.md) - version bumps, notarization, appcast, GitHub release, Homebrew tap.
- [Testing Guide](agents/testing.md) - Rust/Swift/GPUI test placement, fixtures, UI test rules.
- [Architecture Guide](agents/architecture.md) - workspace crates, dependency rules, MVVM boundaries, core module layout.
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
jj split FILE -m "msg"   # filesets are positional, not --paths
jj edit <rev>
jj bookmark set <name> -r <rev>
jj git fetch
jj git push --bookmark <name>
jj fix
```

For PR work, load [Pull Request Workflow](agents/pull-requests.md). Use a pushed bookmark and JayJay's **Pull Request on GitHub** or **Pull Request on Codeberg** action.

Do not use `git commit`, `git add`, `git push`, `git stash`, `git branch`, or `git rebase -i`; use the jj equivalents.

Do not add AI attribution to commits or PRs — no `Generated with`, `Co-Authored-By`, or assistant/session trailers — unless the user explicitly asks.

## Local Review Notes

JayJay can store local review notes on the current working-copy change. Agents should read them before finalizing issue work:

```bash
jayjay review notes --repo .                  # plain text: full bodies + anchor lines, agent-consumable as-is
jayjay review notes --repo . --format json    # structured output for pipelines
jayjay review resolve-note <id> --repo .
jayjay review add-note --repo . --file <path> --line <n> [--side new|old] -m "note body"
```

Treat `current` notes as actionable, `stale` notes as needing re-check against the changed diff, and `orphaned` notes as comments whose original file/anchor disappeared. Resolve notes only after the underlying feedback is addressed.

Agents may also leave notes with `add-note` — anchored to a changed line of the working-copy diff, they render inline in the JayJay diff view for the user. Prefer notes over source-code comments for review commentary (intent, risks, questions): notes live in the review layer and never ship in the change. The line must be a changed (added/removed) line; adding to the same line updates that line's active note.

## UI And Design

JayJay is a macOS-native developer tool for jj users. Keep UI changes:

- Native-first and keyboard-friendly.
- Dense enough for repeated developer workflows without clutter.
- Fast and quiet; avoid spinners when a refresh can be silent.
- Jujutsu-native: changes/bookmarks/revsets, not git branches/commits unless referring to interop.

Load [Design Guide](agents/design.md) before changing visual style, copy, interaction patterns, or user-facing workflows.
