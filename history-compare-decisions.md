# #383 Compare two revisions from file history: edge decisions

Delete this file before `jj describe`.

Split into two runs so each fits the context window. **Run 1: core checks + SwiftUI.** **Run 2: GPUI**, which mirrors run 1's behavior. Do only the run you are asked to do.

## Existing pieces (reuse, do not rebuild)

- Core: `Repo::file_history(path)` (`crates/jayjay-core/src/repo/annotate.rs`) returns `ChangeInfo`s with exact `commit_id`s. `Repo::interdiff_file*` (`repo/diff.rs`) diff two commits' trees (`TreePair::between`, a snapshot diff, not `jj interdiff`'s rebased form). These are the comparison pipeline. No new core API is expected. Add one only if a shell would otherwise duplicate logic.
- SwiftUI: `FileHistoryView` is shown by `ChangeDetailView+PreviewColumn.swift` in `DetailPaneMode.fileHistory`. `DiffSection` already supports a read-only comparison through `compareFromRev` (it disables review actions when that is set, and its `.task(id:)` keys on from/rev/path). Use `DiffSection` for the comparison diff rather than a new diff view.
- GPUI: `shell/gpui/src/windows/file_history.rs` (separate window); `windows/evolog/diff.rs` already renders `interdiff_file*` results; reuse its approach.

## Decisions

- D1 Endpoints are commit IDs. Both base and target hold the entry's `commit_id` captured when chosen. Never re-resolve by change id, so a rewrite or divergent sibling can't retarget them. Labels show the change id **and** the short commit id.
- D2 List identity. History rows are keyed by `commit_id`, not `change_id`. Today's `List(history, id: \.changeId)` collides for divergent entries; fix it.
- D3 Interaction. A single click (or ↑/↓) **selects** a row; the selected row is the target. Return or double-click reveals the change in the DAG, which is the old single-click behavior, preserved. "Pin as Base" is in the row context menu and in a header button that pins the selected row. Header Swap and Clear buttons appear only while a base is pinned. Add no new global shortcuts.
- D4 What the lower diff shows. History list on top, diff for `path` below. With no base pinned, the selected entry's own change to the file (`rev` = target commit, no from). With a base pinned, base commit → target commit for this path. When the target is the base itself, show a short "Same revision as the base" note, not an empty diff.
- D5 Swap exchanges base and target; selection follows the new target row. Clear drops the base, keeps the selected row and scroll position, and returns to D4's no-base view.
- D6 Esc clears the pinned base first if there is one; otherwise it exits history (existing behavior).
- D7 Stale results. Comparison loads are keyed by (base commit, target commit, path). A result whose key no longer matches the current selection is dropped. Fast arrowing must never show an older pair's diff under the current labels. `DiffSection`'s `.task(id:)` cancellation may already cover this; verify it and don't add a second mechanism if so.
- D8 Unsupported content. Deleted-at-one-end, binary, and missing-file cases come from the existing diff capabilities (`HunkType`, binary placeholder). Don't add parallel handling. History is path-filtered and does not follow renames, so an endpoint before a rename is not listed. No rename-specific UI.
- D9 Read-only. Nothing in this flow edits, checks out, or snapshots the working copy. Review marks and notes are not offered in comparison mode.
- D10 State shape. Keep comparison state in one small type with methods, e.g. Swift `FileHistoryComparison { base, target }` with `pin`/`swap`/`clear`, not loose `@State` fields. GPUI does the same on its view struct. Put it in its own file per AGENTS.md.

## Acceptance

Run 1:
1. Core test (jj-test fixtures, see `crates/jj-test`): a file edited in three changes; `interdiff_file(commit_a, commit_c, path)` shows the a→c content. With a divergent change id at one end, the commit-id endpoint still resolves to its own content. A change that deletes the file reads as a deletion. If an existing test already proves one of these, don't add a duplicate. Note it instead.
2. Swift unit test for `FileHistoryComparison`: pin/swap/clear and the "same revision" case.
3. SwiftUI build passes (`just build` or the Xcode scheme build). Swift unit tests for the new type pass. Do not write or run XCUITests; the reviewer runs UI scenes.

Run 2:
4. GPUI component test in `shell/gpui/tests/gpui/` covering pin → arrow target → swap → clear, with endpoint labels and the stale-result guard (D7). Use the existing file-history test helpers if any.

## Traps the reviewer will check

- Rows keyed by change id; divergent rows collapse (D2).
- Endpoints resolved by change id after a refresh (D1).
- Old click behavior (reveal in DAG) lost (D3).
- An older async result painted under newer labels (D7).
- Comparison offering review marks or edit actions (D9).
- Tests that pass with the guard removed.
