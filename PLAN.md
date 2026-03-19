# jayjay — Implementation Plan

## Status

- **Phase 1** (Rust Core): Done
- **Phase 2** (macOS MVP): Done
- **Phase 3** (Diff): Mostly done — semantic diff remaining
- **Phase 4** (Polish): In progress

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
│       ├── App/           # Entry point, settings, window manager
│       └── Views/         # DAG, detail, diff, components
├── Cargo.toml
├── Justfile
└── uniffi.toml
```

## Phase 1: Rust Core + uniffi Bindings ✅

- [x] jj-lib workspace: open, log, log_graph, show, describe, new, squash, abandon, rebase
- [x] Bookmarks: list, create, move, delete
- [x] Git: push, fetch (via jj CLI for SSH auth)
- [x] Working copy snapshot + refresh
- [x] File restore (disk delete for @, tree rewrite for others)
- [x] Ignore & untrack (.gitignore + `jj file untrack`)
- [x] Split with message (`jj split -m`)
- [x] Rename detection (content similarity + filename matching)
- [x] DAG graph via `iter_graph()` with edge types
- [x] Conflict + empty status on changes
- [x] Native diff: jj-lib word-level diff + context collapsing (3 lines)
- [x] tree-sitter syntax highlighting (15 languages)
- [x] Commit with submodule orchestration
- [x] `jj diff --stat` for AI message generation

## Phase 2: macOS SwiftUI App (MVP) ✅

- [x] WindowGroup with CLI arg, folder picker, multi-window
- [x] Recent repos (File → Open Recent)
- [x] Persistent sidebar width
- [x] DAG graph with lane-based fork/merge rendering
- [x] Change list with bookmarks, conflict/empty indicators
- [x] Context menus: new child, squash, abandon
- [x] Detail panel: header, editable description, file list, diff
- [x] File list: flat + tree view, review checkboxes (working copy only, space key)
- [x] File context menus: show in Finder, copy path, split, restore, ignore & untrack
- [x] Batch split: check files → Split button → sheet with description
- [x] Commit box with AI message generation (Apple Foundation Models)
- [x] Bookmark picker with create/delete
- [x] Unified + side-by-side diff with tree-sitter syntax highlighting
- [x] Context collapsing with separator lines
- [x] Copy strips line numbers and separators
- [x] Auto-fallback: added/deleted files → unified in side-by-side mode
- [x] Keyboard shortcuts: ⌘N, ⌘⇧S, ⌘⌫, ⌘⇧P, ⌘⇧F, ⌘R, Space
- [x] Revset filter (empty → reset default, invalid → empty list)
- [x] Default revset includes siblings (`@-+`)

## Phase 3: Diff ✅ (partial)

### Done
- [x] tree-sitter integration (15 languages)
- [x] Native NSTextView renderer (replaced Monaco WebView)
- [x] Side-by-side: NSSplitView with synced scroll, 50/50 split
- [x] Context collapsing, rename detection
- [x] Copy-stripping: line numbers + separators excluded from ⌘C

### Remaining
- [ ] Structural/semantic diff (function-level "foo() modified")
- [ ] Word-level highlighting within changed lines in side-by-side mode
- [ ] Ignore whitespace (Rust-side integration)

## Phase 4: Polish + Platform Expansion

### macOS
- [ ] Undo/redo via `jj op log`
- [ ] Auto-updates (Sparkle or Homebrew cask)
- [ ] Drag-and-drop rebase in DAG view
- [ ] Better DAG rendering (match jj log output more closely)

### Cross-platform
- [ ] Linux shell (gtk-rs or slint) — shared Rust core
- [ ] Windows shell — shared Rust core

## Known Issues

- Copy from diff uses O(lines) scan — may have brief delay on 10k+ line diffs
- Side-by-side diff for new/deleted files falls back to unified (by design)
- `.gitignore` diff may show no changes if working copy snapshot races with display

## Open Questions

- jj-lib API stability — pin version or track latest?
- Semantic diff: AST matching algorithm?
- difftastic: vendor core or build own AST diff?
- Git submodule: deeper integration beyond commit orchestration?
