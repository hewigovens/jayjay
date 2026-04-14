# JayJay

Native macOS GUI for Jujutsu version control. Rust core + SwiftUI shell.

## Build

```bash
just build    # Build debug app
just run      # Build and launch
just lint     # Clippy + SwiftLint
just release  # Sign, notarize, package
```

## Release Pipeline

Releases are not complete after `just release` alone. The full release flow is:

1. Bump version and build number in **all four** sources: `shell/mac/project.yml`, `shell/mac/JayJay.xcodeproj/project.pbxproj`, `crates/jayjay-cli/Cargo.toml`, and `shell/justfile` (the last one is hardcoded — easy to forget).
2. **Write release notes to `releases/<version>.html`** (HTML body, no wrapper tags). `update-appcast.py` reads this file and embeds it as the `<description>` block in the appcast entry. Releases without a notes file print a warning and ship without a description — never acceptable for a published release.
3. Run `just build` to verify the release version still builds cleanly.
4. Run `just release` to build, sign, notarize, zip, produce the SHA-256, and prepend the entry to `docs/appcast.xml`.
5. Commit the version bumps + `releases/<version>.html` + `docs/appcast.xml` change as `release: <version> (build N)`.
6. Push `main`, then create and push the `v<version>` git tag.
7. `gh release create v<version> build/release/JayJay-<version>.zip --notes "<copy of releases/<version>.html or matching markdown>"` — the GitHub release notes should mirror the appcast description.
8. Update and push the Homebrew tap in `../tap` — bump `version "X.Y.Z,build"` and `sha256 "..."` in `Casks/jayjay.rb`.

For JayJay specifically:
- `just release` produces the notarized zip in `build/release/`
- the GitHub release must include the zip asset and its SHA-256
- Sparkle depends on `docs/appcast.xml` matching the uploaded release asset, **including a `<description>` block** sourced from `releases/<version>.html`
- Homebrew depends on `../tap/Casks/jayjay.rb` matching the uploaded release asset and SHA-256

**Always append release notes.** Every release entry in `docs/appcast.xml` and every GitHub release page must carry user-visible details (what's new, fixes, internal changes). Releases without notes are not just an oversight — Sparkle shows the description in the in-app update prompt, so an empty entry tells users "update to nothing".

## Principles

1. **First Principles** — Understand the problem before coding. Ask "why" before "how". Don't cargo-cult solutions from other apps — jj's model is fundamentally different from git's.
2. **DRY** — Don't repeat yourself. Extract shared logic into modules. If you copy-paste, refactor.
3. **KISS** — Keep it simple. The simplest correct solution wins. Three lines of code beat a premature abstraction.
4. **Single Responsibility** — Each file/module does one thing. Each function has one job.
5. **Cross-platform core** — All business logic stays in Rust. Swift/platform code is only for rendering.
6. **Terse comments** — Code should be self-explanatory; comment only non-obvious *why*. When a comment is needed, one concise line. No multi-line doc blocks, no restating the code, no obvious-from-context commentary.

## Architecture: MVVM

```
┌─────────────┐     uniffi     ┌──────────────┐     @Observable     ┌───────────┐
│  Rust Core  │ ──────────────▶│  ViewModel   │ ──────────────────▶│  SwiftUI  │
│  (Model)    │                │              │                     │  (View)   │
│  jayjay-core│◀──────────────│ RepoViewModel│◀──────────────────│  DAGView  │
│             │   sync calls   │              │   user actions      │ DetailView│
└─────────────┘                └──────────────┘                     └───────────┘
```

- **Model** (`crates/jayjay-core/`): jj-lib wrapper, diff engine, tree-sitter syntax. Pure Rust, no platform code. Split into focused modules: `repo/mod.rs`, `repo/log.rs`, `repo/diff.rs`, `repo/mutations.rs`, `repo/bookmarks.rs`, `repo/git.rs`, `repo/working_copy.rs`, `repo/config.rs`, `repo/environment.rs`, `repo/resolve.rs`, `repo/conflicts.rs`, `repo/annotate.rs`, `diff/compute.rs`.
- **Bindings** (`crates/jayjay-uniffi/`): Thin uniffi layer. No business logic — just type conversion.
- **ViewModel** (`Repo/RepoViewModel.swift` + `RepoViewModel+Actions.swift`): `@Observable` class. Owns the `JayJayRepo` instance. All jj operations go through here. Async operations use `Task.detached` → `MainActor.run`.
- **Views** (feature folders): Pure SwiftUI. No jj logic. Receive data and callbacks from ViewModel.

## File Organization

```
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

Each file should be **under 300 lines**. If it grows beyond that, split by responsibility.

## Presentation Surfaces

Use repo-level presentation types from `RepoPresentation.swift` instead of ad hoc booleans.

- **Inline state** — Use inline empty/error views for pane-scoped no-data, first-run guidance, and recoverable section errors. If the rest of the window can stay interactive, keep it inline.
- **Toast** (`RepoOverlayState.toast` / `RepoToast`) — Use for non-blocking action feedback, success messages, conflict follow-up, and lightweight warnings. Keep it short and allow at most one direct action such as Undo.
- **HUD** (`RepoOverlayState.loading`) — Use only for temporary blocking busy states where further interaction would be misleading or unsafe. Prefer quiet refreshes over showing a HUD.
- **Alert** (`RepoAlertState`) — Use for short blocking interruptions that need acknowledgement or a simple binary choice. No forms, no long copy, no more than two meaningful actions.
- **Sheet** (`RepoModalState` + `SheetContainer`) — Use for forms, previews, richer explanations, multi-step flows, or confirmations that need more context than an alert.
- **Do not escalate inline states** into alerts or sheets just because they are errors. Scope the surface to the scope of the problem.

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
jj split --paths FILE -m "msg" # Split files to new change
jj bookmark set <name> -r <rev>  # Set bookmark on a specific change
jj git push                    # Push bookmarks
jj git fetch                   # Fetch from remote
jj fix                         # Auto-format with rustfmt + swiftformat
```

### Do NOT use

- `git commit/add/push/stash/branch` — use jj equivalents
- `git rebase -i` — use `jj squash`, `jj split`, `jj rebase`

## Design Context

### Users
Developers who use jj (Jujutsu) for version control. They value keyboard-driven workflows, fast iteration, and tools that don't get in the way. Coming from git, they expect familiar UX patterns adapted for jj's unique model.

### Brand Personality
**Clean, modern, approachable.** The blue jaybird mascot adds personality without being childish.

### Aesthetic Direction
- **Playful with the jaybird theme**: blue gradient (`#3B82F6` to `#1E3A8A`), orange accent (`#F59E0B`), light blue (`#93C5FD`)
- **Reference**: zed.dev — technical but beautiful
- **Anti-reference**: cluttered enterprise tools, generic SaaS dashboards
- **Both light and dark modes**, following system preference
- **macOS-native feel**: SF Symbols, system fonts, native controls

### Design Principles
1. **Native first** — SwiftUI Form, system fonts, SF Symbols. Don't reinvent platform patterns.
2. **Keyboard-driven** — every action reachable via command palette or shortcut.
3. **Information density without clutter** — progressive disclosure for advanced operations.
4. **Performance is UX** — no loading spinners where avoidable. Quiet refreshes.
5. **Jujutsu-native** — embrace jj's model (changes, not commits; bookmarks, not branches).
