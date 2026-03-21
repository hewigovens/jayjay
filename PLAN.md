# jayjay — Implementation Plan

## Beta Release Checklist

### P0 — Must ship
- [x] App icon (abstract blue jaybird)
- [x] App signing + notarization pipeline (`just release`)
- [x] Distribution: zip + Homebrew cask template
- [x] Crash/error audit (6 fixes: data races, force unwraps, shell injection)
- [ ] Set up `notarytool` keychain profile + run `just release`

### P1 — Done
- [x] Move bookmark forward (advance bookmark to @-)
- [x] Commit message prompt: shared constant from Rust
- [x] File tree building: moved to Rust (`file_tree.rs`)
- [x] Default revset constant: defined in Rust
- [x] Word-level highlighting in side-by-side diff
- [x] Ignore whitespace in diff (Rust-side, settings toggle)
- [x] External editor + terminal settings with auto-detection
- [ ] Landing page / website (GitHub Pages) — post-beta

### Post-beta

| Feature | Difficulty | Notes |
|---------|-----------|-------|
| `jj graft` (cherry-pick) | Easy | CLI shim, similar to split |
| `jj new A B` (merge) | Easy | Pass multiple parent IDs to `new_commit()` |
| `jj tag create/list` | Easy | Similar to bookmark CRUD |
| `jj annotate` (blame) | Medium | New view, parse `jj annotate` output or use jj-lib |
| Landing page (GitHub Pages) | Medium | Static site, screenshots |
| Semantic diff (tree-sitter AST) | Hard | AST diffing, function-level summaries |
| Drag-and-drop rebase in DAG | Hard | Hit testing, drag state machine, preview rendering |
| File watcher for working copy | Medium | Watch repo dir, not just op_heads |
| Linux shell (gtk-rs or slint) | Hard | New UI layer, shared Rust core |
| Windows shell | Hard | Same as Linux, plus Windows-specific paths |

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
