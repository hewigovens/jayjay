# jayjay — Implementation Plan

## Beta Status

All P0 and P1 items complete. Only remaining step: `notarytool` keychain setup + `just release`.

## Post-beta

| Feature | Difficulty | Notes |
|---------|-----------|-------|
| `jj tag create/list` | Easy | When jj stabilizes tag support |
| `jj rebase --after` (reorder) | Medium | Need target picker UI |
| `jj annotate` (blame) | Medium | New view, parse output or use jj-lib |
| Landing page (GitHub Pages) | Medium | Static site, screenshots |
| File watcher for working copy | Medium | Watch repo dir, not just op_heads |
| Semantic diff (tree-sitter AST) | Hard | AST diffing, function-level summaries |
| Drag-and-drop rebase in DAG | Hard | Hit testing, drag state machine, preview |
| Linux shell (gtk-rs or slint) | Hard | New UI layer, shared Rust core |
| Windows shell | Hard | Same as Linux, plus Windows-specific paths |

## Known Issues

- Copy from diff: O(lines) scan, may lag on 10k+ line diffs
- Side-by-side for new/deleted files falls back to unified (by design)

## Done

### Rust Core
- jj-lib: open, log, log_graph, show, describe, new, edit, squash, squash --into, abandon, rebase, split, graft, duplicate, merge
- Bookmarks: list, create, move, delete, rename, track
- Git: push (with auto-track), fetch
- Working copy: snapshot, refresh, file restore, ignore & untrack
- Rename detection, conflict/empty status, file tree building
- Diff engine: LCS line diff, jj-lib word-level, context collapsing, ignore whitespace
- tree-sitter syntax highlighting (17 languages)
- Submodule-aware commit
- AI commit message generation (Codex/Claude CLI)
- uniffi::Remote for all types (zero wrapper boilerplate)
- Cross-platform constants: DEFAULT_REVSET, COMMIT_MESSAGE_PROMPT
- JJ environment check, AI provider detection

### macOS App
- SwiftUI + WindowGroup, multi-window, recent repos, CLI launcher
- DAG graph with lane-based fork rendering (DAGView, DAGLayout, DAGRow)
- Detail panel: header, description, file list (flat + tree), diff
- Unified + side-by-side diff with word-level highlighting, DiffLayoutManager for gap-free rendering
- Batch split with file review checkboxes (space key)
- Commit box with AI message generation (Codex → Claude → Apple Intelligence)
- Bookmark picker with push, rename, track, move forward
- Revset filter, auto-refresh via FS watcher
- Settings: Appearance, Diff, Tools (editor/terminal with auto-detection), Jujutsu config, About
- Onboarding wizard with jj check + GitHub Desktop warning
- Undo via jj op log (⌘Z)
- Protocol-based actions (ChangeActions, DAGActions, BookmarkActions)
- External editor + terminal integration with "Open in" context menu
- Crash audit: data race fixes, force unwrap safety, shell injection prevention
- CI: GitHub Actions (Rust lint/test + Swift build)
- Release pipeline: `just release` (sign, notarize, zip, sha256)
