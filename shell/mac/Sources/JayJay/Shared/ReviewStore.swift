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
    @ObservationIgnored private var displayStatesCache: [String: [ReviewGroupState]] = [:]

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

    // MARK: File-level review

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
            files: [(path: path, identity: identity, snapshot: snapshot)]
        )[path] ?? .unreviewed
    }

    func markReviewed(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        if let snapshot {
            reviewMarkReviewedSnapshot(
                changeId: changeId,
                path: path,
                identity: identity,
                snapshot: snapshot,
                storePath: storePath
            )
        } else {
            reviewMarkReviewed(changeId: changeId, path: path, identity: identity, storePath: storePath)
        }
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
        if let snapshot {
            reviewToggleReviewedSnapshot(
                changeId: changeId,
                path: path,
                identity: identity,
                snapshot: snapshot,
                storePath: storePath
            )
        } else {
            reviewToggleReviewed(changeId: changeId, path: path, identity: identity, storePath: storePath)
        }
        invalidateMarks(changeId: changeId, path: path)
    }

    /// One fresh store read for the whole file list, so refreshes observe marks written by other windows, GPUI, or the CLI.
    func reviewedPaths(changeId: String, files: [(path: String, identity: String)]) -> Set<String> {
        Set(fileRollups(changeId: changeId, files: files).compactMap { path, rollup in
            rollup == .reviewed ? path : nil
        })
    }

    func fileRollups(changeId: String, files: [(path: String, identity: String)]) -> [String: ReviewFileRollup] {
        fileRollups(
            changeId: changeId,
            files: files.map { (path: $0.path, identity: $0.identity, snapshot: nil) }
        )
    }

    func fileRollups(
        changeId: String,
        files: [(path: String, identity: String, snapshot: ReviewFileSnapshot?)]
    ) -> [String: ReviewFileRollup] {
        _ = marksVersion
        var result: [String: ReviewFileRollup] = [:]
        let identityOnly = files.filter { $0.snapshot.map(\.fingerprints.isEmpty) ?? true }
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
        for file in files {
            guard let snapshot = file.snapshot, !snapshot.fingerprints.isEmpty else { continue }
            result[file.path] = reviewFileMarksWithSnapshot(
                changeId: changeId,
                path: file.path,
                identity: file.identity,
                snapshot: snapshot,
                storePath: storePath
            ).rollup
        }
        return result
    }

    /// Drops all cached marks so the next queries re-read the shared store; called when review state refreshes, since external writers never notify this process.
    func invalidateMarksCache() {
        invalidateAllMarks()
    }

    // MARK: Hunk-level review

    func hunkState(
        changeId: String,
        path: String,
        identity: String,
        hunkIndex: UInt32,
        snapshot: ReviewFileSnapshot? = nil
    ) -> ReviewGroupState {
        let marks = fileMarks(changeId: changeId, path: path, identity: identity, snapshot: snapshot)
        if hunkIndex < marks.groupStates.count {
            return marks.groupStates[Int(hunkIndex)]
        }
        if marks.fileMarked || marks.hunks.contains(hunkIndex) {
            return .reviewed
        }
        return .unreviewed
    }

    /// file_marked implies all hunks reviewed; otherwise check the explicit set.
    func isHunkReviewed(
        changeId: String,
        path: String,
        identity: String,
        hunkIndex: UInt32,
        snapshot: ReviewFileSnapshot? = nil
    ) -> Bool {
        hunkState(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndex: hunkIndex,
            snapshot: snapshot
        ) == .reviewed
    }

    func displayHunkStates(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot,
        mapping: [[UInt32]]
    ) -> [ReviewGroupState] {
        _ = marksVersion
        let key = cacheKey(changeId: changeId, path: path, identity: identity, snapshot: snapshot)
            + "|map:" + mapping.map { $0.map(String.init).joined(separator: ".") }.joined(separator: "/")
        if let cached = displayStatesCache[key] {
            return cached
        }
        let states = reviewDisplayHunkStates(
            changeId: changeId,
            path: path,
            identity: identity,
            snapshot: snapshot,
            mapping: mapping,
            storePath: storePath
        )
        displayStatesCache[key] = states
        return states
    }

    func markHunkReviewed(
        changeId: String,
        path: String,
        identity: String,
        hunkIndex: UInt32,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        if let snapshot {
            reviewMarkHunkReviewedSnapshot(
                changeId: changeId,
                path: path,
                identity: identity,
                snapshot: snapshot,
                hunkIndex: hunkIndex,
                storePath: storePath
            )
        } else {
            reviewMarkHunkReviewed(
                changeId: changeId,
                path: path,
                identity: identity,
                hunkIndex: hunkIndex,
                storePath: storePath
            )
        }
        invalidateMarks(changeId: changeId, path: path)
    }

    func markHunkUnreviewed(
        changeId: String,
        path: String,
        hunkIndex: UInt32,
        identity: String? = nil,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        if let identity, let snapshot {
            reviewMarkHunkUnreviewedSnapshot(
                changeId: changeId,
                path: path,
                identity: identity,
                snapshot: snapshot,
                hunkIndex: hunkIndex,
                storePath: storePath
            )
        } else {
            reviewMarkHunkUnreviewed(
                changeId: changeId,
                path: path,
                hunkIndex: hunkIndex,
                storePath: storePath
            )
        }
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleHunkReviewed(
        changeId: String,
        path: String,
        identity: String,
        hunkIndex: UInt32,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        if let snapshot {
            reviewToggleHunkSnapshot(
                changeId: changeId,
                path: path,
                identity: identity,
                snapshot: snapshot,
                hunkIndex: hunkIndex,
                storePath: storePath
            )
        } else {
            reviewToggleHunk(
                changeId: changeId,
                path: path,
                identity: identity,
                hunkIndex: hunkIndex,
                storePath: storePath
            )
        }
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleDisplayHunk(
        changeId: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot,
        mapping: [[UInt32]],
        displayIndex: UInt32
    ) {
        reviewToggleDisplayHunkSnapshot(
            changeId: changeId,
            path: path,
            identity: identity,
            snapshot: snapshot,
            mapping: mapping,
            displayIndex: displayIndex,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func setReviewedHunks(
        changeId: String,
        path: String,
        identity: String,
        hunkIndices: [UInt32],
        snapshot: ReviewFileSnapshot? = nil
    ) {
        if let snapshot {
            reviewSetReviewedHunksSnapshot(
                changeId: changeId,
                path: path,
                identity: identity,
                snapshot: snapshot,
                hunkIndices: hunkIndices,
                storePath: storePath
            )
        } else {
            reviewSetReviewedHunks(
                changeId: changeId,
                path: path,
                identity: identity,
                hunkIndices: hunkIndices,
                storePath: storePath
            )
        }
        invalidateMarks(changeId: changeId, path: path)
    }

    /// Clears only this change's marks: the store is shared, so other changes and windows keep their review state.
    func clearChange(changeId: String) {
        reviewClearChange(changeId: changeId, storePath: storePath)
        invalidateAllMarks()
    }

    // MARK: Marks cache

    private func fileMarks(
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
        let marks: ReviewFileMarks
        if let snapshot {
            marks = reviewFileMarksWithSnapshot(
                changeId: changeId,
                path: path,
                identity: identity,
                snapshot: snapshot,
                storePath: storePath
            )
        } else {
            marks = reviewFileMarks(
                changeId: changeId, path: path, identity: identity, storePath: storePath
            )
        }
        marksCache[key] = marks
        return marks
    }

    private func cacheKey(
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
    private func invalidateMarks(changeId: String, path: String) {
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
