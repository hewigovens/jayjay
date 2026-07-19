# Shell Feature Parity Guide

Load this file before adding, removing, or changing user-visible behavior in either shell, especially when the feature exists in SwiftUI but not GPUI, or vice versa. Also load [SwiftUI Shell Guide](swiftui.md) or [GPUI Shell Guide](gpui.md) before editing that shell's code.

JayJay has two shells over the same product: the SwiftUI macOS app and the GPUI cross-platform app. They do not need identical implementation details, but users should not have to relearn core jj workflows when switching shells.

Use this as a lightweight user-facing feature map sourced from [the public user guide](../docs/guide.html). Keep rows aligned with guide features, not internal modules.

## Status

- **Yes**: the shell supports the user-facing workflow.
- **Partial — _gap_**: the shell supports the core workflow, but the named guide affordances or edge cases are missing. Never use a bare **Partial** cell; summarize the gap in the cell and enumerate the exact missing behavior in Notes.
- **Gap**: the guide feature is not available in that shell yet.
- **N/A**: the feature is intentionally shell-specific or platform-specific.

## Feature Map

Update this matrix when the user guide adds a feature, a shell closes a gap, or a feature deliberately remains shell-specific.

| User Guide Feature | SwiftUI | GPUI | Notes |
| --- | --- | --- | --- |
| Open a Repository | Yes | Yes | Repository-list history stays shell-local, while pins share the Rust-backed `repositories.json`. Both shells keep Pinned above Recent, preserve pins when Recent is cleared, expose live windows plus closed pins from the repository title, activate an existing window without duplication, open closed pins in a new window, and return to the repository list after the last repo window closes. |
| Main Window | Yes | Yes | DAG, detail header, file column, diff pane, status bar, bookmark/tag/conflict markers, and working-copy state should describe the same jj data. |
| Navigate History | Yes | Yes | Selection, keyboard navigation, toolbar revset filtering (presets and custom expressions), load-more behavior, context actions, drag/drop outcomes, and divergent-change handling should stay aligned. |
| Review Diffs | Yes | Partial — added/deleted side-by-side; rich previews | Text diff, unified/side-by-side modes, find, image diff, file review, and flat/tree file lists are GPUI-covered. Exact gaps: added and deleted files fall back to unified instead of rendering side-by-side; rich-preview gaps are enumerated in the next row. |
| Rich File Previews | Yes | Partial — Markdown images; inline HTML | Raw/processed projection modes and cache identity match. GPUI has projection controls, banners, HTML external open, native SVG preview, and a rendered Markdown preview (native block renderer, single post-change document with scrolling — same single-view model as SwiftUI). Exact gaps: Markdown image blocks render as placeholders instead of actual images; GPUI has no inline sandboxed HTML preview toggle, only external-open. |
| Review Notes | Yes | Yes | GPUI supports add/edit/resolve/delete review notes, gutter dot markers, inline note rows, file-list badges, the noted-files filter, and a stale/orphaned banner. Inline note rendering is unified-view-only in both shells; side-by-side shows a note-count banner with "Show in Unified" in both, so that is not a GPUI gap. |
| Compare Changes | Yes | Yes | Shift-click compare, bookmark diff, reverse compare, clear compare, and interdiff loading should use the same rev semantics. |
| Edit Diffs & Split Work | Yes | Yes | Both shells cover line-granularity discard from the normal diff gutter and a dedicated multi-file Diff Edit view with per-file cards, line/hunk/file selection, select-all, keep-only-selected Done, Move to Working Copy, New Child, and New Parallel. |
| Change Operations | Yes | Partial — DAG rewrites; @ description prefill | GPUI covers new-change-on-top, abandon, operation-log restore, editing descriptions (including `jj describe` on @), committing, codex/claude AI messages, file multi-select split/commit and batch actions, and Diff Edit destinations including Move to Working Copy, New Child, and New Parallel. Exact gaps: no DAG actions for `jj edit`, squash into parent/selected, rebase selected, merge selected, duplicate, absorb, or revert/backout; the commit box also does not initially prefill from @'s existing description. |
| Bookmarks, Git & Pull Requests | Yes | Partial — bookmark-manager mutations; post-drag Push | GPUI covers bookmark creation, DAG drag/move, tracking, push/delete, Git fetch/push, remote URL opening, GitHub/GitLab/Codeberg PR/MR opening, and selected-bookmark PR/MR plus CI status. Exact gaps: the Bookmark Manager does not expose rename, push, delete, PR/MR open, conflict resolution, per-remote ahead/behind/diverged details, remote choice when tracking, or the hide-by-default **Show deleted** plus per-row **Forget** flow; dragging a tracked bookmark also does not show SwiftUI's one-click sidebar **Push** affordance. |
| Workspaces | Yes | Yes | Both shells create a workspace by name into a sibling directory, open it in its own window, show workspace context, and forget workspaces. GPUI entry points: Repository menu, status-bar workspace picker, and palette; switching lives in the status-bar picker where SwiftUI also offers palette "Switch to" entries — a presentation difference, not a workflow gap. |
| Stacked Pull Requests | Yes | Yes | Both shells preview the detected stack, validate edited bookmark names, and submit create/update operations with dependent bases. |
| Conflict Resolution | Yes | Yes | Conflicted changes/files, conflict diff styling, Use Ours/Theirs, resolve-in-editor, and refresh-after-resolution should stay behaviorally equivalent. |
| Inspection Tools | Yes | Yes | File Annotate, File History, and Change Evolution should use the same source revs, copy values, and compare targets. |
| Command Palette | Yes | Yes | Action names, raw jj behavior, command output handling, and searchable help topics should stay aligned even if presentation differs. |
| Tools & Settings | Yes | Partial — custom command editing; macOS integrations | Shared config ids and option vocabularies match. Exact gaps: GPUI displays but cannot edit custom editor/terminal command values in Settings, and font size is changed only through zoom commands rather than the Settings control; Apple Intelligence detection, the macOS Help Book, Sparkle updates, and macOS app-bundle CLI installation remain SwiftUI-only. GPUI does include codex/claude and jj/gh/glab detection plus a Linux `jayjay` installer (AppImage-aware symlink in `~/.local/bin`). |
| Help & User Guide Access | Yes | Yes | Both shells reach the same help content and expose Send Feedback from the Help menu and command palette. GPUI's Help menu opens the canonical public guide, and its palette lists the same searchable help topics (shared `HelpFeatures.json`) opening the guide at each topic's anchor. The bundled offline macOS Help Book stays SwiftUI-only (platform-specific packaging, same content). |
| Keyboard Shortcuts | Yes | Partial — shortcut reference window | GPUI implements every shortcut published in the user guide: palette, find, refresh, open repository, zoom, Bookmark Manager, operation log/undo, file review, shift-click compare, `j`/`k`, and `Ctrl+N`/`Ctrl+P`. The primary modifier is `Cmd` on macOS and `Ctrl` elsewhere. Exact gap: GPUI does not provide SwiftUI's **Help → Keyboard Shortcuts** reference window. |
| GPUI Shell Alpha | N/A | Yes | The guide's GPUI alpha section is the source of truth for current GPUI coverage claims. Update it and this matrix together. |

## Parity Rules

1. Put shared behavior in Rust core when possible; SwiftUI reaches it through UniFFI, and GPUI links it directly.
2. Share persisted config ids, command ids, projection identities, review identities, font/editor/terminal option vocabularies, and jj action semantics.
3. Preserve shell-native presentation. Matching behavior does not require matching exact layout, animation, or menu placement.
4. If a shell does not change, keep the reason visible in the matrix or in the PR notes.
5. Validate at the smallest useful layer: Rust unit tests for shared behavior, Swift tests or XCUITests for Swift-only UI, and GPUI component tests for GPUI state and render behavior.

Do not add tests that only mirror constants or field wiring. A parity test should prove behavior, such as a shared config id round-tripping, a diff projection loading the same mode, or a UI action dispatching the same core mutation.
