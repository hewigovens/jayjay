# jayjay — Implementation Plan

## Beta Release Checklist

### P0 — Must ship
- [ ] Crash audit: fix all known crash paths (NSView constraints, side-by-side toggle)
- [ ] Error handling: show user-friendly messages, never fail silently
- [ ] Logging: `os.Logger` structured logging throughout the app
- [ ] Crash reporting: Sentry SDK (opt-in, privacy-respecting)
- [ ] App signing + notarization (Developer ID)
- [ ] Distribution: DMG or zip + Homebrew cask (`brew install --cask jayjay`)
- [ ] First-launch: detect if `jj` is installed, show install instructions if not
- [ ] `just release` recipe using `release-macos` skill

### P1 — Should ship
- [ ] LLM commit messages: Codex CLI / Claude CLI → Apple Foundation Models → manual
- [ ] App icon refinement (current jaybird SVG)
- [ ] Landing page / website (GitHub Pages)
- [ ] Undo: `jj op restore` with operation log viewer
- [ ] Confirmation dialogs for destructive actions (abandon, delete bookmark)
- [ ] Move bookmark forward (common workflow: advance bookmark to @-)
- [ ] Screenshots for README

### P2 — Post-beta
- [ ] Ignore whitespace in diff (Rust-side integration)
- [ ] Semantic diff (function-level summaries via tree-sitter AST matching)
- [ ] Drag-and-drop rebase in DAG view
- [ ] Word-level highlighting in side-by-side diff
- [ ] File watcher for working copy changes (not just op_heads)
- [ ] Linux shell (gtk-rs or slint)

---

## Architecture

```
jayjay/
├── crates/
│   ├── jayjay-core/       # jj-lib wrapper, diff engine, tree-sitter syntax
│   │   └── src/repo/      # Modules: mod, log, diff, mutations, bookmarks, git, working_copy
│   ├── jayjay-uniffi/     # uniffi proc-macro bindings → Swift
│   └── jayjay-cli/        # Native Rust CLI launcher
├── shell/mac/             # macOS SwiftUI app
│   └── Sources/JayJay/
│       ├── App/           # Entry point, settings, window manager, FS watcher
│       └── Views/         # DAG, detail, diff, components, shared
├── Cargo.toml
├── Justfile               # Root: delegates to shell submodule
└── shell/justfile         # Build recipes: ffi, build, run
```

## Completed

### Phase 1: Rust Core ✅
- jj-lib: open, log, log_graph, show, describe, new, squash, abandon, rebase, split
- Bookmarks: list, create, move, delete
- Git: push (with auto-track), fetch
- Working copy: snapshot, refresh, file restore, ignore & untrack
- Rename detection, conflict/empty status
- Native diff: jj-lib word-level + context collapsing (3 lines)
- tree-sitter syntax highlighting (15 languages)
- Submodule-aware commit

### Phase 2: macOS App ✅
- SwiftUI + WindowGroup, multi-window, recent repos, CLI launcher
- DAG graph with lane-based fork rendering
- Detail panel: header, description, file list (flat + tree), diff
- Unified + side-by-side diff, copy strips line numbers
- Batch split with file review checkboxes (space key)
- Commit box with AI message generation
- Bookmark picker with per-bookmark push
- Revset filter, auto-refresh via FS watcher
- Settings: tabbed (Appearance, Diff, Jujutsu config, About)
- Keyboard shortcuts: ⌘N, ⌘⇧S, ⌘⌫, ⌘⇧P, ⌘⇧F, ⌘R, Space, ↑/↓

### Phase 3: Diff ✅ (partial)
- tree-sitter (15 languages), native NSTextView renderer
- Side-by-side: NSSplitView, synced scroll
- Context collapsing, rename detection, copy stripping

## Known Issues
- Copy from diff: O(lines) scan, may lag on 10k+ line diffs
- Side-by-side for new/deleted files falls back to unified (by design)
- `justfile_directory()` unreliable with `mod` — use `git rev-parse` instead
- NSUserNotification deprecated on newer macOS — migrate to UNUserNotificationCenter
