import JayJayCore

extension ReviewStore {
    func isHunkReviewed(changeId: String, path: String, identity: String, hunkIndex: UInt32) -> Bool {
        let marks = fileMarks(changeId: changeId, path: path, identity: identity)
        if hunkIndex < marks.groupStates.count {
            return marks.groupStates[Int(hunkIndex)] == .reviewed
        }
        return marks.fileMarked || marks.hunks.contains(hunkIndex)
    }

    func displayHunkStates(changeId: String, query: ReviewDisplayQuery) -> [ReviewGroupState] {
        _ = marksVersion
        let key = DisplayStatesCacheKey(changeId: changeId, path: query.path, query: query.cacheKey)
        if let cached = displayStatesCache[key] {
            return cached
        }
        let states = native.displayHunkStates(
            changeId: changeId,
            path: query.path,
            identity: query.identity,
            snapshot: query.snapshot,
            mapping: query.mapping
        )
        displayStatesCache[key] = states
        return states
    }

    func toggleHunkReviewed(
        changeId: String,
        path: String,
        identity: String,
        hunkIndex: UInt32,
        snapshot: ReviewFileSnapshot? = nil
    ) {
        native.toggleHunk(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndex: hunkIndex,
            snapshot: snapshot
        )
        invalidateMarks(changeId: changeId, path: path)
    }

    func toggleDisplayHunk(changeId: String, query: ReviewDisplayQuery, displayIndex: UInt32) {
        native.toggleDisplayHunkSnapshot(
            changeId: changeId,
            path: query.path,
            identity: query.identity,
            snapshot: query.snapshot,
            mapping: query.mapping,
            displayIndex: displayIndex
        )
        invalidateMarks(changeId: changeId, path: query.path)
    }

    func setReviewedHunks(
        changeId: String,
        path: String,
        identity: String,
        hunkIndices: [UInt32],
        snapshot: ReviewFileSnapshot? = nil
    ) {
        native.setReviewedHunks(
            changeId: changeId,
            path: path,
            identity: identity,
            hunkIndices: hunkIndices,
            snapshot: snapshot
        )
        invalidateMarks(changeId: changeId, path: path)
    }
}
