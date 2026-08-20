# Review State Guide

Load this file before changing review marks, review notes, the review store, or the reconciliation pipeline. Crate boundaries live in [Architecture Guide](architecture.md).

Review state (marks and notes) is persistent across app restarts, local to the user, and shared by every surface through one store file.

## Store

- Canonical implementation: `jayjay_review::ReviewStore` (marks in `marks.rs`, notes in `note_store.rs`, reconciliation in `reconcile.rs`).
- Persistence and file format: [Storage Guide](storage.md).
- SwiftUI: `Shared/ReviewStore.swift` is an `@Observable` UniFFI facade with a per-file marks cache.
- GPUI: one process-global store; mutate only through `window/review.rs::mutate` (refresh from disk first). Render-path reads use `refresh_if_stale`. Note reconciliation loads asynchronously on the view model (`loaders/review_notes.rs`, generation-guarded).

## Marks

- Keying: `(change_id, path) -> { identity, file_marked, hunks }`. Always key by the real change id (`ChangeDetail.info.changeId`), never `selectionRevision` — that is a commit id for divergent working copies and hides state from CLI/core reconciliation.
- Review identity is computed in `jayjay_core::repo::diff::entry::compute_review_identity` from `MergedTreeValue` blob IDs, so reviews survive rebases/amends that do not change file bytes; any byte change invalidates only that file. Renamed files fold into a combined `rename|old|new` identity. The file-list identities come from `show_summary` (content-free rename pairing); anchors and reconciliation must use those, not `show`'s content-similarity pairing.
- Hunk indices are the canonical change groups from `jj_diff::change_groups` over `build_diff_display_lines` output. `is_hunk_reviewed(idx)` is true when the file is marked or the hunk set contains `idx`; marking every hunk promotes to file-level review, and unmarking a hunk drops the file flag. Committing clears only that change's marks (`clear_change`).

## Notes

Notes attach reviewer feedback to diff lines — an anchor of side, line, excerpt, context, and the `ignore_whitespace` mode the diff was rendered with — and are consumed by agents through the CLI:

- Reconciliation (`ReviewStore::reconcile`) runs against a `ReviewDiffProvider`. The only production provider is `jayjay-core`'s (`repo/review_notes.rs`), used by GUI and CLI alike — this is what keeps both surfaces agreeing on identity, rename pairing, and LFS/image placeholder content.
- Anchor matching replays the exact GUI render pipeline: Histogram `compute_file_diff` with the note's recorded `ignore_whitespace`, then display lines, then `change_group_for_anchor`. If you change any stage of the diff pipeline, note anchors are part of its contract.
- GUI anchors are built shell-side from the rendered line (side/line from the line's span style, excerpt/context from the display lines and change group) and stamped with the ignore-whitespace mode the diff was actually rendered with. `Repo::review_note_anchor`/`build_note_anchor` are the CLI add-note path only — they hardcode `ignore_whitespace = false`, so calling them from a GUI with the whitespace toggle on either errors or records a wrong-mode anchor that can reconcile stale in the very diff it was created from.

## Agent Workflow

Read notes before finalizing issue work:

```bash
jayjay review notes --repo .                  # plain text with bodies and anchor lines
jayjay review notes --repo . --format json    # structured pipeline output
jayjay review resolve-note <id> --repo .
jayjay review add-note --repo . --file <path> --line <n> [--side new|old] -m "note body"
```

Treat `current` notes as actionable, `stale` notes as needing re-check against the changed diff, and `orphaned` notes as comments whose original file or anchor disappeared. Resolve notes only after addressing the feedback; resolution is limited to IDs belonging to the repository's working-copy change.

Agents may leave notes anchored to changed lines. Prefer them over source-code comments for review intent, risks, and questions because notes never ship in the change. Adding a note to the same changed line updates that line's active note.
