# Architecture Guide

Load this file before changing module ownership, repo state, review state, presentation surfaces, or large file layout.

## MVVM

```text
Rust Core -> UniFFI -> ViewModel -> SwiftUI/GPUI Views
    ^             |          |              |
    |             v          v              v
 business     type bridge  actions       rendering
```

- **Model** (`crates/jayjay-core/`): jj-lib wrapper, diff engine, tree-sitter syntax, review state, repo operations. Pure Rust, no platform code.
- **Bindings** (`crates/jayjay-uniffi/`): thin UniFFI layer. Convert types and expose core APIs; do not add business logic.
- **SwiftUI ViewModel** (`Repo/RepoViewModel.swift` and `RepoViewModel+Actions.swift`): owns `JayJayRepo`. jj operations go through here. Async work uses `Task.detached` then `MainActor.run`.
- **Views**: feature folders in SwiftUI and GPUI. Views render state and call callbacks; they should not know jj internals.

## Core Modules

Keep core logic split by responsibility:

- `repo/mod.rs`
- `repo/log.rs`
- `repo/diff.rs`
- `repo/mutations.rs`
- `repo/bookmarks.rs`
- `repo/git.rs`
- `repo/working_copy.rs`
- `repo/config.rs`
- `repo/environment.rs`
- `repo/resolve.rs`
- `repo/conflicts.rs`
- `repo/annotate.rs`
- `diff/compute.rs`
- `review.rs`
- `hash.rs`

## SwiftUI File Layout

```text
shell/mac/Sources/JayJay/
  App/
    Config/       AppSettings, AppearanceTypes, EditorTypes, TerminalTypes, AppSettingsTools, FontEnvironment, AppMetadata, JJEnvironment
    Window/       RepoWindowManager, RepositoryCommands, RepositoryFocus, RepositoryActions
    Watcher/      RepoFSWatcher
    JayJayApp.swift, CLIInstaller.swift, DebugBadge.swift, LaunchArguments.swift, SparkleUpdater.swift
  Repo/           RepoWindow, RepoSidebar, RepoViewModel, RepoViewModel+Actions, DAGView, DAGLayout, DAGRow, DAGRowViewModel, RepoPresentation, RepoToast, CommitBox, BookmarkPicker, UndoView
  Detail/         DetailView, DetailHeader, FileColumn, FileListView, AnnotateView, FileHistoryView
  Diff/           DiffSection, DiffColors, NativeDiffView, SideBySideDiffView
  Onboarding/     OnboardingView, WelcomeView
  Settings/       SettingsView, JJConfigView, AboutView, SettingsComponents
  Shared/         ChangeActions, ErrorMessages, ReviewStore, SheetViews, CommandPalette
```

## Review State

Review state is persistent across app restarts and local to the user.

- Canonical implementation: `jayjay_core::review::ReviewStore`.
- SwiftUI still has `Shared/ReviewStore.swift` with the same shape; GPUI uses the Rust store.
- Identity is caller-supplied. The store is a pure keyed store with no disk access and no hashing.
- Review identity is computed in `jayjay_core::repo::diff::entry::compute_review_identity` from `MergedTreeValue` blob IDs.
- Keying: `(changeId, path) -> { identity, file_marked, hunks }`.
- Valid marks require the stored identity to match the current `hunk.review_identity`.
- Same blob IDs preserve review across rebases or amends that do not change file bytes.
- Any byte change in the old or new side invalidates only that file's review.
- `is_hunk_reviewed(idx)` is true when the file is marked or the hunk set contains `idx`.
- Marking every hunk promotes to file-level review. Unmarking a hunk drops the file flag.
- Persistence path: `~/Library/Application Support/dev.hewig.jayjay/review_store.json` for Rust and `UserDefaults` for Swift.

## Presentation Surfaces

Use repo-level presentation types from `RepoPresentation.swift` instead of ad hoc booleans.

- **Inline state**: pane-scoped no-data, first-run guidance, and recoverable section errors.
- **Toast** (`RepoOverlayState.toast` / `RepoToast`): non-blocking action feedback, success messages, conflict follow-up, and lightweight warnings. Keep it short and allow at most one direct action.
- **HUD** (`RepoOverlayState.loading`): temporary blocking busy states where further interaction would be misleading or unsafe. Prefer quiet refreshes.
- **Alert** (`RepoAlertState`): short blocking interruptions that need acknowledgement or a simple binary choice. No forms, long copy, or more than two meaningful actions.
- **Sheet** (`RepoModalState` + `SheetContainer`): forms, previews, richer explanations, multi-step flows, or confirmations needing more context than an alert.

Do not escalate inline states into alerts or sheets just because they are errors. Scope the surface to the problem.
