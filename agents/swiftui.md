# SwiftUI Shell Guide

Load this file before changing SwiftUI file layout, view-model structure, diff rendering wiring, or presentation surfaces. Crate boundaries live in [Architecture Guide](architecture.md); shell-to-shell user-visible behavior belongs in [Shell Feature Parity Guide](shell-parity.md); review marks and notes in [Review State Guide](review-state.md).

## File Layout

```text
shell/mac/
├── Sources/JayJay/
│   ├── App/            JayJayApp, AppDelegate, CLIInstaller, SparkleUpdater, LaunchArguments,
│   │                   AppInfoCommands, HelpCommands; Config/, Watcher/, Window/
│   ├── Repo/           RepoWindow, RepoMenuHandler, ActivePane, RepoPresentation, RepoToast,
│   │   │               CommitBox, UndoView, SubmoduleAttentionSheet
│   │   ├── ContentView/    RepoContentView (+Sidebar, +StatusBar, +Toolbar, +CommandPalette,
│   │   │                   +Presentation, +Sheets)
│   │   ├── DAG/            DAGView (+BookmarkDrag, +ContextMenu, +RebaseDrag), DAGViewModel,
│   │   │                   DAGLayout, DAGRow (+GraphColumn, +Refs), DAGRowViewModel,
│   │   │                   rebase and bookmark-drag models and gesture policies
│   │   ├── Bookmarks/      BookmarkManagerView, BookmarkManagerRow, BookmarkPicker
│   │   ├── StackedPr/      StackedPrPanel (+Results), StackedPrNamer
│   │   └── ViewModel/
│   │       ├── Core/       RepoViewModel, +Refresh, +Selection, +AsyncSupport, +WindowActivity, StatusBarSnapshot
│   │       └── Actions/    +ChangeActions, +BookmarkActions, +FileActions, +GitActions, +Rebase,
│   │                       +Evolog, +CommitMessage, +WorkspaceActions
│   ├── Detail/         DetailView, ChangeDetailView (+PreviewColumn, +State, +ReviewState),
│   │   │               DetailHeader, DetailDescription, ReviewNotesSection, SplitSheet,
│   │   │               AnnotateView, FileHistoryView
│   │   ├── FileList/       FileColumn (+Actions, +Selection), FileRow, TreeFileList
│   │   └── Evolog/         EvologView, EvologViewModel, EvologDisplay
│   ├── Diff/           DiffSection (+Content, +ReviewActions, +NoteActions, +EditActions), DiffStore,
│   │                   ReviewNoteSheet
│   ├── DiffEdit/       DiffEditView, selection models
│   ├── Onboarding/     OnboardingView, WelcomeView
│   ├── Settings/       SettingsView (+Tools), JJConfigView, AboutView
│   ├── StatusBar/      StatusBarView and items
│   └── Shared/         ReviewStore (+Notes), ChangeActions/DAGActions/BookmarkActions protocols,
│       │               ChangeIdentity, ErrorMessages, HelpBook, HelpFeatureIndex, SheetContainer,
│       │               CopyableRow, accessibility ids
│       └── CommandPalette/  CommandPaletteItem, CommandPalettePanel, PaletteRoot (+RawJJ),
│                            CommandPaletteSupport
└── Packages/JayJayDiffUI/   AppKit diff rendering: NativeDiffView at the root, sources grouped
                             into Gutter/, Notes/, SideBySide/, Rendering/, Text/, Media/;
                             own test target
```

Group sibling files into a responsibility folder once a feature grows past a few files (see `Repo/DAG/`); `project.yml` globs `Sources/JayJay`, so moves need no project edits — except `Shared/AccessibilityIdentifiers.swift`, which the UITest target references by path.

The ViewModel owns `JayJayRepo`; all jj operations go through it. `Core/` holds lifecycle and state; `Actions/` holds mutation verbs; all are `extension RepoViewModel` files.

## Conventions

- **JayJayDiffUI boundary**: the package declares the gutter-action protocols (`DiffGutterContextActions` and its `Selection`/`Edit`/`Review`/`Note` sub-protocols, one file each under `Gutter/`) plus the anchor/summary value types in `Notes/`; the app target conforms via `DiffSection` and its `+EditActions`/`+ReviewActions`/`+NoteActions` extensions. The package stays app-agnostic and calls back only through those protocols.
- **File splitting**: SwiftLint opts into `file_length` and `function_parameter_count` for `Sources/JayJay`. One primary type per file, named after the type (private helpers used only there stay put; a view keeps its `#Preview`). Growing types split into `+Feature.swift` extension files (see `DAGView+*`, `DiffSection+*`, the `ViewModel/` folders) instead of growing one file; wide parameter lists become value types (e.g. `NoteAnchor`, `SplitSheetRequest`). Dense folders get responsibility subfolders (`Detail/FileList/`, `Repo/ContentView/`, the JayJayDiffUI `Gutter/`/`Notes/`/`Rendering/` layout).
- **Pinned paths**: `Shared/AccessibilityIdentifiers.swift` is referenced by exact path in `project.yml` (the UITests target compiles it directly); moving it breaks project generation. Everything else under `Sources/JayJay` is glob-included and moves freely.
- **Diff caching**: `Diff/DiffStore.swift` (`@Observable`) fronts an `actor DiffCache`, an LRU bounded by content bytes. Keys are content-addressed on the immutable commit id (never the mutable rev) plus compare side, whitespace mode, and path, so amends/rebases cannot serve stale diffs. `preload()` cancels the prior preload task.
- **Refresh pipeline** (`ViewModel/Core/RepoViewModel+Refresh.swift`): one cancel-and-replace `refreshTask`; FS-triggered refreshes are dropped while one is in flight; snapshots (e.g. `StatusBarSnapshot`) load off-thread and apply atomically. Commit-box drafts reseed only when the working-copy change id actually changes.

## Presentation Surfaces

Use repo-level presentation types from `RepoPresentation.swift` instead of ad hoc booleans.

- **Inline state**: pane-scoped no-data, first-run guidance, and recoverable section errors.
- **Toast** (`RepoOverlayState.toast` / `RepoToast`): non-blocking action feedback, success messages, conflict follow-up, and lightweight warnings. Keep it short and allow at most one direct action.
- **HUD** (`RepoOverlayState.loading`): temporary blocking busy states where further interaction would be misleading or unsafe. Prefer quiet refreshes.
- **Alert** (`RepoAlertState`): short blocking interruptions that need acknowledgement or a simple binary choice. No forms, long copy, or more than two meaningful actions.
- **Sheet** (`RepoModalState` + `SheetContainer`): forms, previews, richer explanations, multi-step flows, or confirmations needing more context than an alert.

Do not escalate inline states into alerts or sheets just because they are errors. Scope the surface to the problem.
