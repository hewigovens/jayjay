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
}

struct DiffSectionLoadedDiff {
    var path: String
    var fileDiff: FileDiff?
    var displayLines: [DiffLine]?
    var displayGroups: [ChangeGroup]?
    var content: DiffLoadedContent
    var identity: DiffContextExpansionIdentity?
    var reviewSnapshot: ReviewFileSnapshot?
    var reviewMapping: [[UInt32]] = []

    func withReviewFingerprints(ignoreWhitespace: Bool) -> DiffSectionLoadedDiff {
        var copy = self
        guard content.projection == nil,
              let old = content.oldContent,
              let new = content.newContent,
              isEditableDiffText(text: old),
              isEditableDiffText(text: new)
        else {
            copy.reviewSnapshot = nil
            copy.reviewMapping = []
            return copy
        }
        copy.reviewSnapshot = reviewCanonicalSnapshot(oldContent: old, newContent: new)
        copy.reviewMapping = reviewDisplayGroupMap(
            oldContent: old,
            newContent: new,
            ignoreWhitespace: ignoreWhitespace
        )
        return copy
    }
}
