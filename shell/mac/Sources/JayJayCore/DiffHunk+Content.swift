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
        projection: DiffProjection?
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
            projection: projection
        )
    }

    /// A byte-identical rename: the core cleared both sides because the content is unchanged, so there is nothing to diff and callers must not re-load it as a fresh add.
    var isContentFreeRename: Bool {
        hunkType == .renamed
            && oldContent == nil && newContent == nil
            && oldPreview == nil && newPreview == nil
    }
}
