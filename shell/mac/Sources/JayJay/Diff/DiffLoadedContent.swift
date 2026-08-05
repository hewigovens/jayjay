import JayJayCore

struct DiffLoadedContent {
    var old: DiffContent
    var new: DiffContent
    var projection: DiffProjection?

    init(
        old: DiffContent = DiffContent(content: nil, preview: nil),
        new: DiffContent = DiffContent(content: nil, preview: nil),
        projection: DiffProjection? = nil
    ) {
        self.old = old
        self.new = new
        self.projection = projection
    }

    init(
        oldContent: String?,
        newContent: String?,
        oldPreview: DiffPreview? = nil,
        newPreview: DiffPreview? = nil,
        projection: DiffProjection? = nil
    ) {
        self.init(
            old: DiffContent(content: oldContent, preview: oldPreview),
            new: DiffContent(content: newContent, preview: newPreview),
            projection: projection
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
}
