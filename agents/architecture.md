# Architecture Guide

Load this file before changing crate ownership, module boundaries, or cross-shell contracts. For shell file layouts and conventions load [SwiftUI Shell Guide](swiftui.md) or [GPUI Shell Guide](gpui.md); for review marks and notes load [Review State Guide](review-state.md).

## Workspace Crates

| Crate / target | Responsibility |
| --- | --- |
| `jayjay-primitives` | jj-lib-free domain types: `Change`, `Bookmark`, `DiffHunk`, review types, hashing |
| `jj-diff` | Diff engine: Histogram line diff, word diff, tree-sitter syntax, context collapse, wrapping, conflict display, canonical change groups |
| `jayjay-review` | Local review store: marks, notes, reconciliation (uses `jj-diff`) |
| `jayjay-network` | Shared blocking HTTP client helpers |
| `jayjay-core` | jj-lib wrapper and repo operations; re-exports `jj-diff` as `jayjay_core::diff` |
| `jayjay-uniffi` | UniFFI bindings for the SwiftUI shell only; no business logic |
| `jayjay-cli` | `jayjay` binary: thin app launcher; app-owned CLI commands are forwarded to the bundled macOS executable |
| `jj-test` | Shared jj repo fixtures for integration and component tests |
| `shell/gpui` | Cross-platform GPUI shell; links the Rust crates directly (no UniFFI) |
| `shell/mac` | SwiftUI shell + the `JayJayDiffUI` Swift package (AppKit diff renderer) |

Dependency direction (never invert): `primitives` and `jj-diff` are leaves → `jayjay-review` → `jayjay-core` → `jayjay-uniffi` / `shell/gpui`.

- New shared types go in `jayjay-primitives`, not `jayjay-core`, so review/CLI code stays jj-lib-light.
- Anything two surfaces must agree on (change groups, review identity, note reconciliation) lives at or below `jayjay-review`/`jj-diff` and is consumed by all surfaces. Do not re-implement a diff or identity computation per surface; the GUI and `jayjay review notes` must reconcile through the same provider or notes silently report stale.

## MVVM

```text
Rust Core -> UniFFI -> ViewModel -> SwiftUI Views
    |                                      
    +------------- direct link ---------> GPUI shell (view_model + window)
```

- **Model** (`crates/`): all business logic. Pure Rust, no platform code.
- **Bindings** (`crates/jayjay-uniffi/`): convert types and expose core APIs; do not add business logic. Bindings regenerate during `just build`.
- **ViewModels** own the repo handle and all jj operations: `Repo/ViewModel/` in SwiftUI, `repo/view_model/` in GPUI.
- **Views**: feature folders in SwiftUI and GPUI. Views render state and call callbacks; they should not know jj internals.

Async conventions (both shells): heavy jj work runs off the UI thread (`Task.detached` → `MainActor.run` in Swift; `cx.background_spawn` → `this.update` in GPUI) and every in-flight result is guarded by a supersession check — a token, generation counter, or commit-id compare against current `@State`/VM state — so a slow stale result can never overwrite a newer one. Both shells also suppress the FS-watcher echo of their own writes (`lastInternalMutationAt` / `last_internal_mutation_at`).

## Core Modules

Keep `jayjay-core` logic split by responsibility under `repo/` (one file or folder per operation family: `log`, `diff/`, `mutations`, `bookmarks`, `git/`, `working_copy`, `resolve/`, `conflicts`, `annotate`, `evolog`, `diffedit/`, `stacked_pr/`, `pull_requests/`, `review_notes`, `undo`, `workspace`). Top-level modules (`dag`, `file_tree`, `fuzzy`, `palette`, `theme`, `commit_message`) are repo-free helpers.
