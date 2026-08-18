# User Guide

This guide covers JayJay's user-facing features. The released macOS app uses the SwiftUI shell. The GPUI shell is an alpha source-build shell that shares the same Rust core and is covered near the end.

## Open a Repository

- Open a repository with `Cmd+O`, the app menu, the Dock recent-repositories menu, or the CLI launcher: `jayjay /path/to/repo`.
- Open the current terminal directory with `jayjay .` after installing the bundled CLI launcher.
- If you open a folder that is not a jj repository, JayJay shows an onboarding view with a `jj git init` path.
- JayJay watches the repository and working tree, then refreshes when jj operations or file edits change the repo.

## Main Window

- The left graph shows jj changes as a DAG with lanes for forks, merges, bookmarks, tags, conflicts, divergent changes, and working-copy state. Each row shows bookmark and tag chips, name@ chips for other workspaces’ working copies, the author avatar, a relative timestamp, and the shortest unique change-id prefix highlighted.
- The detail header shows the selected change, description, author, status, bookmarks, PR state, and available actions. The change-id and commit-id are shown with their shortest unique prefix in bold.
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
- Drag a change row to preview and confirm a rebase from the graph.
- Drag a bookmark chip onto another change to move the bookmark there, or drag the working-copy `@` chip to move the working copy (`jj edit`). Dropping on the same change is a no-op; backward moves are allowed and are undoable from the operation log. Press Return to confirm or Esc to cancel mid-drag.
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

## Review Notes

- Right-click a changed line's gutter in the working-copy diff and choose Add Review Note to leave line-anchored feedback. The editor shows a short diff excerpt around the anchored line; save with `Cmd+Return`.
- Saved notes render inline in the unified diff as an orange-outlined bubble below the annotated line, with an orange dot in the gutter's note column. Click the dot — or right-click the line again — to edit, resolve, or delete the note.
- Files with active notes show an orange note count in the file list, and the header note badge filters the list to noted files.
- Notes anchor to the line's content. If the file changes underneath, the note turns stale and moves to a banner above the diff, together with orphaned notes whose anchor disappeared. Resolving a note keeps a dimmed gutter dot as a record.
- Notes are local to your machine, shared across app windows and the `jayjay` CLI, and survive rebases of the change. They render in unified view; side-by-side shows the file's note count with a one-click Show in Unified.
- Agents can read notes with `jayjay review notes --repo . --format json` and resolve them with `jayjay review resolve-note <id> --repo .` once the feedback is addressed.
- Agents can annotate too: `jayjay review add-note --repo . --file <path> --line <n> -m "note"` anchors a note to a changed line, and it appears in the diff view like any other note — a cleaner channel for review commentary than source-code comments.

## Compare Changes

- Shift-click two graph revisions to compare them.
- When both revisions have bookmarks, JayJay uses bookmark names in the compare banner.
- Use the compare direction control to switch the diff direction.
- Bookmark diff is useful for PR-style review: compare the main bookmark or fork point against a feature bookmark.
- Interdiff mode uses the same unified and side-by-side diff renderers, but hides working-copy review controls because the comparison is not a file-review session.

## Edit Diffs and Split Work

- Diff edit mode lets you select files, hunks, or line ranges from the working copy or another mutable change.
- Selected edits can become a child change, a parallel change, or be moved into the working copy.
- Working-copy edits can be discarded at selected line granularity.
- Batch split can use reviewed files as the selection model.
- Split supports a parallel option when the selected edits should become a sibling instead of a child.
- Topology-aware destinations preserve the intended jj graph shape when moving edits.

## Change Operations

- Edit a change description directly from JayJay.
- Use the commit box to describe and commit the working copy.
- Generate commit messages with the AI provider chain: Codex CLI, Claude CLI, then Apple Intelligence when available.
- Create new changes, edit an existing change, squash into a parent, abandon, duplicate, merge, absorb into ancestors, and back out changes.
- Restore, ignore, or untrack working-copy files from file actions where applicable.
- Move selected files from any change into the working copy.
- Use Undo to inspect the jj operation log and roll back recent operations.
- JayJay shows lightweight toasts for completed actions and keeps the rest of the window usable when possible.

## Bookmarks, Git, and Pull Requests

- Use the Bookmark Manager with `Cmd+Shift+B` to inspect bookmark stats, filter bookmarks, reveal their changes, copy names, diff them, resolve conflicts, and clean up stale entries.
- Use bookmark actions to create, rename, track, move forward, delete, and push bookmarks, or drag a bookmark chip in the DAG to move it onto any change.
- After moving a remote-tracking bookmark by drag, a one-click **Push** affordance appears in the sidebar so you can publish the move (it never pushes automatically).
- Push and fetch Git remotes from JayJay; push can auto-track a bookmark when needed.
- Right-click a bookmark in the DAG or Bookmark Manager to open a GitHub, GitLab, or Codeberg pull/merge request.
- If a matching PR or MR already exists, JayJay opens it. Otherwise it opens the provider's PR/MR compose page for that bookmark.
- The status bar can show the selected bookmark's PR/MR link and CI check status via the GitHub `gh` CLI, the GitLab REST API, or Codeberg's Forgejo API. Private GitLab projects use a `GITLAB_TOKEN` environment variable.
- Remote repository URLs can be opened in the browser, including `git@...` URLs converted to HTTPS.

## Stacked Pull Requests

Turn a linear stack of changes into one PR (GitHub) or MR (GitLab) per change, each targeting the one below it.

- Right-click the **tip** change in the DAG and choose **Create / Update Stacked PRs**. Whatever change you click becomes the top of the stack; everything from just above `trunk()` up to it is included.
- The preview shows one row per change — bottom-first targeting your default branch (`main`), each higher one targeting the bookmark below it. Each row's **branch name is editable** (pencil → edit → Done); when Apple Intelligence is available, **Generate bookmarks** suggests names from the commit messages. Existing bookmarks are reused unchanged.
- **Submit** pushes every bookmark at once, then creates or updates the PRs/MRs with their dependent bases. **Done** opens the top PR for a linked GitHub stack, always opens the highest submitted GitLab MR, and opens each submitted PR if GitHub native linking falls back. Re-running is idempotent — bookmarks anchor on the change-id, so it updates the same PRs/MRs and stack instead of duplicating.
- **GitHub native stacks:** JayJay uses the standard `gh api` command; no extension is required. If GitHub Stacked PRs is not enabled for the repository or rejects the chain, the dependent PRs remain usable and the result explains that native linking was skipped.
- **GitLab stacks:** GitLab detects the dependent MR chain automatically and shows a stack navigator in each MR; no separate linking request is required.
- **Merging:** for ordinary GitHub PR chains and GitLab MRs, merge **bottom-up** (the one targeting `main` first). After each merge, run `jj git fetch` and **Create / Update Stacked PRs** again to retarget the remaining layers. If GitHub has linked the PRs into a native stack, use GitHub's stack controls; merging a PR also merges every unmerged layer below it, then GitHub rebases and retargets the remainder.
- JayJay requires an authenticated `gh` CLI (GitHub) or `glab` CLI (GitLab). The forge is taken from the repo's `origin` remote; Codeberg is not yet supported.

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

- Configure appearance, diff behavior, editor, terminal, CLI detection, jj settings, and app metadata in Settings.
- The update channel dropdown in Settings → About switches between Stable and Beta; the Beta channel receives pre-release builds through the regular update check.
- Anonymous build and OS statistics are enabled by default and can be disabled in Settings. JayJay sends no repository, file, or command data, and rotating identifiers cannot link an installation across months.
- The Tools tab configures editor, terminal, and AI commit-message providers (Codex, Claude, and Apple Intelligence).
- The CLI tab groups version-control tools (`jayjay`, `jj`) and forge CLIs (`gh`, `glab`).
- Pick a font family and adjust zoom with `Cmd++`, `Cmd+-`, and `Cmd+0`.
- Open files in external editors such as VS Code, VSCodium, Cursor, Zed, Xcode, or Vim. Cursor launches with `--classic` so it opens in editor mode rather than its agent window.
- Open terminals such as Terminal.app, iTerm2, or Ghostty at the repository path.
- Commit avatars can come from GitHub or Gravatar.
- Multi-window mode keeps one window per repository and deduplicates URL-scheme launches.
- Choose **Help -> Send Feedback** to email us.

## GPUI Shell Alpha

- Build and run it from source with `just gpui` or `just gpui /path/to/repo`.
- GPUI's current parity target is Linux. Its macOS build is for development and is not expected to duplicate every SwiftUI integration; the released macOS app remains SwiftUI.
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
