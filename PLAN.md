# jayjay — Implementation Plan

## Beta Status

All P0 and P1 items complete. Ready for release.

## Roadmap

| Feature | Difficulty | Notes |
|---------|-----------|-------|
| `jj tag create/list` | Easy | When jj stabilizes tag support |
| `jj rebase --after` (reorder) | Medium | Need target picker UI |
| `jj annotate` (blame) | Medium | New view, parse output or use jj-lib |
| Git submodule testing | Medium | Submodule-aware commit exists but needs real-world testing |
| Landing page (GitHub Pages) | Medium | Static site, screenshots |
| Tab-based multi-repo | Medium | Multiple repos as tabs in same window |
| Semantic diff (tree-sitter AST) | Hard | AST diffing, function-level summaries |
| Drag-and-drop rebase in DAG | Hard | Hit testing, drag state machine, preview |
| Linux + Windows shell | Hard | gpui-component (Apache 2.0, 60+ components) or Slint — shared Rust core, no uniffi needed |

## Known Issues

- Copy from diff: O(lines) scan, may lag on 10k+ line diffs
- Side-by-side for new/deleted files falls back to unified (by design)

## Done

### Rust Core
- jj-lib: open, log, log_graph, show, describe, new, edit, squash, squash --into, abandon, rebase, split, graft, duplicate, merge
- Bookmarks: list, create, move, delete, rename, track
- Git: push (with auto-track), fetch, remote URL
- Working copy: snapshot, refresh, file restore, ignore & untrack
- Rename detection, conflict/empty status, file tree building
- Diff engine: LCS line diff, jj-lib word-level, context collapsing, ignore whitespace
- tree-sitter syntax highlighting (18 languages)
- Submodule-aware commit
- AI commit message generation (Codex/Claude CLI)
- uniffi::Remote for all types (zero wrapper boilerplate)
- Cross-platform constants: DEFAULT_REVSET, COMMIT_MESSAGE_PROMPT
- JJ environment check, AI provider detection

### macOS App
- SwiftUI + WindowGroup, multi-window (one per repo, URL scheme dedup)
- DAG graph with lane-based fork rendering (DAGView, DAGLayout, DAGRow)
- Detail panel: header, description, file list (flat + tree), diff
- Unified + side-by-side diff with word-level highlighting, DiffLayoutManager for gap-free rendering
- Persistent file review state (survives app restart, keyed by changeId+commitId+path)
- Batch split with file review checkboxes (space key)
- Commit box with AI message generation (Codex → Claude → Apple Intelligence)
- Bookmark picker with push, rename, track, move forward
- Revset filter, auto-refresh via FS watcher
- Native Form-based settings: Appearance, Diff, Tools (editor/terminal), Jujutsu config, About
- Font family picker (System/Menlo/SF Mono/JetBrains/Fira/Cascadia) + size stepper + ⌘+/-/0 zoom
- Onboarding wizard with jj check + GitHub Desktop warning
- jj git init button for non-jj folders
- Undo via jj op log (⌘Z)
- Protocol-based actions (ChangeActions, DAGActions, BookmarkActions)
- External editor + terminal integration with per-app cd handling
- CLI: clap-based, bundled in app, symlink installer, URL scheme for running instance
- View Remote Repository (git@ → https conversion)
- Friendly error messages (hides uniffi enum prefixes)
- In-app HUD toast for success messages
- Help menu: GitHub, jj docs, report issue
- Crash audit: data race fixes, force unwrap safety, shell injection prevention
- CI: GitHub Actions (Rust lint/test + Swift build)
- Release pipeline: `just release` (sign, notarize, zip, sha256)
