import Foundation
import JayJayCore

/// Queries are answered from a per-file marks cache so per-gutter-line lookups never re-read the store from disk; mutations write through the Rust store (which read-modify-writes the shared file) and invalidate the cache.
@Observable
final class ReviewStore {
    typealias ReviewNote = NoteEntry

    private static let legacyStorageKey = "jayjay.reviewedFiles"

    let storeURL: URL?
    var notes: [ReviewNote]
    // Observable stand-in for the cache's contents: SwiftUI views read marks during render (gutter stripes, file rows), and without a tracked read a toggle would not re-render them until something else invalidated the view.
    private var marksVersion = 0
    @ObservationIgnored private var marksCache: [String: ReviewFileMarks] = [:]

    init() {
        storeURL = reviewStorePath().map { URL(fileURLWithPath: $0) }
        notes = []
        importLegacyMarks(from: .standard)
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

    func markReviewed(changeId: String, path: String, identity: String) {
        reviewMarkReviewed(changeId: changeId, path: path, identity: identity, storePath: storePath)
        invalidateMarks(changeId: changeId, path: path)
    }

    func markUnreviewed(changeId: String, path: String) {
        reviewMarkUnreviewed(changeId: changeId, path: path, storePath: storePath)
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleReviewed(changeId: String, path: String, identity: String) {
        reviewToggleReviewed(changeId: changeId, path: path, identity: identity, storePath: storePath)
        invalidateMarks(changeId: changeId, path: path)
    }

    /// One fresh store read for the whole file list, so refreshes observe marks written by other windows, GPUI, or the CLI.
    func reviewedPaths(changeId: String, files: [(path: String, identity: String)]) -> Set<String> {
        Set(reviewReviewedPaths(
            changeId: changeId,
            paths: files.map(\.path),
            identities: files.map(\.identity),
            storePath: storePath
        ))
    }

    /// Drops all cached marks so the next queries re-read the shared store; called when review state refreshes, since external writers never notify this process.
    func invalidateMarksCache() {
        invalidateAllMarks()
    }

    // MARK: Hunk-level review

    /// file_marked implies all hunks reviewed; otherwise check the explicit set.
    func isHunkReviewed(changeId: String, path: String, identity: String, hunkIndex: UInt32) -> Bool {
        let marks = fileMarks(changeId: changeId, path: path, identity: identity)
        return marks.fileMarked || marks.hunks.contains(hunkIndex)
    }

    func markHunkReviewed(changeId: String, path: String, identity: String, hunkIndex: UInt32) {
        reviewMarkHunkReviewed(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndex: hunkIndex,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func markHunkUnreviewed(changeId: String, path: String, hunkIndex: UInt32) {
        reviewMarkHunkUnreviewed(
            changeId: changeId,
            path: path,
            hunkIndex: hunkIndex,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleHunkReviewed(changeId: String, path: String, identity: String, hunkIndex: UInt32) {
        reviewToggleHunk(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndex: hunkIndex,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func setReviewedHunks(changeId: String, path: String, identity: String, hunkIndices: [UInt32]) {
        reviewSetReviewedHunks(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndices: hunkIndices,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    /// Clears only this change's marks: the store is shared, so other changes and windows keep their review state.
    func clearChange(changeId: String) {
        reviewClearChange(changeId: changeId, storePath: storePath)
        invalidateAllMarks()
    }

    // MARK: Marks cache

    private func fileMarks(changeId: String, path: String, identity: String) -> ReviewFileMarks {
        _ = marksVersion
        let key = "\(changeId)|\(path)|\(identity)"
        if let cached = marksCache[key] {
            return cached
        }
        let marks = reviewFileMarks(
            changeId: changeId, path: path, identity: identity, storePath: storePath
        )
        marksCache[key] = marks
        return marks
    }

    /// Evict only the mutated file so a toggle doesn't force a store re-read for every visible file.
    private func invalidateMarks(changeId: String, path: String) {
        let prefix = "\(changeId)|\(path)|"
        marksCache = marksCache.filter { !$0.key.hasPrefix(prefix) }
        marksVersion &+= 1
    }

    private func invalidateAllMarks() {
        marksCache.removeAll()
        marksVersion &+= 1
    }

    // MARK: Legacy migration

    /// One-time import of marks the old UserDefaults-backed store left behind; runs only while no shared store file exists yet, then drops the legacy blob.
    func importLegacyMarks(from defaults: UserDefaults) {
        guard let storeURL, !FileManager.default.fileExists(atPath: storeURL.path),
              let data = defaults.data(forKey: Self.legacyStorageKey),
              let raw = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return }
        for (key, value) in raw {
            guard let separator = key.firstIndex(of: "|"),
                  let dict = value as? [String: Any],
                  let identity = dict["identity"] as? String,
                  !identity.isEmpty
            else { continue }
            let changeId = String(key[..<separator])
            let path = String(key[key.index(after: separator)...])
            let hunks = (dict["hunks"] as? [Int])?.compactMap { $0 >= 0 ? UInt32($0) : nil } ?? []
            if dict["file_marked"] as? Bool ?? false {
                markReviewed(changeId: changeId, path: path, identity: identity)
            } else if !hunks.isEmpty {
                setReviewedHunks(changeId: changeId, path: path, identity: identity, hunkIndices: hunks)
            }
        }
        defaults.removeObject(forKey: Self.legacyStorageKey)
    }
}
