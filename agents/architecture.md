# Architecture Guide

Load this file before changing crate ownership, module boundaries, or cross-shell contracts. For shell file layouts and conventions load [SwiftUI Shell Guide](swiftui.md) or [GPUI Shell Guide](gpui.md); for review marks and notes load [Review State Guide](review-state.md).

## Workspace Crates

| Crate / target | Responsibility |
| --- | --- |
| `jayjay-primitives` | jj-lib-free domain types: `Change`, `Bookmark`, `DiffHunk`, review types, hashing |
| `jayjay-markdown` | Shared Markdown parser/event stream plus safe HTML renderer for rich previews |
| `jj-diff` | Diff engine: Histogram line diff, word diff, tree-sitter syntax, context collapse, wrapping, conflict display, canonical change groups, review group fingerprints |
| `jayjay-review` | Review marks, hunk baselines, notes, and reconciliation with optional native filesystem persistence (uses `jj-diff`) |
| `jayjay-network` | Platform-neutral HTTP request/response policy with an optional blocking adapter |
| `jayjay-core` | Portable analysis helpers plus native jj-lib/repository operations and the shared app-owned CLI dispatcher; re-exports `jj-diff` as `jayjay_core::diff` |
| `jayjay-uniffi` | UniFFI bindings for the SwiftUI shell and portable/WASM consumers; no business logic |
| `jayjay-cli` | `jayjay` binary: thin app launcher; app-owned CLI commands are forwarded to the bundled macOS executable, falling back to the GPUI binary (the only option on Linux/Windows) |
| `jj-test` | Shared jj repo fixtures for integration and component tests |
| `shell/gpui` | Cross-platform GPUI shell; links the Rust crates directly (no UniFFI); writes shared headless CLI outcomes before any window init |
| `shell/mac` | SwiftUI shell + the `JayJayDiffUI` Swift package (AppKit diff renderer) |

Dependency direction (never invert): `primitives` is the leaf → `jj-diff` (may use `primitives`) → `jayjay-review` → `jayjay-core` → `jayjay-uniffi` / `shell/gpui`.

- New shared types go in `jayjay-primitives`, not `jayjay-core`, so review/CLI code stays jj-lib-light.
- Anything two surfaces must agree on (change groups, review identity, group fingerprints, note reconciliation) lives at or below `jayjay-review`/`jj-diff` and is consumed by all surfaces. Do not re-implement a diff, fingerprint, or identity computation per surface; the GUI and `jayjay review notes` must reconcile through the same provider or notes silently report stale.
- App-owned headless commands parse, execute, and format their outcomes once in `jayjay-core`; GPUI calls that dispatcher directly and SwiftUI calls it through one UniFFI entry point. Shell code only writes stdout/stderr and exits.

## Build Profiles

Diffing, Tree-sitter syntax, DAG layout, file trees, themes, projections, and in-memory review state are always portable. `jayjay-core`'s `repository` feature adds jj-lib, Gix, the blocking network adapter, and repository operations; `watchman` adds filesystem monitoring. `jayjay-review`'s `storage` feature adds native filesystem persistence.

`jayjay-uniffi` defaults to its `desktop` profile, which enables repository operations, Watchman, review storage, and the binding generator CLI. A WASM consumer builds with `--no-default-features --features wasm`, supplies persistence and repository access, and implements the async `HttpTransport` foreign trait. Verify that graph and its LLVM-linked Tree-sitter grammars with `just test-wasm`.

## MVVM

```text
Rust Core -> UniFFI -> ViewModel -> SwiftUI Views
    |                                      
    +------------- direct link ---------> GPUI shell (view_model + window)
```

- **Model** (`crates/`): all business logic. Pure Rust, no platform code.
- **Bindings** (`crates/jayjay-uniffi/`): convert types and expose core APIs; do not add business logic. Bindings regenerate during `just ffi` (and as part of `just build`).
- **ViewModels** own the repo handle and all jj operations: `Repo/ViewModel/` in SwiftUI, `repo/view_model/` in GPUI.
- **Views**: feature folders in SwiftUI and GPUI. Views render state and call callbacks; they should not know jj internals.

Async conventions (both shells): heavy jj work runs off the UI thread (`Task.detached` → `MainActor.run` in Swift; `cx.background_spawn` → `this.update` in GPUI) and every in-flight result is guarded by a supersession check — a token, generation counter, or commit-id compare against current `@State`/VM state — so a slow stale result can never overwrite a newer one. Guard the result application; do not preemptively discard still-valid presentation state while its replacement loads. If an old-state action would be unsafe, disable that action explicitly instead of replacing the whole pane with an unrelated empty state. Both shells also suppress the FS-watcher echo of their own writes (`lastInternalMutationAt` / `last_internal_mutation_at`).

## Core Modules

Keep `jayjay-core` logic split by responsibility under `repo/` (one file or folder per operation family: `log`, `diff/`, `mutations`, `bookmarks`, `git/`, `working_copy`, `resolve/`, `conflicts`, `annotate`, `evolog`, `diffedit/`, `stacked_pr/`, `pull_requests`, `review_notes`, `undo`, `workspace`). Top-level portable modules (`dag`, `file_tree`, `fuzzy`, `palette`, `theme`, `commit_message`) are repo-free helpers. The native-only `repositories` module owns the file-backed pin contract shared by both desktop shells, and the native-only `cli` module owns the shared app CLI dispatcher (`--version`, `config`, `review …`) that both shells adapt.
