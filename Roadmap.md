# Roadmap

JayJay now covers most common jj history, diff, bookmark, conflict, and Git flows, plus raw `jj` execution via `!` in the command palette. The next phase is less about basic command parity and more about making jj-native workflows feel faster and more visual than the CLI.

## Longer-term

- [ ] GPUI shell feature parity — Linux + Windows native shell using GPUI (one Rust shell, identical look across both). Mac stays on SwiftUI until the GPUI shell proves parity. OS integration via freedesktop standards (`.desktop`, hicolor, D-Bus via `notify-rust` / `ashpd` / `zbus`) — no GTK dependency.
  - Definition of parity: every user-visible SwiftUI feature is either implemented in GPUI, deliberately marked macOS-only, or explicitly cut from both shells. Every shipped GPUI feature has a hermetic `#[gpui::test]` component test or a stronger end-to-end check.
  - [ ] Read/navigation parity:
    - [ ] Revset filter UI with editable expression, preset chips (All, Mine, Bookmarks, Trunk, Conflicts, Heads), reset-to-default, load-more semantics, and clear empty/error states.
    - [ ] Saved revsets library and command palette revset commands, matching the completed SwiftUI workflow.
    - [ ] Command palette parity: refresh, view toggles, revset presets, Git actions, bookmark manager, change actions, workspace actions, zoom, Show in Finder / file manager, View Remote Repository, editor/terminal, undo, settings, and raw `jj` command history/output polish.
    - [ ] Interdiff / compare mode: shift-click and context-menu compare, compare banner, clear compare, and arbitrary `from`/`to` file diffs.
    - [ ] DAG context menu parity: new, edit/switch, compare, rebase selected onto target, squash selected into target, merge with selected, create bookmark, evolog, graft, duplicate, absorb, revert, abandon, plus divergent-change wording.
    - [ ] Bookmark chip/menu parity in DAG rows: move to `@-`, push, pull request on GitHub, and copy bookmark name.
    - [ ] Status bar parity: conflict-count action, selected-bookmark PR link/check state, change count, repo path, workspace open/forget/delete actions.
    - [ ] File column parity: search/filter, hide reviewed files, split reviewed files shortcut, conflict badges, multi-select and shift-select, batch context actions, hidden Git LFS/submodule counts, and settings-backed filtering.
    - [ ] Non-jj folder onboarding action that actually runs `jj git init`, not only a hint.
  - [ ] Diff/review parity:
    - [ ] Hunk-level review checkboxes, reviewed-hunk persistence, file auto-promotion when all hunks are reviewed, and materialized survivors when unmarking a file-marked hunk.
    - [ ] Diff edit mode: change-wide select all / clear all, file/hunk/line selection, new-child destination, parallel destination, move-to-working-copy destination, topology-aware destination copy, and unsupported-file messaging.
    - [ ] Inline line actions: abandon selected working-copy lines and open selected lines in diff edit.
    - [ ] Conflict UI: conflicted-path loading, per-file conflict bar, Use Ours, Use Theirs, resolve with configured merge tool, and post-action refresh/toast.
    - [ ] Image/SVG parity with SwiftUI: rendered SVG toggle where supported, clear unsupported placeholders, and consistent new/deleted/modified image layouts.
    - [ ] Diff selection polish remaining from the baseline: cross-hunk-gap selection through `…N hidden lines…` separators, triple-click line-select, Cmd+A select-all.
  - [ ] Write/action parity:
    - [ ] Shared GPUI mutation path around `RepoViewModel::perform`-style helpers: detached work, friendly errors, success messages, refresh selection, review-store cleanup, and FS-watcher loop suppression.
    - [ ] Presentation primitives matching SwiftUI: inline empty/error states, blocking HUD only for unsafe operations, toast with optional undo action, alerts, and sheets.
    - [ ] Describe/edit message for any change and working-copy commit box with AI message generation using `jayjay_core::COMMIT_MESSAGE_PROMPT`.
    - [ ] Commit working copy, including submodule-attention flow and safe submodule update commit.
    - [ ] `jj new`, edit/switch, abandon with confirmation, squash into parent, squash into selected target, rebase, merge, duplicate, graft, absorb, and revert.
    - [ ] File actions: split, parallel split, move to working copy, restore to parent, delete from disk, ignore & untrack, and batch actions over multi-selection.
    - [ ] Drag-to-rebase with arm delay, hover preview, confirmation sheet, Return/Escape handling, undo toast, and conflict follow-up.
    - [ ] Undo via `jj op log` (`Cmd/Ctrl+Shift+U`) with restore action and operation labels.
  - [ ] Bookmark / Git / GitHub parity:
    - [ ] Bookmark create, rename, delete, track remote, move forward, filter by bookmark, and full Bookmark Manager.
    - [ ] Git fetch/pull, push all, push selected bookmark, auto-track push result handling, and clean up stale bookmarks.
    - [ ] Open existing PR or GitHub compose URL from bookmarks; keep the status-bar PR/checks surface in sync.
    - [ ] View Remote Repository action with `git@` to `https` conversion.
  - [ ] Workspace / window / app-shell parity:
    - [ ] New workspace, open existing workspace, forget workspace, forget-and-delete workspace, and recent repo list updates.
    - [ ] Multi-window repo management, URL scheme / CLI handoff to running instance, repo-window deduplication, and active-repo command routing.
    - [ ] Onboarding wizard with jj environment check, recent repositories, GitHub Desktop warning, and first-run state.
    - [ ] Help menu/actions: GitHub, jj docs, report issue, sponsor prompt policy if GPUI ships as a primary shell.
  - [ ] Settings / platform parity:
    - [ ] Appearance settings are fully interactive: system appearance detection, font family picker, font size controls, and Cmd/Ctrl +/-/0 zoom shortcuts.
    - [ ] Diff/settings values are wired through all views, especially Git LFS hiding, submodule hiding/support, abandon confirmation, and drag-rebase confirmation.
    - [ ] Tools settings can edit custom editor/terminal commands in-app, and Open in Editor / Terminal works cross-platform.
    - [ ] Environment status view for `jj`, `gh`, Codex CLI, Claude CLI, and platform AI providers.
    - [ ] Document or replace macOS-only SwiftUI surfaces: Sparkle updates, notarized release packaging, AppKit menus, Finder integration, and bundled CLI symlink installer.
    - [ ] GPUI package/install story: app icon set, `.desktop` entry, Windows metadata, D-Bus/toast notifications for long-running ops, file picker fallback, and persisted window placement on all targets.
  - [ ] Test parity:
    - [ ] Expand from the current single GPUI smoke test to component tests for selection, refresh, revset, diff loading, file review, command palette, settings persistence, and every mutation action.
    - [ ] Add deterministic fixtures mirroring SwiftUI UI scenes: file diff, annotate, interdiff, command palette, new change, review split, undo, bookmark manager, and conflict resolution.
    - [ ] Parity is not done until each SwiftUI UI test scene has a GPUI equivalent or a documented macOS-only exemption.
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

### Recent Workflow Polish
- [x] Stack surgery polish (`jj rebase --after` / `--before` and related flows)
  Drag-to-rebase now supports onto, insert-after, and insert-before zones with clearer previews, confirmation copy that calls out descendant movement, context-menu insert actions, and core bindings for `jj rebase -r ... --insert-after/--insert-before`
- [x] Diff edit polish
  Added change-wide select all / clear all, stronger unsupported-file messaging, and topology copy that explains what each destination does to the source, child, sibling, or working copy
- [x] Saved revsets library
  Added built-in named revsets for common jj queries plus user-saved revsets with save/delete UI and command-palette commands
- [x] Command palette polish
  Added raw jj command history, richer inline output with exit status and copy action, parsing for quoted arguments, suggestions, and clearer discoverability for `jj ...` / `! ...`
- [x] Evolog polish
  Added inline restore to `@`, hide-snapshots toggle, and run-of-snapshots collapsing

### GPUI Shell
- [x] Read-only baseline:
  - [x] DAG, lane rendering, keyboard navigation, detail header/description, stats, avatars, unified + side-by-side diffs, image previews, annotate, file history, evolog, PR status fetch, bookmark picker, workspace picker, settings window, command palette basics, diff find, diff text selection/copy, column resizing, description resizing, auto-refresh, and no-repo placeholder.
  - [x] Persistent working-copy file review (space to toggle, `n / total reviewed`, content-hash auto-invalidation, survives jj rebases that produce identical content) — `jayjay_core::review::ReviewStore` is the canonical impl; SwiftUI's UserDefaults-backed copy is the next migration target.
  - [x] Diff view selection + copy — column-precision cross-line text selection in unified and side-by-side diffs; Cmd+C copies the joined slice; double-click selects a word; gutter is excluded structurally.

### Major Milestones
- [x] Absorb + Revert support (`jj absorb` / `jj revert`) — [#2](https://github.com/hewigovens/jayjay/issues/2)
- [x] Interdiff between arbitrary revisions (`jj diff --from X --to Y`) — [#4](https://github.com/hewigovens/jayjay/issues/4)
- [x] Conflict resolution UI (`jj resolve`) — [#1](https://github.com/hewigovens/jayjay/issues/1)
- [x] File annotate / blame view (`jj file annotate`) — [#3](https://github.com/hewigovens/jayjay/issues/3)
- [x] Graph revset filtering presets — [#5](https://github.com/hewigovens/jayjay/issues/5)
- [x] Change-wide diff edit mode (`jj diffedit`) — [#6](https://github.com/hewigovens/jayjay/issues/6)
- [x] Pull Request creation from bookmark right-click (DAG row + Bookmark Manager) — [#24](https://github.com/hewigovens/jayjay/pull/24)
- [x] Change evolution viewer (`jj evolog`) with interdiff against current + Copy `jj restore` command
- [x] Landing page (GitHub Pages)

### Rust Core
- jj-lib: open, log, log_graph, show, describe, new, edit, squash, squash --into, abandon, rebase, split, graft, duplicate, merge, absorb, revert
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
