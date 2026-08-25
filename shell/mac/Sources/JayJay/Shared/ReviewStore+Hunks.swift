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
        let states = reviewDisplayHunkStates(
            changeId: changeId,
            path: query.path,
            identity: query.identity,
            snapshot: query.snapshot,
            mapping: query.mapping,
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

    func toggleDisplayHunk(changeId: String, query: ReviewDisplayQuery, displayIndex: UInt32) {
        reviewToggleDisplayHunkSnapshot(
            changeId: changeId,
            path: query.path,
            identity: query.identity,
            snapshot: query.snapshot,
            mapping: query.mapping,
            displayIndex: displayIndex,
            storePath: storePath
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
