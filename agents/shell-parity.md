# Shell Feature Parity Guide

Load this file before adding, removing, or changing user-visible behavior in either shell, especially when the feature exists in SwiftUI but not GPUI, or vice versa. Also load [SwiftUI Shell Guide](swiftui.md) or [GPUI Shell Guide](gpui.md) before editing that shell's code.

JayJay has two shells over the same product: the SwiftUI macOS app and the GPUI cross-platform app. They do not need identical implementation details, but users should not have to relearn core jj workflows when switching shells.

Use this as a lightweight user-facing feature map sourced from [the public user guide](../docs/guide.html). Keep rows aligned with guide features, not internal modules.

## Status

- **Yes**: the shell supports the user-facing workflow.
- **Partial**: the shell supports the core workflow, but important guide affordances or edge cases are missing.
- **Gap**: the guide feature is not available in that shell yet.
- **N/A**: the feature is intentionally shell-specific or platform-specific.

## Feature Map

Update this matrix when the user guide adds a feature, a shell closes a gap, or a feature deliberately remains shell-specific.

| User Guide Feature | SwiftUI | GPUI | Notes |
| --- | --- | --- | --- |
| Open a Repository | Yes | Yes | Refresh, repository watching, current selection, and non-repo onboarding should preserve the same user outcome. Shell-specific launch surfaces are fine. |
| Main Window | Yes | Yes | DAG, detail header, file column, diff pane, status bar, bookmark/tag/conflict markers, and working-copy state should describe the same jj data. |
| Navigate History | Yes | Yes | Selection, keyboard navigation, revsets, load-more behavior, context actions, drag/drop outcomes, and divergent-change handling should stay aligned where implemented. |
| Review Diffs | Yes | Partial | Text diff, unified/side-by-side modes, find, image diff, file review, and flat/tree file lists are GPUI-covered. Rich rendered previews remain partial. |
| Rich File Previews | Yes | Partial | Raw/processed projection modes and cache identity must match. GPUI has projection controls, banners, HTML external open, and native SVG preview, but not full Markdown rendered preview parity. |
| Review Notes | Yes | Gap | GPUI shares review storage semantics for file marks, but does not yet provide the line-note authoring and inline rendering workflow from the guide. |
| Compare Changes | Yes | Yes | Shift-click compare, bookmark diff, reverse compare, clear compare, and interdiff loading should use the same rev semantics. |
| Edit Diffs & Split Work | Yes | Gap | SwiftUI-only until GPUI diff edit is ported. Do not let edit-mode assumptions leak into shared diff rendering. |
| Change Operations | Yes | Partial | GPUI covers editing descriptions and committing the working copy. Other change/file mutations need explicit parity checks before being documented as covered. |
| Bookmarks, Git & Pull Requests | Yes | Partial | GPUI has bookmark manager and PR-open entry points. Bookmark sync details, provider status, and full PR workflows should be checked feature by feature. |
| Workspaces | Yes | Partial | GPUI shows workspace context and can forget workspaces; new workspace creation and full workspace management are not complete parity. |
| Stacked Pull Requests | Yes | Gap | SwiftUI-only until GPUI gets the stacked PR preview, branch-name editing, push, and create/update flow. |
| Conflict Resolution | Yes | Yes | Conflicted changes/files, conflict diff styling, Use Ours/Theirs, resolve-in-editor, and refresh-after-resolution should stay behaviorally equivalent. |
| Inspection Tools | Yes | Yes | File Annotate, File History, and Change Evolution should use the same source revs, copy values, and compare targets. |
| Command Palette | Yes | Yes | Action names, raw jj behavior, command output handling, and searchable help topics should stay aligned even if presentation differs. |
| Tools & Settings | Yes | Partial | Shared config ids and option vocabularies must match. GPUI settings are cross-platform and should not copy macOS-only Help Book, Sparkle, or CLI-installer plumbing. |
| Help & User Guide Access | Yes | Partial | SwiftUI owns the bundled macOS Help Book. GPUI should link to the same public guide when it exposes help. |
| Keyboard Shortcuts | Yes | Partial | Shared workflow shortcuts should match where platform conventions allow; shell-specific shortcuts should remain discoverable in menus or the palette. |
| GPUI Shell Alpha | N/A | Yes | The guide's GPUI alpha section is the source of truth for current GPUI coverage claims. Update it and this matrix together. |

## Parity Rules

1. Put shared behavior in Rust core when possible; SwiftUI reaches it through UniFFI, and GPUI links it directly.
2. Share persisted config ids, command ids, projection identities, review identities, font/editor/terminal option vocabularies, and jj action semantics.
3. Preserve shell-native presentation. Matching behavior does not require matching exact layout, animation, or menu placement.
4. If a shell does not change, keep the reason visible in the matrix or in the PR notes.
5. Validate at the smallest useful layer: Rust unit tests for shared behavior, Swift tests or XCUITests for Swift-only UI, and GPUI component tests for GPUI state and render behavior.

Do not add tests that only mirror constants or field wiring. A parity test should prove behavior, such as a shared config id round-tripping, a diff projection loading the same mode, or a UI action dispatching the same core mutation.
