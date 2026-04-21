# Roadmap

JayJay now covers most common jj history, diff, bookmark, conflict, and Git flows, plus raw `jj` execution via `!` in the command palette. The next phase is less about basic command parity and more about making jj-native workflows feel faster and more visual than the CLI.

## Near-term

- [ ] Change evolution history (`jj evolog`)
  Goal: show prior versions of a rewritten change with diffs — jj's killer feature, and one of the clearest places JayJay can beat git-native GUIs
- [ ] Stack surgery polish (`jj rebase --after` / `--before` and related flows)
  Current baseline: drag-to-rebase already handles "onto". Next: make insert-after / insert-before flows, descendant behavior, and previews clearer and more visual
- [ ] Diff edit polish
  Next: change-wide select all / clear all, stronger unsupported-file messaging, better topology copy
- [ ] Saved revsets library
  Goal: move beyond the six preset chips. Ship a named revset library (authored by you, touching file, fork point of x, commits with no children, etc.) and a "save this revset" action so users can build their own
- [ ] Command palette polish
  Next: command history, better inline output, and better discoverability for `! jj ...`
- [ ] GitHub integration via `gh` CLI
  Current baseline: PR link + status checks already appear in the status bar. Next: create-PR shortcut from the bookmark menu. Keep the footprint small — we drive `gh`, we don't reimplement it

## Longer-term

- [ ] Tag UI (`jj tag ...`) once jj stabilizes the model and command surface
- [ ] Multi-repo tabs or richer workspace switching model
- [ ] Advanced DAG reordering
  Scope: drag a whole subtree, insert before / after siblings, and preview descendant movement before committing the rewrite
- [ ] Semantic diff (tree-sitter AST diffing, function-level summaries)
- [ ] AI-native integration via ACP ([Agent Client Protocol](https://agentclientprotocol.com/))
  Not a chat tab, not a terminal. Speak ACP so any ACP-compatible agent (Claude Code, Codex, Zed's agent) can drive jj operations through JayJay — describe, split, squash, rebase — with structured tool calls and the agent's reasoning visible in JayJay's own surface. Binds naturally to the existing op log + persistent review state. Big scope, worth doing right

## TBD

- [ ] Linux: native shell using gtk4-rs
- [ ] Windows: native shell using GPUI

## Known Issues

- Side-by-side for new/deleted files falls back to unified (by design)

## Done

### Major Milestones
- [x] Absorb + Backout support (`jj absorb` / `jj backout`) — [#2](https://github.com/hewigovens/jayjay/issues/2)
- [x] Interdiff between arbitrary revisions (`jj diff --from X --to Y`) — [#4](https://github.com/hewigovens/jayjay/issues/4)
- [x] Conflict resolution UI (`jj resolve`) — [#1](https://github.com/hewigovens/jayjay/issues/1)
- [x] File annotate / blame view (`jj file annotate`) — [#3](https://github.com/hewigovens/jayjay/issues/3)
- [x] Graph revset filtering presets — [#5](https://github.com/hewigovens/jayjay/issues/5)
- [x] Change-wide diff edit mode (`jj diffedit`) — [#6](https://github.com/hewigovens/jayjay/issues/6)
- [x] Landing page (GitHub Pages)

### Rust Core
- jj-lib: open, log, log_graph, show, describe, new, edit, squash, squash --into, abandon, rebase, split, graft, duplicate, merge, absorb, backout
- Interdiff: compare any two revisions via TreePair helpers
- Diff edit engine: apply selected files/hunks/line ranges to child, parallel, working-copy, or remove-from-source destinations
- Revset + fileset alias resolution from jj config
- Bookmarks: list, create, move, delete, rename, track
- Git: push (with auto-track), fetch, remote URL
- GitHub: `gh pr view` parsing for PR link + checks
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
- DAG keyboard navigation (j/k, arrows, ctrl-n/p)
- Drag-to-rebase with hover preview, confirmation sheet, and undo toast
- Detail panel: header, description, file list (flat + tree), diff
- Unified + side-by-side diff with word-level highlighting, DiffLayoutManager for gap-free rendering
- Image diff rendering (PNG/JPG/GIF/WebP/HEIC/BMP/TIFF/ICNS) + rendered SVG toggle
- Diff edit mode with dedicated selection UI, gutter checkboxes, quick working-copy abandon shortcut, and topology-aware destinations
- Synced gutter view for unified diff line numbers; copy now excludes gutter content without text-stripping hacks
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
- Status bar PR link + checks for the selected bookmark via `gh`
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
