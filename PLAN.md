# jayjay — Implementation Plan

## Architecture

```
jayjay/
├── crates/
│   ├── jayjay-core/       # jj-lib wrapper, diff, syntax highlighting
│   │   └── src/
│   │       ├── repo/       # Rust modules: mod, log, diff, mutations, bookmarks, git, working_copy
│   │       ├── native_diff.rs  # Word-level diff + context collapsing
│   │       ├── syntax.rs       # tree-sitter highlighting (15 languages)
│   │       └── types.rs        # Shared types
│   ├── jayjay-uniffi/     # uniffi bindings → Swift
│   └── jayjay-cli/        # Native Rust CLI launcher
├── shell/
│   └── mac/               # macOS SwiftUI app
│       ├── Sources/JayJay/
│       │   ├── App/           # JayJayApp, AppSettings, RepoWindowManager
│       │   └── Views/         # RepoWindow, DetailView, DAGView, Diff/, Components/
│       ├── Package.swift      # Swift Package for bindings
│       └── project.yml        # xcodegen spec
├── Cargo.toml
├── Justfile
└── PLAN.md
```

## Phase 1: Rust Core + uniffi Bindings ✅

- [x] Scaffold Rust workspace with `jayjay-core` + `jayjay-uniffi`
- [x] Wire up jj-lib: open, log, show, describe, new, squash, abandon, rebase
- [x] Bookmarks: list, create, move, delete
- [x] Git: push, fetch (via jj CLI for auth handling)
- [x] Working copy snapshot + refresh
- [x] File restore (delete from disk for @, tree rewrite for others)
- [x] Ignore & untrack (append .gitignore + `jj file untrack`)
- [x] Split (via `jj split --paths`)
- [x] Rename detection (content similarity + filename matching)
- [x] Graph topology via `iter_graph()` with edge types
- [x] Conflict + empty status on changes
- [x] uniffi proc-macro bindings with `uniffi.toml` config
- [x] Native diff computation with jj-lib word-level diff
- [x] tree-sitter syntax highlighting (15 languages)
- [x] Context collapsing (3 lines around changes)
- [x] Commit with submodule orchestration (`git commit` in dirty submodules + `jj commit`)
- [x] `jj diff --stat` summary for AI message generation

## Phase 2: macOS SwiftUI App (MVP) ✅

- [x] WindowGroup app with CLI arg + folder picker
- [x] Multi-window via RepoWindowManager
- [x] Recent repos in File → Open Recent
- [x] Persistent sidebar width

### Core views
- [x] DAG graph with node types (working copy, empty, conflict) + edge lines
- [x] Change list with bookmarks, context menus (new, squash, abandon)
- [x] Detail panel: header, editable description, file list, diff view
- [x] File list: flat + tree view toggle, review checkboxes (working copy only)
- [x] File context menus: show in Finder, copy path, split, restore, ignore & untrack
- [x] Commit box with AI message generation (Apple Foundation Models)
- [x] Bookmark picker with create/delete

### Diff views
- [x] Native unified diff (NSTextView + NSAttributedString)
- [x] Side-by-side diff (NSSplitView, synced scroll, equal 50/50 split)
- [x] tree-sitter syntax highlighting in both modes
- [x] Context collapsing with separator lines
- [x] Auto-fallback: added/deleted files → unified, modified → side-by-side

### Actions + shortcuts
- [x] New change (⌘N)
- [x] Squash (⌘⇧S)
- [x] Abandon (⌘⌫)
- [x] Git push (⌘⇧P)
- [x] Git fetch (⌘⇧F)
- [x] Refresh (⌘R)
- [x] Space to toggle reviewed (working copy)
- [x] Arrow keys to navigate files

### Settings
- [x] Theme (system/light/dark)
- [x] Font scale
- [x] Side-by-side diff toggle
- [x] Ignore whitespace (plumbed, needs Rust integration)
- [x] Tree view for files

## Phase 3: Semantic Diff

### Done
- [x] tree-sitter integration (15 languages via Rust crates)
- [x] Native renderer replacing Monaco WebView
- [x] Context collapsing
- [x] Rename detection

### Remaining
- [ ] Structural/semantic diff: function-level summaries ("function `foo()` modified")
- [ ] AST node matching between old/new trees
- [ ] Collapsed-by-default structural view with expand-to-inline

## Phase 4: Polish + Platform Expansion

### macOS polish
- [ ] Undo/redo via `jj op log`
- [ ] Side-by-side diff: word-level highlighting within changed lines
- [ ] Ignore whitespace in diff (Rust-side integration)
- [ ] Auto-updates via Sparkle or Homebrew cask (`release-macos` skill)
- [ ] Drag-and-drop rebase in DAG view

### Cross-platform
- [ ] Linux shell (gtk-rs or slint) — shared Rust core
- [ ] Windows shell — shared Rust core

## Open Questions

- [ ] jj-lib API stability — pin version or track latest?
- [ ] Semantic diff: AST matching algorithm (simple name-based vs patience diff on nodes)
- [ ] Git submodule: deeper integration beyond commit orchestration?
- [ ] difftastic integration: vendor core modules or build own AST diff?
