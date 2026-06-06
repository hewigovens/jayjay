# User Guide

This guide covers JayJay's user-facing features. The released macOS app uses the SwiftUI shell. The GPUI shell is an alpha source-build shell that shares the same Rust core and is covered near the end.

## Open a Repository

- Open a repository with `Cmd+O`, the app menu, the Dock recent-repositories menu, or the CLI launcher: `jayjay /path/to/repo`.
- Open the current terminal directory with `jayjay .` after installing the bundled CLI launcher.
- If you open a folder that is not a jj repository, JayJay shows an onboarding view with a `jj git init` path.
- JayJay watches the repository and working tree, then refreshes when jj operations or file edits change the repo.

## Main Window

- The left graph shows jj changes as a DAG with lanes for forks, merges, bookmarks, conflicts, divergent changes, and working-copy state.
- The detail header shows the selected change, description, author, status, bookmarks, PR state, and available actions.
- The file column lists changed files in flat or tree form and shows review status, conflicts, renames, and file-level actions.
- The diff pane shows the selected file with unified or side-by-side layout, syntax highlighting, word-level changes, and collapsed context.
- The status bar surfaces repository state, selected bookmark PR links/checks, and useful workspace context.

## Navigate History

- Click a change to inspect it.
- Use `j`/`k`, arrow keys, or `Ctrl+N`/`Ctrl+P` to move through the graph and file lists.
- Use Load More to fetch older history without expanding the whole repo at once.
- Use revset chips for common views: all changes, mine, bookmarks, trunk, conflicts, and heads.
- Type a custom revset when the presets are too broad.
- Custom revsets can use aliases from your jj config.
- Use the context menu to reveal related changes, open file history, start comparisons, or run change operations.
- Use drag-to-rebase to preview and confirm a rebase from the graph.
- Divergent changes are marked in the graph so they are visible before you pick a resolution.

## Review Diffs

- Toggle between unified and side-by-side diffs.
- Use `Cmd+F` to search within the current diff.
- Use `Space` to mark the selected file reviewed.
- Hide reviewed files to focus on the remaining work.
- Review state is local to your machine and survives app restarts.
- Review marks invalidate when the file's old or new content changes, but survive rebases that keep the same bytes.
- Image files render as images where possible. SVG files can be viewed as source or rendered output.
- Renames, collapsed context, and ignore-whitespace behavior are reflected in the diff view.
- Copying diff text excludes gutter line numbers.

## Compare Changes

- Shift-click two graph revisions to compare them.
- When both revisions have bookmarks, JayJay uses bookmark names in the compare banner.
- Use the compare direction control to switch the diff direction.
- Bookmark diff is useful for PR-style review: compare the main bookmark or fork point against a feature bookmark.
- Interdiff mode uses the same unified and side-by-side diff renderers, but hides working-copy review controls because the comparison is not a file-review session.

## Edit Diffs and Split Work

- Diff edit mode lets you select files, hunks, or line ranges from a change.
- Selected edits can become a child change, a parallel change, or be moved into the working copy.
- Working-copy edits can be discarded at selected line granularity.
- Batch split can use reviewed files as the selection model.
- Split supports a parallel option when the selected edits should become a sibling instead of a child.
- Topology-aware destinations preserve the intended jj graph shape when moving edits.

## Change Operations

- Edit a change description directly from JayJay.
- Use the commit box to describe and commit the working copy.
- Generate commit messages with the AI provider chain: Codex CLI, Claude CLI, then Apple Intelligence when available.
- Create new changes, edit an existing change, squash into a parent, abandon, duplicate, graft, merge, absorb into ancestors, and back out changes.
- Restore, ignore, or untrack working-copy files from file actions where applicable.
- Move selected files from any change into the working copy.
- Use Undo to inspect the jj operation log and roll back recent operations.
- JayJay shows lightweight toasts for completed actions and keeps the rest of the window usable when possible.

## Bookmarks, Git, and Pull Requests

- Use the Bookmark Manager with `Cmd+Shift+B` to inspect bookmark stats, filter bookmarks, reveal their changes, copy names, diff them, resolve conflicts, and clean up stale entries.
- Use bookmark actions to create, rename, track, move forward, delete, and push bookmarks.
- Push and fetch Git remotes from JayJay; push can auto-track a bookmark when needed.
- Right-click a bookmark in the DAG or Bookmark Manager to open a GitHub or Codeberg pull request.
- If a GitHub PR or public Codeberg PR already exists, JayJay opens it. Otherwise it opens a GitHub or Codeberg PR compose page for that bookmark.
- The status bar can show the selected bookmark's PR link and check status via `gh` for GitHub or Codeberg's public Forgejo API.
- Remote repository URLs can be opened in the browser, including `git@...` URLs converted to HTTPS.

## Conflict Resolution

- Conflicted changes and files are marked in the graph and file list.
- The conflict bar offers one-click Use Ours and Use Theirs actions when the file can be resolved that way.
- Resolve in Editor opens `jj resolve --tool` with your configured merge editor, such as VS Code or Zed.
- JayJay refreshes after resolution so the graph and file list reflect the new repo state.

## Inspection Tools

- File Annotate shows blame information with a syntax-highlighted gutter and lets you navigate to the responsible change.
- File History lists revisions that modified the selected file.
- Change Evolution shows prior versions of a rewritten change with operation labels such as snapshot, describe, rebase, squash, and split.
- Evolution entries can be compared against the current version.
- Right-click an evolution entry to copy its commit id or a `jj restore` recovery command.

## Command Palette

- Open the command palette with `Cmd+Shift+P`.
- Search built-in actions by name.
- Type `jj <args>` or `! <args>` to run raw jj commands inline.
- Raw command output appears inside the palette and can be copied.
- Command history is available during the session, so repeated jj commands are easy to recall.

## Tools and Settings

- Configure appearance, diff behavior, editor, terminal, jj settings, and app metadata in Settings.
- JayJay checks for jj availability and detects supported AI providers.
- Pick a font family and adjust zoom with `Cmd++`, `Cmd+-`, and `Cmd+0`.
- Open files in external editors such as VS Code, Zed, Xcode, Android Studio, or Vim.
- Open terminals such as Terminal.app, iTerm2, or Ghostty at the repository path.
- Commit avatars can come from GitHub or Gravatar.
- Multi-window mode keeps one window per repository and deduplicates URL-scheme launches.
- Help menu links open JayJay, jj documentation, and issue reporting.

## GPUI Shell Alpha

- Build and run it from source with `just gpui` or `just gpui /path/to/repo`.
- GPUI targets a shared native shell for macOS, Linux, and Windows while the released macOS app remains SwiftUI.
- Current GPUI coverage includes graph browsing, diffs, file history, annotate, evolog, file review, bookmark manager, filesystem refresh, command palette, raw jj commands, native appearance tracking, and diff text selection/copy.
- Early write coverage includes editing descriptions and committing from the commit box.
- Remaining GPUI work is tracked in [Roadmap.md](Roadmap.md).

## Keyboard Shortcuts

| Key | Action |
| --- | --- |
| `Cmd+Shift+P` | Command palette |
| `Cmd+F` | Find in diff |
| `Cmd+R` | Refresh |
| `Cmd+O` | Open repository |
| `Cmd++` / `Cmd+-` / `Cmd+0` | Zoom in, zoom out, reset zoom |
| `Cmd+Shift+B` | Bookmark Manager |
| `Cmd+Shift+U` | Undo from jj operation log |
| `Space` | Toggle selected file reviewed |
| `Shift+Click` | Compare two revisions |
| `j` / `k` | Move through graph rows |
| `Ctrl+N` / `Ctrl+P` | Move to next or previous item |
