# Roadmap

JayJay now covers most common jj history, diff, bookmark, conflict, and Git flows, plus raw `jj` execution via `!` in the command palette. The next phase is less about basic command parity and more about making jj-native workflows feel faster and more visual than the CLI.

## Near-term

- [ ] Stack surgery polish (`jj rebase --after` / `--before` and related flows)
  Current baseline: drag-to-rebase already handles "onto". Next: make insert-after / insert-before flows, descendant behavior, and previews clearer and more visual
- [ ] Diff edit polish
  Next: change-wide select all / clear all, stronger unsupported-file messaging, better topology copy
- [ ] Saved revsets library
  Goal: move beyond the six preset chips. Ship a named revset library (authored by you, touching file, fork point of x, commits with no children, etc.) and a "save this revset" action so users can build their own
- [ ] Command palette polish
  Next: command history, better inline output, and better discoverability for `jj ...` / `! ...`
- [ ] Evolog polish
  Current baseline: read-only viewer with interdiff against current and "Copy `jj restore` command" — already useful for recovery. Next: inline restore action, hide-snapshots toggle, run-of-snapshots collapsing

## Longer-term

- [ ] GPUI shell (Alpha) — Linux + Windows native shell using GPUI (one Rust shell, identical look across both). Mac stays on SwiftUI. OS integration via freedesktop standards (`.desktop`, hicolor, D-Bus via `notify-rust` / `ashpd` / `zbus`) — no GTK dependency.
  - [x] Read-only milestone — at parity with the SwiftUI shell for read flows:
    - [x] Per-file history viewer — surfaced from the file row "Show History" context menu
    - [x] Auto-refresh on filesystem changes — `notify`-based watcher on `.jj/repo/op_heads/heads` + working tree, debounced
    - [x] Onboarding / no-repo state — welcome card with `jj git init` hint when the path isn't a jj repo
    - [x] Reveal-to-changeId — `LogView::reveal_change_id` scrolls + selects, used by file-history and bookmark clicks
    - [x] Bookmark bar in sidebar header + workspace pill in status bar (read-only; switching is a write-action, deferred)
    - [x] Persistent file review (space to toggle, `n / total reviewed` count, content-hash auto-invalidation so a re-edited file flips back to unreviewed; survives jj rebases that produce identical content) — `jayjay_core::review::ReviewStore` is the canonical impl; SwiftUI's UserDefaults-backed copy is the next migration target.
  - [x] Diff view selection + copy — column-precision cross-line text selection in unified and side-by-side (per-side, independent) diffs, custom selection layer on the gutter/content split, gutter excluded from copy structurally. Cmd+C copies the joined slice; double-click selects a word; glyph advance measured via `text_system::ch_advance` so multi-line highlights stop at each line's actual EOL (matches VSCode/GitHub Desktop). Same trim applied to the SwiftUI diff via `DiffLayoutManager.rectArray` override. Polish remaining: cross-hunk-gap selection through `…N hidden lines…` separators, triple-click line-select, Cmd+A select-all.
  - [ ] Write milestone — first set of mutating actions, all routed through the existing `RepoViewModel::refresh()` so the FS watcher + review store stay coherent:
    - [ ] Describe + commit box (edit working-copy description, AI message generation reusing `jayjay_core::COMMIT_MESSAGE_PROMPT`)
    - [ ] `jj new` button on the toolbar
    - [ ] Abandon / squash-into-parent from the change context menu, with confirmation sheet
    - [ ] Split (file-level) using the read-only review checkboxes as the selection model — closes the loop on the persistent review store
    - [ ] Bookmark create / move-forward / push, surfaced from the existing bookmark picker dropdown
    - [ ] Undo via `jj op log` (`⌘⇧U`), mirroring the SwiftUI shortcut
  - [ ] Drag-to-rebase + conflict resolve — DAG row drag with hover preview + confirmation sheet; basic `jj resolve` UI (sidecar diff, "Use Ours/Theirs" buttons). Higher complexity, deferred until the basic write actions land.
  - [ ] Linux/Windows polish — `.desktop` entry + hicolor icon set, D-Bus notifications via `notify-rust`/`zbus` for long-running ops, file picker fallback when `gpui::Window::prompt_for_paths` isn't available on the target platform.
- [ ] Tag UI (`jj tag ...`) once jj stabilizes the model and command surface
- [ ] Multi-repo tabs or richer workspace switching model
- [ ] Advanced DAG reordering
  Scope: drag a whole subtree, insert before / after siblings, and preview descendant movement before committing the rewrite
- [ ] Semantic diff (tree-sitter AST diffing, function-level summaries)
- [ ] AI-native integration via ACP ([Agent Client Protocol](https://agentclientprotocol.com/))
  Not a chat tab, not a terminal. Speak ACP so any ACP-compatible agent (Claude Code, Codex, Zed's agent) can drive jj operations through JayJay — describe, split, squash, rebase — with structured tool calls and the agent's reasoning visible in JayJay's own surface. Binds naturally to the existing op log + persistent review state. Big scope, worth doing right

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
- [x] Pull Request creation from bookmark right-click (DAG row + Bookmark Manager) — [#24](https://github.com/hewigovens/jayjay/pull/24)
- [x] Change evolution viewer (`jj evolog`) with interdiff against current + Copy `jj restore` command
- [x] Landing page (GitHub Pages)

### Rust Core
- jj-lib: open, log, log_graph, show, describe, new, edit, squash, squash --into, abandon, rebase, split, graft, duplicate, merge, absorb, backout
- Evolog: in-process via `jj_lib::evolution::walk_predecessors` (no CLI shell-out)
- File annotate (blame): in-process via `jj_lib::annotate::FileAnnotator` (no CLI shell-out)
- File history: type-safe revset built from `RevsetExpression::filter` + `FilesetExpression::file_path` (no string formatting)
- Interdiff: compare any two revisions via TreePair helpers
- Diff edit engine: apply selected files/hunks/line ranges to child, parallel, working-copy, or remove-from-source destinations
- Revset + fileset alias resolution from jj config
- Bookmarks: list, create, move, delete, rename, track
- Git: push (with auto-track), fetch, remote URL
- GitHub: `gh pr view` parsing for PR link + checks; `gh_pr_open_url` resolves existing PR or builds compose URL with safe userinfo-stripping `github_slug` parser + URL-encoded bookmark
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
- Persistent review state (file-level + hunk-level), content-hashed so it survives rebases that produce identical bytes; auto-promotes to file-marked when every hunk is reviewed; "Hide reviewed files" filter in the file column
- Batch split with file review checkboxes (space key), parallel split option
- Commit box with AI message generation (Codex → Claude → Apple Intelligence)
- Bookmark picker with push, rename, track, move forward
- Revset filter, auto-refresh via FS watcher
- Native Form-based settings: Appearance, Diff, Tools (editor/terminal), Jujutsu config, About
- Font family picker (System/Menlo/SF Mono/JetBrains/Fira/Cascadia) + size stepper + ⌘+/-/0 zoom
- Onboarding wizard with jj check + GitHub Desktop warning
- jj git init button for non-jj folders
- Undo via jj op log (⌘⇧U)
- Command palette (⌘⇧P): search commands, `jj `/`!` prefix for jj CLI with inline output
- Pull Request on GitHub right-click action on bookmarks (DAG row + Bookmark Manager) — opens existing PR if one exists, else GitHub compose URL
- Change evolution viewer: list of past commit_ids per change with operation labels (snapshot/describe/rebase/squash/split), interdiff against current head, right-click to copy commit-id or `jj restore` command
- DetailPaneMode enum: collapses 5 mutually-exclusive `@State` vars (annotate / file history / diff edit / files) into one type-safe state
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
