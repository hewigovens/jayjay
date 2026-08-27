# Shell Feature Parity Guide

Load this file when checking whether a user-visible workflow is missing in one product shell, or when refreshing the matrix during the [release](release.md) shipped-docs pass. During feature work, put shared behavior in Rust and implement the requested shell; do not edit this matrix in the feature PR. Also load [SwiftUI Shell Guide](swiftui.md) or [GPUI Shell Guide](gpui.md) before editing that shell's code.

JayJay has two product shells: the SwiftUI macOS app and the GPUI Linux shell. GPUI also builds on macOS for development, but that build is not a parity target and does not need to duplicate SwiftUI-only platform integrations. Unless a row says otherwise, the GPUI column describes Linux behavior; core jj workflows should remain familiar across both product shells.

Use this as a lightweight user-facing feature map sourced from [the public user guide](../docs/guide.html). Keep rows aligned with guide features, not internal modules.

## Status

- **Yes**: the shell supports the user-facing workflow.
- **Partial — _gap_**: the shell supports the core workflow, but the named guide affordances or edge cases are missing. Never use a bare **Partial** cell; summarize the gap in the cell and enumerate the exact missing behavior in Notes.
- **Gap**: the guide feature is not available in that shell yet.
- **N/A**: the feature is intentionally shell-specific or platform-specific.

## Feature Map

At release, update this matrix when the user guide adds a feature, a shell closes a gap, or a feature deliberately remains shell-specific.

| User Guide Feature | SwiftUI | GPUI (Linux) | Notes |
| --- | --- | --- | --- |
| Open a Repository | Yes | Yes | Pins share `repositories.json`; Recent is shell-local. Title menu activates an open window or opens a closed pin without duplicating. Both persist window frames and pane widths; SwiftUI's separate onboarding/list/repository scenes and Dock menu are macOS integrations. |
| Main Window | Yes | Yes | Same jj data in DAG, detail, file column, diff, status bar, markers, and workspace chips. |
| Navigate History | Yes | Yes | Selection, revset filter, load-more, context actions, drag/drop, and divergent changes stay aligned. |
| Review Diffs | Yes | Yes | Same text/image diffs, layout modes, find, file review, and tree/flat lists. Expansion: GPUI tab-focuses per-region controls; SwiftUI has Expand All Unmodified Lines. SwiftUI tints small reveals when Reduce Motion is off; GPUI swaps atomically. Both fall back to unified for purely added or deleted files. |
| Rich File Previews | Yes | Partial — Markdown images; inline HTML | Raw/processed modes and cache identity match. GPUI gaps: Markdown images are placeholders; no inline sandboxed HTML preview (external-open only). |
| Review Notes | Yes | Yes | Same add/edit/resolve/delete workflow. Inline rows unified-only; side-by-side uses a note-count banner plus Show in Unified in both shells. |
| Compare Changes | Yes | Yes | Shift-click, bookmark diff, reverse, clear, and interdiff use the same rev semantics. |
| Edit Diffs & Split Work | Yes | Yes | Same Diff Edit destinations, gutter discard, per-file cards/stats/collapse/keyboard focus, and working-copy text editor. Historical, compare, conflicted, removed, projected, and image files stay read-only. Both honor jj and Git custom difftool contracts. |
| Change Operations | Yes | Yes | Same change/file mutations and commit-box draft reseeding. Apple Intelligence is SwiftUI-only; GPUI uses codex/claude. |
| Bookmarks, Git & Pull Requests | Yes | Partial — Bookmark Manager rename; remote choice when tracking | GPUI covers create, DAG drag/move with the post-drag Push banner, tracking, push/delete, fetch/push, remote URLs, GitHub/GitLab/Codeberg/Cursor Origin PR/MR open, selected-bookmark CI, and the Bookmark Manager stats bar, Show deleted, remote sync badges, and row actions (resolve, track, push, PR/MR, delete, forget). Gaps: Bookmark Manager rename and choosing the remote when tracking. |
| Workspaces | Yes | Partial — picker refresh | Both create, open, show context, search and switch through a filterable title picker, group sibling checkouts beneath their primary repository, forget, and Forget & Delete after confirmation. Gap: GPUI's picker has no explicit workspace refresh. |
| Stacked Pull Requests | Yes | Yes | Same preview, name validation, and dependent-base submit. GitHub can link a native stack (`gh api`) with dependent-chain fallback; GitLab detects the MR stack; Cursor Origin uses `origin`. Done opens the top GitHub linked PR, the highest GitLab MR, and every PR for a GitHub fallback or Origin stack. |
| Conflict Resolution | Yes | Yes | Same conflict modal (hunk/raw, ours/theirs, Base, n-way markers) over the mounted detail. Both honor jj and Git mergetool contracts. |
| Inspection Tools | Yes | Yes | Annotate, file history, and evolog use the same revs, copy values, and compare targets. |
| Command Palette | Yes | Yes | Action names, raw jj, output handling, and searchable help topics stay aligned even if presentation differs. |
| Tools & Settings | Yes | Partial — custom command editing; font-size setting | Shared config ids. Both copy the Rust-owned jj tool definition; `jayjay config` prints it. Gaps: GPUI cannot edit custom editor/terminal commands; font size is zoom-only. GPUI Settings → Tools has codex/claude; Settings → CLI has jj/gh/glab/origin plus Linux `jayjay` install (AppImage-aware symlink in `~/.local/bin`). SwiftUI-only macOS integrations are outside the GPUI Linux parity target. |
| Help & User Guide Access | Yes | Yes | Same help content, Send Feedback, and Keyboard Shortcuts. Linux GPUI uses `xdg-open`. Palette topics come from shared `HelpFeatures.json`. Bundled Help Book is SwiftUI-only. |
| Keyboard Shortcuts | Yes | Yes | GPUI implements the published guide shortcuts. Both expose **Help → Keyboard Shortcuts** on `mod+/`; GPUI shows platform-correct keys (`Ctrl` on Linux). |
| GPUI Shell Alpha | N/A | Yes | The guide's GPUI alpha section is the source of truth. Refresh it and this matrix together at release. |

## Parity Rules

1. Put shared behavior in Rust core when possible; SwiftUI reaches it through UniFFI, and GPUI links it directly.
2. Share persisted config ids, command ids, projection identities, review identities, font/editor/terminal option vocabularies, and jj action semantics.
3. Preserve shell-native presentation. Matching behavior does not require matching exact layout, animation, or menu placement.
4. Treat GPUI macOS differences as development-build differences, not parity gaps, unless they expose a portable core-behavior defect.
5. If a product shell does not change, keep the reason visible in the matrix at release.
6. Validate at the smallest useful layer: Rust unit tests for shared behavior, Swift tests or XCUITests for Swift-only UI, and GPUI component tests for GPUI state and render behavior.

Do not add tests that only mirror constants or field wiring. A parity test should prove behavior, such as a shared config id round-tripping, a diff projection loading the same mode, or a UI action dispatching the same core mutation.
