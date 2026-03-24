# Roadmap

## Near-term

- [x] Absorb + Backout support (`jj absorb` / `jj backout`) — [#2](https://github.com/hewigovens/jayjay/issues/2)
- [x] Interdiff between arbitrary revisions (`jj diff --from X --to Y`) — [#4](https://github.com/hewigovens/jayjay/issues/4)
- [x] Conflict resolution UI (`jj resolve`) — [#1](https://github.com/hewigovens/jayjay/issues/1)
- [x] File annotate / blame view (`jj file annotate`) — [#3](https://github.com/hewigovens/jayjay/issues/3)
- [x] Graph revset filtering presets — [#5](https://github.com/hewigovens/jayjay/issues/5)
- [ ] Hunk-level split / interactive staging (`jj diffedit`) — [#6](https://github.com/hewigovens/jayjay/issues/6)
- [ ] Drag-and-drop bookmark on DAG to move it to another commit
- [ ] `jj tag create/list` (when jj stabilizes tag support)
- [ ] `jj rebase --after` reorder (need target picker UI)
- [ ] Tab-based multi-repo (multiple repos as tabs in same window)
- [ ] Landing page (GitHub Pages)

## Long-term

- [ ] Linux: native shell using [gtk4-rs](https://gtk-rs.org/)
- [ ] Windows: native shell using WinUI 3 or [GPUI](https://gpui.rs/)
- [ ] Semantic diff (tree-sitter AST diffing, function-level summaries)
- [ ] Drag-and-drop rebase in DAG

## Known Issues

- Copy from diff: O(lines) scan, may lag on 10k+ line diffs
- Side-by-side for new/deleted files falls back to unified (by design)

## Done

### Rust Core
- jj-lib: open, log, log_graph, show, describe, new, edit, squash, squash --into, abandon, rebase, split, graft, duplicate, merge, absorb, backout
- Interdiff: compare any two revisions via TreePair helpers
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
- Shift-click compare mode for interdiff between two revisions
- Persistent file review state (survives app restart, keyed by changeId+commitId+path)
- Batch split with file review checkboxes (space key), parallel split option
- Commit box with AI message generation (Codex → Claude → Apple Intelligence)
- Bookmark picker with push, rename, track, move forward
- Revset filter, auto-refresh via FS watcher
- Native Form-based settings: Appearance, Diff, Tools (editor/terminal), Jujutsu config, About
- Font family picker (System/Menlo/SF Mono/JetBrains/Fira/Cascadia) + size stepper + ⌘+/-/0 zoom
- Onboarding wizard with jj check + GitHub Desktop warning
- jj git init button for non-jj folders
- Undo via jj op log (⌘⇧U)
- Command palette (⌘⇧P): search commands, `!` prefix for jj CLI with inline output
- ⌘F find in diff view (native macOS find bar)
- Move to Working Copy (squash files from any change into @)
- SheetContainer reusable component for modal dialogs
- Protocol-based actions (ChangeActions, DAGActions, BookmarkActions)
- External editor + terminal integration with per-app cd handling
- CLI: clap-based, bundled in app, symlink installer, URL scheme for running instance
- View Remote Repository (git@ → https conversion)
- Friendly error messages (hides uniffi enum prefixes)
- In-app HUD toast for success messages
- Help menu: GitHub, jj docs, report issue
- Working copy file watcher with .gitignore-aware filtering
- Crash audit: data race fixes, force unwrap safety, shell injection prevention
- CI: GitHub Actions (Rust lint/test + Swift build)
- Release pipeline: `just release` (sign, notarize, zip, sha256)
