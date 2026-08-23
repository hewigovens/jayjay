import JayJayCore

extension ReviewStore {
    func isHunkReviewed(
        changeId: String,
        path: String,
        identity: String,
        hunkIndex: UInt32,
        snapshot: ReviewFileSnapshot? = nil
    ) -> Bool {
        let marks = fileMarks(changeId: changeId, path: path, identity: identity, snapshot: snapshot)
        if hunkIndex < marks.groupStates.count {
            return marks.groupStates[Int(hunkIndex)] == .reviewed
        }
        return marks.fileMarked || marks.hunks.contains(hunkIndex)
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
        reviewMarkHunkReviewed(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndex: hunkIndex,
            snapshot: snapshot,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleHunkReviewed(
        changeId: String,
        path: String,
        identity: String,
        hunkIndex: UInt32,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        reviewToggleHunk(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndex: hunkIndex,
            snapshot: snapshot,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleDisplayHunk(
        changeId: String,
        file: ReviewFileQuery,
        mapping: [[UInt32]],
        displayIndex: UInt32
    ) {
        guard let snapshot = file.snapshot else { return }
        reviewToggleDisplayHunkSnapshot(
            changeId: changeId,
            path: file.path,
            identity: file.identity,
            snapshot: snapshot,
            mapping: mapping,
            displayIndex: displayIndex,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: file.path)
    }

    func setReviewedHunks(
        changeId: String,
        path: String,
        identity: String,
        hunkIndices: [UInt32],
        snapshot: ReviewFileSnapshot? = nil
    ) {
        reviewSetReviewedHunks(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndices: hunkIndices,
            snapshot: snapshot,
            storePath: storePath
        )
        invalidateMarks(changeId: changeId, path: path)
    }
}
