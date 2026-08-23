# Review State Guide

Load this file before changing review marks, review notes, the review store, or the reconciliation pipeline. Crate boundaries live in [Architecture Guide](architecture.md).

Review state (marks and notes) is persistent across app restarts, local to the user, and shared by every surface through one store file.

## Store

- Canonical implementation: `jayjay_review::ReviewStore` (marks in `marks.rs`, notes in `note_store.rs`, reconciliation in `reconcile.rs`).
- Persistence and file format: [Storage Guide](storage.md).
- SwiftUI: `Shared/ReviewStore.swift` is an `@Observable` UniFFI facade with a per-file marks cache.
- GPUI: one process-global store; mutate only through `window/review.rs::mutate` (refresh from disk first). Render-path reads use `refresh_if_stale`. Note reconciliation loads asynchronously on the view model (`loaders/review_notes.rs`, generation-guarded).

## Marks

- Keying: `(change_id, path) -> { identity, file_marked, hunks }` plus a sibling `review_baselines` map of per-group fingerprints. Always key by the real change id (`ChangeDetail.info.changeId`), never `selectionRevision` — that is a commit id for divergent working copies and hides state from CLI/core reconciliation.
- Review identity is computed in `jayjay_core::repo::diff::entry::compute_review_identity` from `MergedTreeValue` blob IDs. It is a fast-path detector (same bytes after a rebase keep the stored group states without rematerializing the file) but not the final review-state identity. Group fingerprints live in `jj_diff::canonical_review_snapshot`: exact-whitespace payloads plus a bounded unchanged context window, with no line numbers, wrapping, highlighting, or collapse. Display ignore-whitespace groups map onto those canonical fingerprints; a hidden whitespace-only group must not make the file look fully reviewed.
- Reconciliation (`jayjay_review::file_state`) preserves a group's state only on a unique digest match in both the stored baseline and the current snapshot. Line-number shifts from edits above a group are allowed; moving a patch to different context, splits, merges, and duplicate fingerprints become `ChangedSinceReview`. A previously reviewed group with no safe match is a file-level removed-reviewed tombstone. No baseline means current groups are `Unreviewed`. False invalidation is acceptable; false preservation is not.
- Persist a baseline only on user review mutations, not filesystem refresh. The `reviewed` map remains a compatibility mirror (`file_marked` plus hunk indices) with `mirror_digest` so an older binary that mutated `reviewed` alone does not keep a stale fingerprint baseline. Legacy entries whose identity still matches translate `file_marked`/hunk indices; a mismatched identity does not guess. `clear_change` drops that change's marks and baselines only.
- SwiftUI consumes `ReviewGroupState` / `ReviewFileRollup` through UniFFI and must not hash groups itself. GPUI currently uses file-level `is_reviewed` (identity-only marks) and does not render per-group dispositions.
- Images, binaries, submodules, content-free renames, projections, conflict-only placeholders, and unpaired renames keep conservative whole-file identity: do not invent empty hunk fingerprints for them.

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
