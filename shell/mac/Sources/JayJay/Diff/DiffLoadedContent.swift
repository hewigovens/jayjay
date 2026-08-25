import JayJayCore

struct DiffLoadedContent {
    var old: DiffContent
    var new: DiffContent
    var projection: DiffProjection?
    var supportsFileEditor: Bool

    init(
        old: DiffContent = DiffContent(content: nil, preview: nil),
        new: DiffContent = DiffContent(content: nil, preview: nil),
        projection: DiffProjection? = nil,
        supportsFileEditor: Bool = false
    ) {
        self.old = old
        self.new = new
        self.projection = projection
        self.supportsFileEditor = supportsFileEditor
    }

    init(
        oldContent: String?,
        newContent: String?,
        oldPreview: DiffPreview? = nil,
        newPreview: DiffPreview? = nil,
        projection: DiffProjection? = nil,
        supportsFileEditor: Bool = false
    ) {
        self.init(
            old: DiffContent(content: oldContent, preview: oldPreview),
            new: DiffContent(content: newContent, preview: newPreview),
            projection: projection,
            supportsFileEditor: supportsFileEditor
        )
    }

    var oldContent: String? {
        old.content
    }

    var newContent: String? {
        new.content
    }

    var oldPreview: DiffPreview? {
        old.preview
    }

    var newPreview: DiffPreview? {
        new.preview
    }

    var oldText: String {
        old.content ?? ""
    }

    var newText: String {
        new.content ?? ""
    }

    /// The summary hunk with this loaded content filled in, so core decides review-snapshot eligibility from one place.
    nonisolated func applied(to hunk: DiffHunk) -> DiffHunk {
        DiffHunk(
            path: hunk.path,
            oldPath: hunk.oldPath,
            old: old,
            new: new,
            hunkType: hunk.hunkType,
            supportsConflictEditor: hunk.supportsConflictEditor,
            supportsFileEditor: hunk.supportsFileEditor,
            reviewIdentity: hunk.reviewIdentity,
            projection: projection
        )
    }
}

struct DiffSectionLoadedDiff {
    var path: String
    var fileDiff: FileDiff?
    var displayLines: [DiffLine]?
    var displayGroups: [ChangeGroup]?
    var content: DiffLoadedContent
    var identity: DiffContextExpansionIdentity?
    var reviewQuery: ReviewDisplayQuery?

    nonisolated func withReviewFingerprints(hunk: DiffHunk, ignoreWhitespace: Bool) -> DiffSectionLoadedDiff {
        var copy = self
        let loaded = content.applied(to: hunk)
        let snapshot = reviewSnapshotFromDiffHunk(hunk: loaded)
        guard !snapshot.fingerprints.isEmpty else {
            copy.reviewQuery = nil
            return copy
        }
        copy.reviewQuery = ReviewDisplayQuery(
            path: hunk.path,
            identity: hunk.reviewIdentity,
            snapshot: snapshot,
            mapping: reviewDisplayGroupMapFromDiffHunk(hunk: loaded, ignoreWhitespace: ignoreWhitespace)
        )
        return copy
    }
}
