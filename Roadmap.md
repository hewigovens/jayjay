# Roadmap

JayJay already covers the common jj history, diff, bookmark, conflict, Git, review, and command-palette flows. See [UserGuide.md](UserGuide.md) for shipped features. Next work should focus on making GPUI write flows solid, tightening jj-native editing, and keeping the surface small.

## Next

- [ ] GPUI write actions: AI commit messages, `jj new`, abandon, squash into parent, file-level split from review marks, bookmark create/move/push, and undo.
- [ ] GPUI rewrite and resolve flows: drag-to-rebase insert before/after, subtree movement previews, clearer descendant behavior, and basic `jj resolve` UI.
- [ ] Diff edit polish: unsupported-file messaging, and better topology copy.
- [ ] Stacked PR assistant: detect a jj change stack, assign/push per-change bookmarks, create GitHub PRs bottom-up with dependent bases, and review each layer with bookmark diff.
- [ ] Saved revsets: named revset library plus "save this revset".
- [ ] Evolog polish: inline restore, hide snapshots, and collapse snapshot runs.
- [ ] GPUI Linux/Windows polish: notifications, file picker fallback, installer UX, and Windows packaging.
- [ ] Semantic diff: tree-sitter AST diffing and function-level summaries.

## Known Issues

- Side-by-side for new/deleted files falls back to unified.

## Done

- Major issue milestones: conflicts #1, absorb/backout #2, annotate #3, interdiff #4, revset presets #5, diff edit #6, and GitHub PR creation #24.
- Core jj operations: open, log, show, describe, new, edit, squash, abandon, rebase, split, duplicate, merge, absorb, backout, undo/op log, workspace, Git, and bookmark flows.
- Diff and review: unified and side-by-side diff, word highlighting, images/SVG, diff edit, persistent review state, batch split, move to working copy, conflict resolve, annotate, file history, and evolog.
- Graph and navigation: DAG graph, revset presets and aliases, shift-click interdiff/bookmark diff, drag-to-rebase, keyboard navigation, find, and file/change reveal.
- Command palette and integrations: searchable actions, raw `jj`/`!` commands, command history, inline output, copy output, GitHub PR links/checks/opening, editor tools, and terminal tools.
- GPUI shell baseline: read parity, file history, filesystem watcher, onboarding, bookmark manager, review store, diff selection/copy, native appearance tracking, Lucide icons, Zed/GPUI bump, compact UI polish, and first write actions with describe plus commit box.
- GPUI Linux AppImage baseline: Nix AppImage build, desktop entry, AppStream metadata, hicolor icons, and CI artifact checks.
- SwiftUI macOS shell: multi-window repo management, settings, command box, AI commit messages, bookmark picker, undo, onboarding, CLI installer, URL scheme, and release pipeline.
- Safety and quality: friendly errors, crash audit fixes, shell injection hardening, CI, Rust tests, Swift tests, and GPUI component tests.
