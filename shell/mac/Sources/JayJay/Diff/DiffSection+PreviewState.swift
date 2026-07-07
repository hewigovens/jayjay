import JayJayCore

extension DiffSection {
    var fileDiff: FileDiff? {
        loadedDiff?.fileDiff
    }

    var displayGroups: [ChangeGroup]? {
        loadedDiff?.displayGroups
    }

    var loadedDisplayLines: [DiffLine]? {
        loadedDiff?.displayLines
    }

    var loadedPath: String? {
        loadedDiff?.path
    }

    var loadedOldContent: String? {
        loadedDiff?.content.oldContent
    }

    var loadedNewContent: String? {
        loadedDiff?.content.newContent
    }

    var loadedOldPreview: DiffPreview? {
        loadedDiff?.content.oldPreview
    }

    var loadedNewPreview: DiffPreview? {
        loadedDiff?.content.newPreview
    }

    var loadedProjection: DiffProjection? {
        loadedDiff?.content.projection
    }

    var isSvgFile: Bool {
        hunk.path.lowercased().hasSuffix(".svg")
    }

    var isMarkdownFile: Bool {
        let path = hunk.path.lowercased()
        return path.hasSuffix(".md") || path.hasSuffix(".markdown")
    }

    var isHTMLFile: Bool {
        let path = hunk.path.lowercased()
        return path.hasSuffix(".html") || path.hasSuffix(".htm")
    }

    var canRenderMarkdownPreview: Bool {
        (canRenderMarkdownFilePreview && activeMarkdownRichView) || canRenderProjectionAsMarkdown
    }

    var canRenderMarkdownFilePreview: Bool {
        isMarkdownFile && hunk.projection == nil
    }

    var canOpenHTMLExternally: Bool {
        Self.canOpenHTMLExternally(
            isHTMLFile: isHTMLFile,
            hasProjection: hunk.projection != nil,
            repoPath: repo?.path(),
            path: hunk.path
        )
    }

    static func canOpenHTMLExternally(
        isHTMLFile: Bool,
        hasProjection: Bool,
        repoPath: String?,
        path: String
    ) -> Bool {
        guard isHTMLFile, !hasProjection, let repoPath else { return false }
        return RepositoryActions.existingRepoFileURL(repoPath: repoPath, path: path) != nil
    }

    var canRenderProjectionAsMarkdown: Bool {
        guard activeProjectionRichView, effectiveProjection?.mode == .processed else { return false }
        switch effectiveProjection?.renderKind {
            case .markdown, .table:
                return true
            case .text, .none:
                return false
        }
    }

    var shouldShowProjectionBanner: Bool {
        guard let projection = effectiveProjection else { return false }
        return DiffProjectionDisplayPolicy.showsBanner(
            for: projection,
            richView: activeProjectionRichView
        )
    }

    var effectiveProjection: DiffProjection? {
        loadedPath == hunk.path ? loadedProjection : hunk.projection
    }

    var shouldShowProjectionToggle: Bool {
        guard let projection = effectiveProjection else { return false }
        return !DiffProjectionDisplayPolicy.opensAutomatically(projection)
    }

    var projectionRequestMode: DiffProjectionMode? {
        DiffProjectionDisplayPolicy.requestMode(
            for: effectiveProjection,
            richView: activeProjectionRichView
        )
    }

    var activeSvgRichView: Bool {
        richPreviewSelection?.isActive(.svg, path: hunk.path) ?? false
    }

    var activeMarkdownRichView: Bool {
        richPreviewSelection?.isActive(.markdown, path: hunk.path) ?? false
    }

    var activeProjectionRichView: Bool {
        richPreviewSelection?.isActive(.projection, path: hunk.path) ?? false
    }

    var projectionModeKey: String {
        switch projectionRequestMode {
            case .some(.raw): "raw"
            case .some(.processed): "processed"
            case .none: "none"
        }
    }

    func resetRichViewState() {
        richPreviewSelection = nil
    }

    func toggleSvgRichView() {
        toggleRichView(.svg)
    }

    func toggleMarkdownRichView() {
        toggleRichView(.markdown)
    }

    func openHTMLExternally() {
        guard let repo else { return }
        RepositoryActions.openInDefaultApp(repoPath: repo.path(), path: hunk.path)
    }

    func toggleProjectionRichView() {
        toggleRichView(.projection)
    }

    private func toggleRichView(_ kind: DiffRichPreviewSelection.Kind) {
        if richPreviewSelection?.isActive(kind, path: hunk.path) == true {
            richPreviewSelection = nil
        } else {
            richPreviewSelection = DiffRichPreviewSelection(kind: kind, path: hunk.path)
        }
    }
}
