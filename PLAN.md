# jayjay — Implementation Plan

## Beta Release Checklist

### P0 — Must ship
- [ ] App icon
- [ ] App signing + notarization (Developer ID) — `just release`
- [ ] Distribution: zip + Homebrew cask (`brew install --cask jayjay`)
- [ ] Quick crash/error audit pass

### P1 — Should ship
- [ ] Landing page / website (GitHub Pages)
- [ ] Move bookmark forward (advance bookmark to @-)
- [ ] Commit message prompt: expose shared prompt from Rust
- [ ] File tree building: move to Rust for cross-platform reuse
- [ ] Default revset constant: define in Rust, expose to all shells

### P2 — Post-beta
- [ ] Ignore whitespace in diff (Rust-side integration)
- [ ] Semantic diff (function-level summaries via tree-sitter)
- [ ] Drag-and-drop rebase in DAG view
- [ ] Word-level highlighting in side-by-side diff
- [ ] File watcher for working copy changes (not just op_heads)
- [ ] Linux shell (gtk-rs or slint)
- [ ] Windows shell

---

## Architecture

```
jayjay/
├── crates/
│   ├── jayjay-core/       # jj-lib wrapper, diff engine, tree-sitter syntax
│   │   └── src/
│   │       ├── repo/      # Modules: mod, log, diff, mutations, bookmarks, git, working_copy, undo
│   │       ├── diff/      # Line diff, word diff, context collapsing, highlights
│   │       └── syntax.rs  # tree-sitter (17 languages)
│   ├── jayjay-uniffi/     # uniffi::Remote bindings → Swift
│   └── jayjay-cli/        # Native Rust CLI launcher
├── shell/mac/             # macOS SwiftUI app
│   └── Sources/JayJay/
│       ├── App/           # Config, Window, Watcher
│       ├── Repo/          # RepoWindow, RepoViewModel, DAGView, CommitBox, BookmarkPicker
│       ├── Detail/        # DetailView, FileListView
│       ├── Diff/          # DiffSection, NativeDiffView, SideBySideDiffView, DiffColors
│       ├── Onboarding/    # OnboardingView, WelcomeView
│       ├── Settings/      # SettingsView, JJConfigView, AboutView
│       └── Shared/        # ChangeActions
├── Justfile               # Root: build, run, test, lint, format
└── shell/justfile         # Shell: ffi, build, run, lint, format
```

## Known Issues
- Copy from diff: O(lines) scan, may lag on 10k+ line diffs
- Side-by-side for new/deleted files falls back to unified (by design)
- `justfile_directory()` unreliable with `mod` — use `git rev-parse` instead

---

## Done

### Phase 1: Rust Core
- jj-lib: open, log, log_graph, show, describe, new, squash, abandon, rebase, split
- Bookmarks: list, create, move, delete
- Git: push (with auto-track), fetch
- Working copy: snapshot, refresh, file restore, ignore & untrack
- Rename detection, conflict/empty status
- Native diff: jj-lib word-level + context collapsing (3 lines)
- tree-sitter syntax highlighting (17 languages)
- Submodule-aware commit

### Phase 2: macOS App
- SwiftUI + WindowGroup, multi-window, recent repos, CLI launcher
- DAG graph with lane-based fork rendering
- Detail panel: header, description, file list (flat + tree), diff
- Unified + side-by-side diff, copy strips line numbers
- Batch split with file review checkboxes (space key)
- Commit box with AI message generation
- Bookmark picker with per-bookmark push
- Revset filter, auto-refresh via FS watcher
- Settings: tabbed (Appearance, Diff, Jujutsu config, About)
- Keyboard shortcuts

### Phase 3: Diff
- tree-sitter (17 languages), native NSTextView renderer
- Side-by-side: NSSplitView, synced scroll
- Context collapsing, rename detection, copy stripping

### This session
- First-launch jj detection + onboarding wizard
- LLM commit messages: Codex/Claude CLI → Apple Foundation Models
- Undo: jj op restore with operation log viewer
- Confirmation dialogs for destructive actions
- AI provider detection moved to Rust
- isVisibleChange consolidated in Rust
- JJ environment check moved to Rust
- uniffi::Remote for all types
- Module splits (diff/, uniffi/)
- Feature-based Swift folder structure
- tree-sitter: 17 languages
- just lint/format recipes
- NSUserNotification → UNUserNotificationCenter
- Conventional commit format for AI messages
