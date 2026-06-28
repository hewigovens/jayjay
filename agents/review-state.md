# Review State Guide

Load this file before changing review marks, review notes, the review store, or the reconciliation pipeline. Crate boundaries live in [Architecture Guide](architecture.md).

Review state (marks and notes) is persistent across app restarts, local to the user, and shared by every surface through one store file.

## Store

- Canonical implementation: `jayjay_review::ReviewStore` (marks in `marks.rs`, notes in `note_store.rs`, reconciliation in `reconcile.rs`).
- Persistence: `review_store.json` under the app config dir (`~/Library/Application Support/dev.hewig.jayjay/` on macOS); `JAYJAY_REVIEW_STORE_PATH` overrides it for tests. Writes are atomic (temp + rename). An unparseable file is preserved as `.json.corrupt` before defaulting; individual notes that fail to parse — or carry fields from a newer version — are carried through save untouched.
- Shells: the SwiftUI `Shared/ReviewStore.swift` is an `@Observable` facade over UniFFI calls with a per-file marks cache (invalidated on mutation) and a one-time UserDefaults legacy import. GPUI holds one process-global store and must mutate through `window/review.rs::mutate`, which refreshes from disk first so a long-lived snapshot never clobbers writes from the CLI or the other shell.

## Marks

- Keying: `(change_id, path) -> { identity, file_marked, hunks }`. Always key by the real change id (`ChangeDetail.info.changeId`), never `selectionRevision` — that is a commit id for divergent working copies and hides state from CLI/core reconciliation.
- Review identity is computed in `jayjay_core::repo::diff::entry::compute_review_identity` from `MergedTreeValue` blob IDs, so reviews survive rebases/amends that do not change file bytes; any byte change invalidates only that file. Renamed files fold into a combined `rename|old|new` identity. The file-list identities come from `show_summary` (content-free rename pairing); anchors and reconciliation must use those, not `show`'s content-similarity pairing.
- Hunk indices are the canonical change groups from `jj_diff::change_groups` over `build_diff_display_lines` output. `is_hunk_reviewed(idx)` is true when the file is marked or the hunk set contains `idx`; marking every hunk promotes to file-level review, and unmarking a hunk drops the file flag. Committing clears only that change's marks (`clear_change`).

## Notes

Notes attach reviewer feedback to diff lines — an anchor of side, line, excerpt, context, and the `ignore_whitespace` mode the diff was rendered with — and are consumed by agents via the CLI (see `AGENTS.md`):

- `jayjay review notes --repo . --format json` reconciles each note to `current`, `stale`, `orphaned`, or `resolved`; `jayjay review resolve-note <id>` resolves ids belonging to the repo's working-copy change only.
- Reconciliation (`ReviewStore::reconcile`) runs against a `ReviewDiffProvider`. The only production provider is `jayjay-core`'s (`repo/review_notes.rs`), used by GUI and CLI alike — this is what keeps both surfaces agreeing on identity, rename pairing, and LFS/image placeholder content.
- Anchor matching replays the exact GUI render pipeline: Histogram `compute_file_diff` with the note's recorded `ignore_whitespace`, then display lines, then `change_group_for_anchor`. If you change any stage of the diff pipeline, note anchors are part of its contract.
