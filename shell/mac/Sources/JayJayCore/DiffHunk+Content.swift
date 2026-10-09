public extension DiffHunk {
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

    init(
        path: String,
        oldPath: String?,
        oldContent: String?,
        newContent: String?,
        oldPreview: DiffPreview?,
        newPreview: DiffPreview?,
        hunkType: HunkType,
        supportsConflictEditor: Bool = false,
        supportsFileEditor: Bool = false,
        reviewIdentity: String,
        projection: DiffProjection?,
        matchesSource: Bool = false
    ) {
        self.init(
            path: path,
            oldPath: oldPath,
            old: DiffContent(content: oldContent, preview: oldPreview),
            new: DiffContent(content: newContent, preview: newPreview),
            hunkType: hunkType,
            supportsConflictEditor: supportsConflictEditor,
            supportsFileEditor: supportsFileEditor,
            reviewIdentity: reviewIdentity,
            projection: projection,
            matchesSource: matchesSource
        )
    }

    /// A rename or copy identical to its source: there is nothing to diff, and callers must not re-load it as a fresh add.
    var isContentFreeRenameOrCopy: Bool {
        hunkType.hasSource && matchesSource
    }

    /// Paths a file action on this card must touch: a rename's removed source travels with it, a copy's source is a separate card.
    var changedPaths: [String] {
        guard hunkType == .renamed, let oldPath else { return [path] }
        return [path, oldPath]
    }
}
