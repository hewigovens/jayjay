# Review State Guide

Load this file before changing review marks, review notes, the review store, or the reconciliation pipeline. Crate boundaries live in [Architecture Guide](architecture.md).

Review state (marks and notes) is persistent across app restarts and local to the user. Every surface shares the canonical Rust model and JSON format; desktop surfaces use one store file, while the browser uses an IndexedDB adapter.

## Store

- Canonical implementation: `jayjay_review::ReviewStore` (marks in `marks/`, notes in `note_store.rs`, reconciliation in `reconcile.rs`).
- Persistence and file format: [Storage Guide](storage.md). Every native review writer shares a lock across reload, mutation, and atomic replacement, including marks, ownership, notes, and clear operations. Use checked transactions for callers that must acknowledge persistence.
- SwiftUI: `Shared/ReviewStore.swift` is an `@Observable` UniFFI facade with a per-file marks cache.
- GPUI: one process-global store; mutate only through `window/review.rs::mutate` (refresh from disk first). Render-path reads use `refresh_if_stale`. Note reconciliation loads asynchronously on the view model (`loaders/review_notes.rs`, generation-guarded).

## Marks

- Keying: `(change_id, path) -> { identity, state }`, where `state` is a tagged file mark, index-only hunk mark, or fingerprinted group state. Always key by the real change id (`ChangeDetail.info.changeId`), never `selectionRevision` — that is a commit id for divergent working copies and hides state from CLI/core reconciliation.
- Review identity is computed in `jayjay_core::repo::diff::entry::compute_review_identity` from `MergedTreeValue` blob IDs. It is a fast-path detector (same bytes after a rebase keep the stored group states without rematerializing the file) but not the final review-state identity. Group fingerprints live in `jj_diff::canonical_review_snapshot`: exact-whitespace payloads plus a bounded unchanged context window that skips other groups' changed lines (so a hunk inserted nearby does not disturb a reviewed group), with no line numbers, wrapping, highlighting, or collapse. Display ignore-whitespace groups map onto those canonical fingerprints; a hidden whitespace-only group must not make the file look fully reviewed.
- Reconciliation (`jayjay_review::file_state`) preserves a group's state only on a unique digest match in both the stored baseline and the current snapshot. Line-number shifts from edits above a group are allowed; moving a patch to different context, splits, merges, and duplicate fingerprints become `ChangedSinceReview`. A previously reviewed group with no safe match is a file-level removed-reviewed tombstone, cleared once every current group is reviewed. No baseline means current groups are `Unreviewed`. False invalidation is acceptable; false preservation is not.
- Persist fingerprints only on user review mutations, not filesystem refresh. Unknown entry fields survive a save; pre-tag `file_marked`/`hunks` entries migrate to file/hunk states on load, unreadable ones are dropped, and notes remain intact either way. Snapshot-less file and hunk operations remain supported for GPUI through tagged file/index states; a mismatched identity does not guess. `clear_change` drops only that change's marks; `clear_all` (Settings) drops every mark and note.
- SwiftUI consumes `ReviewGroupState` / `ReviewFileRollup` through UniFFI and must not hash groups or decide snapshot eligibility itself: it fills its loaded content into the `DiffHunk` and asks `review_snapshot_from_hunk` / `review_display_group_map_from_hunk`. After the fast identity-only file-list query, it loads current snapshots off the main actor for changed baseline entries so unchanged groups can recover their reviewed rollup without opening each file; generation guards reject results for an older detail. GPUI computes the snapshot and display-group map once in the diff loader (`loaders/diff_compute.rs`, kept on `LoadedDiff`) and renders per-group gutter stripes from `display_hunk_states`; a file whose diff is not loaded falls back to identity-only marks.
- Images, binaries, submodules, content-free renames, projections, conflict-only placeholders, and unpaired renames keep conservative whole-file identity: do not invent empty hunk fingerprints for them.

## Notes

Notes attach reviewer feedback to diff lines — an anchor of side, line, excerpt, context, and the `ignore_whitespace` mode the diff was rendered with — and are consumed by agents through the CLI:

- Reconciliation (`ReviewStore::reconcile`) runs against a `ReviewDiffProvider`. Repository-backed GUI and CLI use `jayjay-core`'s provider (`repo/review_notes.rs`), keeping identity, rename pairing, and LFS/image placeholder content consistent. Remote sessions run the same provider on the client over the host's diffs; note bodies never go to the host. The browser's validated text-bundle provider (`packages/review-web/wasm/src/bundle.rs`) implements the same trait and requires caller-supplied native review identities; it does not materialize repository objects or binary placeholders.
- Anchor matching replays the exact GUI render pipeline: Histogram `compute_file_diff` with the note's recorded `ignore_whitespace`, then display lines, then `change_group_for_anchor`. If you change any stage of the diff pipeline, note anchors are part of its contract.
- GUI anchors are built shell-side from the rendered line (side/line from the line's span style, excerpt/context from the display lines and change group) and stamped with the ignore-whitespace mode the diff was actually rendered with. `Repo::review_note_anchor`/`build_note_anchor` are the CLI add-note path only — they hardcode `ignore_whitespace = false`, so calling them from a GUI with the whitespace toggle on either errors or records a wrong-mode anchor that can reconcile stale in the very diff it was created from.

## External Working-Copy Review

`jayjay_core::WorkingCopyReview::open(path)` pins the workspace’s `@` to one existing operation without snapshotting, importing Git state, updating stale workspaces, or resolving concurrent operation heads. Multiple operation heads return an error for the caller to retry after the repository owner reconciles them. `files()` lists metadata; `load_file()` materializes one file through the native provider; `status(&store, ignore_whitespace)` returns loaded files, canonical snapshots, display-to-canonical group maps, rollups, ownership, and group states. Read a fresh `ReviewStore::try_load()` for each API refresh.

`set_reviewed(&mut store, &WorkingCopyReviewMark)` accepts the full expected commit ID, path, optional display-group index, displayed whitespace mode, and desired reviewed state. A group index is scoped to that commit, path, and mode. The setter checks the current workspace commit again just before taking the review-store lock (a later rewrite still reconciles as changed since review), rejects stale requests, and writes a person’s ownership without changing jj state. Whole-file actions take ownership of that file; group actions preserve untouched owners. Branch/range comparisons are read-only: their combined fingerprints must never be saved under the working-copy change ID.

`open_change(path, change_id)` gives the same view of another mutable change, such as an unpushed ancestor of `@`, keyed by that change; its writes re-resolve the full change id and are refused once it was rewritten. Bookmarks, revsets, divergent ids and immutable changes are refused.

`notes(&store)` reconciles the change's notes against the inspected commit. `add_note(&mut store, &WorkingCopyReviewNote)` anchors with the same pipeline as reconciliation in the displayed whitespace mode and saves only if `@` is still the expected commit; `resolve_note` accepts only IDs belonging to the inspected change. Both write under the review-store lock and return errors on failed saves.

Connected Control Room clients must link this Rust API and use the shared native store as authority. The standalone browser PWA’s IndexedDB adapter remains an offline/local prototype; connected clients must route shared marks through the server rather than overwrite native state with a cached JSON document.

## Agent Workflow

Read notes only when the change used review notes or the user requested reconciliation, per [Local Review Notes](../AGENTS.md#local-review-notes). Serialize these JJ-aware commands with other repository operations:

```bash
jayjay review notes --repo .                  # plain text with bodies and anchor lines
jayjay review notes --repo . --format json    # structured pipeline output
jayjay review resolve-note <id> --repo .
jayjay review add-note --repo . --file <path> --line <n> [--side new|old] -m "note body"
jayjay review status --repo . [--format json] # per-file rollup of the working-copy change
jayjay review mark --repo . --expected-commit <commit-id> --file <path> [--line <n> [--side new|old]]
jayjay review unmark --repo . [--file <path>] # drops agent marks only
```

`status` reports the full `commit_id` of the snapshotted working copy. Inspect that immutable commit with `jj --ignore-working-copy diff -r <commit-id>`, then pass it to `mark --expected-commit`. `mark` snapshots the working copy and rejects a different commit before writing anything; refresh status and inspect again after rejection. It fingerprints the pinned commit with the same logic as the shells; `--line` marks only the group containing that changed line.

Ownership is per canonical group. An user group action takes ownership of only the groups it touches, while a whole-file review takes ownership of the whole file. Agent marks preserve already reviewed user groups. The entry's `source: "agent"` means at least one current group or removed-group warning is agent-owned; both shells show an agent tag and `status` reports that aggregate per file. `unmark` removes only agent-owned marks and warnings, preserving user marks in mixed files. Agent mutations persist marks and ownership atomically and return errors on failed saves. The skill `review-triage` in `.agents/skills/` runs the pre-review pass that uses these commands.

Treat `current` notes as actionable, `stale` notes as needing re-check against the changed diff, and `orphaned` notes as comments whose original file or anchor disappeared. Resolve notes only after addressing the feedback; resolution is limited to IDs belonging to the repository's working-copy change.

Agent triage notes are reserved for concrete, unresolved critical risks; ordinary unreviewed code needs no note. Follow the [review-triage skill](../.agents/skills/review-triage/SKILL.md) for the threshold and content. Keep qualifying review feedback in notes rather than source-code comments because notes never ship in the change. Adding a note to the same changed line updates that line's active note.
