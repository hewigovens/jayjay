import Foundation
import JayJayCore

/// Queries are answered from a per-file marks cache so per-gutter-line lookups never re-read the store from disk; mutations write through the Rust store (which read-modify-writes the shared file) and invalidate the cache.
@Observable
final class ReviewStore {
    typealias ReviewNote = NoteEntry

    let storeURL: URL?
    var notes: [ReviewNote]
    // Observable stand-in for the cache's contents: SwiftUI views read marks during render (gutter stripes, file rows), and without a tracked read a toggle would not re-render them until something else invalidated the view.
    private(set) var marksVersion: UInt64 = 0
    @ObservationIgnored private var marksCache: [String: ReviewFileMarks] = [:]
    @ObservationIgnored var displayStatesCache: [String: [ReviewGroupState]] = [:]

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
        var result: [String: ReviewFileRollup] = [:]
        let identityOnly = files.filter { !$0.hasSnapshot }
        if !identityOnly.isEmpty {
            let rollups = reviewFileRollups(
                changeId: changeId,
                paths: identityOnly.map(\.path),
                identities: identityOnly.map(\.identity),
                storePath: storePath
            )
            for (file, rollup) in zip(identityOnly, rollups) {
                result[file.path] = rollup
            }
        }
        for file in files where file.hasSnapshot {
            result[file.path] = reviewFileMarks(
                changeId: changeId,
                path: file.path,
                identity: file.identity,
                snapshot: file.snapshot,
                storePath: storePath
            ).rollup
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

    func fileMarks(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot? = nil
    ) -> ReviewFileMarks {
        _ = marksVersion
        let key = cacheKey(changeId: changeId, path: path, identity: identity, snapshot: snapshot)
        if let cached = marksCache[key] {
            return cached
        }
        let marks = reviewFileMarks(
            changeId: changeId,
            path: path,
            identity: identity,
            snapshot: snapshot,
            storePath: storePath
        )
        marksCache[key] = marks
        return marks
    }

    func cacheKey(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot?
    ) -> String {
        var key = "\(changeId)|\(path)|\(identity)"
        if let snapshot {
            key += "|" + snapshot.fingerprints.map(\.digest).joined(separator: ",")
        }
        return key
    }

    /// Evict only the mutated file so a toggle doesn't force a store re-read for every visible file.
    func invalidateMarks(changeId: String, path: String) {
        let prefix = "\(changeId)|\(path)|"
        marksCache = marksCache.filter { !$0.key.hasPrefix(prefix) }
        displayStatesCache = displayStatesCache.filter { !$0.key.hasPrefix(prefix) }
        marksVersion &+= 1
    }

    private func invalidateAllMarks() {
        marksCache.removeAll()
        displayStatesCache.removeAll()
        marksVersion &+= 1
    }
}

extension ReviewFileMarks {
    var rollup: ReviewFileRollup {
        if removedReviewedCount > 0 || groupStates.contains(.changedSinceReview) {
            return .changedSinceReview
        }
        if fileMarked || (!groupStates.isEmpty && groupStates.allSatisfy({ $0 == .reviewed })) {
            return .reviewed
        }
        if groupStates.contains(.reviewed) {
            return .partial
        }
        return .unreviewed
    }
}
