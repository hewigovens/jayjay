import Foundation
import JayJayCore

/// Queries are answered from a per-file marks cache over one long-lived Rust store, which re-reads the shared file only after another writer changed it; mutations write through it and invalidate the cache.
@Observable
final class ReviewStore {
    typealias ReviewNote = NoteEntry

    struct MarksCacheKey: Hashable {
        let changeId: String
        let path: String
        let identity: String
    }

    struct DisplayStatesCacheKey: Hashable {
        let changeId: String
        let path: String
        let query: String
    }

    let native: SharedReviewStore
    var notes: [ReviewNote]
    /// Observable stand-in for the cache's contents: SwiftUI views read marks during render (gutter stripes, file rows), and without a tracked read a toggle would not re-render them until something else invalidated the view.
    private(set) var marksVersion: UInt64 = 0
    /// Bumped when every mark and note was dropped behind this window's back (Settings › Clear), so detail views re-read instead of trusting their @State.
    private(set) var resetGeneration: UInt64 = 0
    @ObservationIgnored private var marksCache: [MarksCacheKey: ReviewFileMarks] = [:]
    @ObservationIgnored var displayStatesCache: [DisplayStatesCacheKey: [ReviewGroupState]] = [:]

    init() {
        native = SharedReviewStore(storePath: nil)
        notes = []
    }

    /// Test seam: persist to an explicit file instead of the shared store path.
    init(storeURL: URL) {
        native = SharedReviewStore(storePath: storeURL.path)
        notes = []
    }

    func isReviewed(changeId: String, path: String, identity: String) -> Bool {
        fileMarks(changeId: changeId, path: path, identity: identity).fileMarked
    }

    func fileRollup(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot? = nil
    ) -> ReviewFileRollup {
        fileRollups(
            changeId: changeId,
            files: [ReviewFileQuery(path: path, identity: identity, snapshot: snapshot)]
        )[path] ?? .unreviewed
    }

    func markReviewed(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        native.markReviewed(
            changeId: changeId,
            path: path,
            identity: identity,
            snapshot: snapshot
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func markUnreviewed(changeId: String, path: String) {
        native.markUnreviewed(changeId: changeId, path: path)
        invalidateMarks(changeId: changeId, path: path)
    }

    /// One fresh store read for the whole file list, so refreshes observe marks written by other windows, GPUI, or the CLI.
    func reviewedPaths(changeId: String, files: [(path: String, identity: String)]) -> Set<String> {
        Set(fileRollups(
            changeId: changeId,
            files: files.map { ReviewFileQuery(path: $0.path, identity: $0.identity) }
        ).compactMap { path, rollup in
            rollup == .reviewed ? path : nil
        })
    }

    func agentMarkedPaths(changeId: String) -> Set<String> {
        _ = marksVersion
        return Set(native.agentMarkedPaths(changeId: changeId))
    }

    func fileRollups(changeId: String, files: [ReviewFileQuery]) -> [String: ReviewFileRollup] {
        _ = marksVersion
        let rollups = native.fileRollups(
            changeId: changeId,
            paths: files.map(\.path),
            identities: files.map(\.identity),
            snapshots: files.map(\.snapshot)
        )
        var result: [String: ReviewFileRollup] = [:]
        for (file, rollup) in zip(files, rollups) {
            result[file.path] = rollup
        }
        return result
    }

    /// Drops all cached marks so the next queries re-read the shared store; called when review state refreshes, since external writers never notify this process.
    func invalidateMarksCache() {
        invalidateAllMarks()
    }

    /// Clears only this change's marks: the store is shared, so other changes and windows keep their review state.
    func clearChange(changeId: String) {
        native.clearChange(changeId: changeId)
        invalidateAllMarks()
    }

    func summary() -> ReviewStoreSummary {
        native.summary()
    }

    func clearAll() {
        native.clearAll()
        applyExternalReset()
    }

    func applyExternalReset() {
        notes = []
        invalidateAllMarks()
        resetGeneration &+= 1
    }

    func fileMarks(changeId: String, path: String, identity: String) -> ReviewFileMarks {
        _ = marksVersion
        let key = MarksCacheKey(changeId: changeId, path: path, identity: identity)
        if let cached = marksCache[key] {
            return cached
        }
        let marks = native.fileMarks(
            changeId: changeId,
            path: path,
            identity: identity
        )
        marksCache[key] = marks
        return marks
    }

    /// Evict only the mutated file so a toggle doesn't force a store re-read for every visible file.
    func invalidateMarks(changeId: String, path: String) {
        marksCache = marksCache.filter { key, _ in
            key.changeId != changeId || key.path != path
        }
        displayStatesCache = displayStatesCache.filter { key, _ in
            key.changeId != changeId || key.path != path
        }
        marksVersion &+= 1
    }

    private func invalidateAllMarks() {
        marksCache.removeAll()
        displayStatesCache.removeAll()
        marksVersion &+= 1
    }
}
