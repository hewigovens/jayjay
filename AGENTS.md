# Agent Instructions for jayjay

## Principles

1. **First Principles** — Understand the problem before coding. Ask "why" before "how". Don't cargo-cult solutions from other apps — jj's model is fundamentally different from git's.
2. **DRY** — Don't repeat yourself. Extract shared logic into modules. If you copy-paste, refactor.
3. **KISS** — Keep it simple. The simplest correct solution wins. Three lines of code beat a premature abstraction.
4. **Single Responsibility** — Each file/module does one thing. Each function has one job.
5. **Cross-platform core** — All business logic stays in Rust. Swift/platform code is only for rendering.

## Architecture: MVVM

```
┌─────────────┐     uniffi     ┌──────────────┐     @Observable     ┌───────────┐
│  Rust Core  │ ──────────────▶│  ViewModel   │ ──────────────────▶│  SwiftUI  │
│  (Model)    │                │              │                     │  (View)   │
│  jayjay-core│◀──────────────│ RepoViewModel│◀──────────────────│  DAGView  │
│             │   sync calls   │              │   user actions      │ DetailView│
└─────────────┘                └──────────────┘                     └───────────┘
```

- **Model** (`crates/jayjay-core/`): jj-lib wrapper, diff engine, tree-sitter syntax. Pure Rust, no platform code. Split into focused modules: `repo/mod.rs`, `repo/log.rs`, `repo/diff.rs`, `repo/mutations.rs`, `repo/bookmarks.rs`, `repo/git.rs`, `repo/working_copy.rs`.
- **Bindings** (`crates/jayjay-uniffi/`): Thin uniffi layer. No business logic — just type conversion.
- **ViewModel** (`Repo/RepoViewModel.swift`): `@Observable` class. Owns the `JayJayRepo` instance. All jj operations go through here. Async operations use `Task.detached` → `MainActor.run`.
- **Views** (feature folders): Pure SwiftUI. No jj logic. Receive data and callbacks from ViewModel.

## File Organization (by feature)

```
shell/mac/Sources/JayJay/
  App/
    Config/       AppSettings, AppSettingsTypes, AppSettingsTools, FontEnvironment, AppMetadata, JJEnvironment
    Window/       RepoWindowManager, RepositoryCommands, RepositoryFocus, RepositoryActions
    Watcher/      RepoFSWatcher
    JayJayApp.swift, CLIInstaller.swift, LaunchArguments.swift, AppInfoCommands.swift
  Repo/           RepoWindow, RepoViewModel, DAGView, DAGLayout, DAGRow, CommitBox, BookmarkPicker, UndoView
  Detail/         DetailView, FileListView
  Diff/           DiffSection, DiffColors, NativeDiffView, SideBySideDiffView
  Onboarding/     OnboardingView, WelcomeView
  Settings/       SettingsView, JJConfigView, AboutView, SettingsComponents
  Shared/         ChangeActions, ErrorMessages, ReviewStore
```

Each file should be **under 300 lines**. If it grows beyond that, split by responsibility.

## Version Control

This repo uses **Jujutsu (jj)**, not git. All version control operations should use `jj` commands.

### Key differences from git

- **No staging area**: jj auto-snapshots the working copy
- **Changes, not commits**: identified by change IDs (reverse hex), not commit hashes
- **`@` is the working copy**, `@-` is the parent
- **Mutable history**: all changes are rewritable

### Common commands

```bash
jj st                          # Status
jj log --limit 10              # Recent history
jj diff                        # Working copy diff
jj describe -m "message"       # Set description for @
jj commit -m "message"         # Describe @ + start new change
jj squash                      # Squash @ into parent
jj abandon <rev>               # Drop a change
jj split --paths FILE -m "msg" # Split files to new change
jj bookmark create <name> -r <rev>  # Create bookmark on a specific change
jj git push --bookmark <name>  # Push specific bookmark
jj git fetch                   # Fetch from remote
```

### Do NOT use

- `git commit/add/push/stash/branch` — use jj equivalents
- `git rebase -i` — use `jj squash`, `jj split`, `jj rebase`

## Build

```bash
just build    # Full build (Rust FFI + Xcode)
just run      # Build and launch
just test     # Run Rust tests
```
