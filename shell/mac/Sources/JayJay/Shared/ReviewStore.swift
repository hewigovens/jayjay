import Foundation
import JayJayCore

/// Queries are answered from a per-file marks cache so per-gutter-line lookups never re-read the store from disk; mutations write through the Rust store (which read-modify-writes the shared file) and invalidate the cache.
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

    let storeURL: URL?
    var notes: [ReviewNote]
    /// Observable stand-in for the cache's contents: SwiftUI views read marks during render (gutter stripes, file rows), and without a tracked read a toggle would not re-render them until something else invalidated the view.
    private(set) var marksVersion: UInt64 = 0
    /// Bumped when every mark and note was dropped behind this window's back (Settings › Clear), so detail views re-read instead of trusting their @State.
    private(set) var resetGeneration: UInt64 = 0
    @ObservationIgnored private var marksCache: [MarksCacheKey: ReviewFileMarks] = [:]
    @ObservationIgnored var displayStatesCache: [DisplayStatesCacheKey: [ReviewGroupState]] = [:]

    init() {
        storeURL = reviewStorePath().map { URL(fileURLWithPath: $0) }
        notes = []
    }

    /// Test seam: persist to an explicit file instead of the shared store path.
    init(storeURL: URL) {
        self.storeURL = storeURL
        notes = []
    }

    var storePath: String? {
        storeURL?.path
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
        reviewMarkReviewed(
            changeId: changeId,
            path: path,
            identity: identity,
            snapshot: snapshot,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func markUnreviewed(changeId: String, path: String) {
        reviewMarkUnreviewed(changeId: changeId, path: path, storePath: storePath)
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleReviewed(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        reviewToggleReviewed(
            changeId: changeId,
            path: path,
            identity: identity,
            snapshot: snapshot,
            storePath: storePath
        )
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

    func fileRollups(changeId: String, files: [ReviewFileQuery]) -> [String: ReviewFileRollup] {
        _ = marksVersion
        let rollups = reviewFileRollups(
            changeId: changeId,
            paths: files.map(\.path),
            identities: files.map(\.identity),
            snapshots: files.map(\.snapshot),
            storePath: storePath
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
        reviewClearChange(changeId: changeId, storePath: storePath)
        invalidateAllMarks()
    }

    func summary() -> ReviewStoreSummary {
        reviewStoreSummary(storePath: storePath)
    }

    func clearAll() {
        reviewClearAll(storePath: storePath)
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
        let marks = reviewFileMarks(
            changeId: changeId,
            path: path,
            identity: identity,
            snapshot: nil,
            storePath: storePath
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
