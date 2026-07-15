# Roadmap

JayJay already covers the common jj history, diff, bookmark, conflict, Git, review, and command-palette flows in its SwiftUI and GPUI shells. See [UserGuide.md](UserGuide.md) for shipped features and [agents/shell-parity.md](agents/shell-parity.md) for the exact remaining shell gaps. Next work should focus on closing those gaps, tightening jj-native editing, and keeping the surface small.

## Next

- [ ] GPUI DAG rewrite actions: `jj edit`, squash into parent or a selected change, rebase, merge, duplicate, absorb, and revert/backout; also prefill the commit box from @'s description.
- [ ] GPUI bookmark-manager parity: rename, push/delete, PR/MR open, conflict resolution, remote selection and status, deleted-bookmark cleanup, and a post-drag Push affordance.
- [ ] GPUI rich-preview parity: render Markdown images and add an inline sandboxed HTML preview.
- [ ] Diff and graph polish: drag-to-rebase insert before/after, subtree movement previews, clearer descendant behavior, and better topology copy.
- [ ] GPUI settings and help parity: editable custom editor/terminal commands, font-size settings, and a keyboard-shortcut reference window.
- [ ] Saved revsets: named revset library plus "save this revset".
- [ ] Evolog polish: inline restore, hide snapshots, and collapse snapshot runs.
- [ ] GPUI Linux/Windows polish: notifications, file picker fallback, installer UX, and Windows packaging.
- [ ] Semantic diff: tree-sitter AST diffing and function-level summaries.

## Known Issues

- Side-by-side for new/deleted files falls back to unified.

## Done

- Major issue milestones: conflicts #1, absorb/backout #2, annotate #3, interdiff #4, revset presets #5, diff edit #6, and GitHub PR creation #24.
- Core jj operations: open, log, show, describe, new, edit, squash, abandon, rebase, split, duplicate, merge, absorb, backout, undo/op log, workspace, Git, and bookmark flows.
- Diff and review: unified and side-by-side diff, word highlighting, image/SVG and rich file previews, line-level discard, multi-file Diff Edit, persistent review marks and notes, batch split, move to working copy, conflict resolution, annotate, file history, and evolog.
- Graph and navigation: DAG graph, revset presets, aliases, and custom filtering, shift-click interdiff/bookmark diff, drag-to-rebase, keyboard navigation, find, and file/change reveal.
- Command palette and integrations: searchable actions and help topics, raw `jj`/`!` commands, command history, inline output, copy output, GitHub/GitLab/Codeberg PR links and checks, editor tools, and terminal tools.
- GPUI write flows: AI commit messages, `jj new`, abandon, describe/commit, file multi-select split and batch actions, bookmark create/move/track/push/delete, Git fetch/push, and operation-log restore/undo.
- GPUI shell baseline: history and core diff coverage, file history, filesystem watcher, onboarding, repository windows, workspaces, bookmark manager, review notes, conflict resolution, Diff Edit, stacked pull requests, revset filtering, native Markdown/SVG previews, searchable help, native appearance tracking, and compact UI polish.
- GPUI Linux AppImage baseline: Nix AppImage build, desktop entry, AppStream metadata, hicolor icons, CI artifact checks, published alpha artifacts, Linux CLI installation, and shared GitHub release notes.
- SwiftUI macOS shell: multi-window repo management, settings, command box, AI commit messages, bookmark picker, undo, onboarding, CLI installer, URL scheme, and release pipeline.
- Safety and quality: friendly errors, crash audit fixes, shell injection hardening, CI, Rust tests, Swift tests, and GPUI component tests.
