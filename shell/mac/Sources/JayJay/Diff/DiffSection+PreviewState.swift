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

    var canRenderHTMLPreview: Bool {
        canOpenHTMLExternally && activeHTMLRichView
    }

    /// Substring check on already-loaded content, not a disk read, so it's safe to compute on every render; `allowsContentJavaScript` stays false regardless.
    var htmlPreviewMayNeedScript: Bool {
        Self.htmlContentMayContainScript(loadedNewContent ?? hunk.newContent)
    }

    static func htmlContentMayContainScript(_ content: String?) -> Bool {
        guard let content else { return false }
        return content.lowercased().contains("<script")
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
        return DiffProjectionDisplayPolicy.showsBanner(for: projection, view: projectionView)
    }

    var effectiveProjection: DiffProjection? {
        loadedPath == hunk.path ? loadedProjection : hunk.projection
    }

    var shouldShowProjectionToggle: Bool {
        guard let projection = effectiveProjection else { return false }
        return !DiffProjectionDisplayPolicy.opensAutomatically(projection)
    }

    var projectionRequestMode: DiffProjectionMode? {
        DiffProjectionDisplayPolicy.requestMode(for: effectiveProjection, view: projectionView)
    }

    var projectionView: DiffProjectionView? {
        guard let projection = effectiveProjection else { return nil }
        switch selectedPreviewKind {
            case .projection: return .rendered
            case .source: return .source
            default: return DiffProjectionDisplayPolicy.defaultView(for: projection)
        }
    }

    private var selectedPreviewKind: DiffRichPreviewSelection.Kind? {
        richPreviewSelection?.path == hunk.path ? richPreviewSelection?.kind : nil
    }

    var activeSvgRichView: Bool {
        switch selectedPreviewKind {
            case .svg: true
            case .source: false
            default: svgOpensPreview
        }
    }

    /// Notes anchor to source lines, so a noted file stays on its source.
    private var svgOpensPreview: Bool {
        isSvgFile && loadedPath == hunk.path && loadedDiff?.opensSvgPreview == true
            && loadedReviewNoteSummaries().isEmpty
    }

    /// A conflicted or binary side would render as a broken image.
    nonisolated static func svgOpensPreview(oldContent: String?, newContent: String?) -> Bool {
        let sides = [oldContent, newContent].compactMap { $0 }.filter { !$0.isEmpty }
        return !sides.isEmpty && sides.allSatisfy { $0.contains("<svg") && !$0.contains("<<<<<<<") }
    }

    var activeMarkdownRichView: Bool {
        richPreviewSelection?.isActive(.markdown, path: hunk.path) ?? false
    }

    var activeHTMLRichView: Bool {
        richPreviewSelection?.isActive(.html, path: hunk.path) ?? false
    }

    var activeProjectionRichView: Bool {
        projectionView == .rendered
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
        toggleDefaultPreview(.svg, previewing: activeSvgRichView, opensPreview: svgOpensPreview)
    }

    func toggleMarkdownRichView() {
        toggleRichView(.markdown)
    }

    func toggleHTMLRichView() {
        toggleRichView(.html)
    }

    func openHTMLExternally() {
        guard let repo else { return }
        RepositoryActions.openInDefaultApp(repoPath: repo.path(), path: hunk.path)
    }

    func toggleProjectionRichView() {
        let opensRendered = effectiveProjection.map(DiffProjectionDisplayPolicy.defaultView) == .rendered
        toggleDefaultPreview(.projection, previewing: activeProjectionRichView, opensPreview: opensRendered)
    }

    func toggleProjectionSourceView() {
        toggleRichView(.source)
    }

    /// Turning off a default preview has to be remembered as choosing source.
    private func toggleDefaultPreview(
        _ kind: DiffRichPreviewSelection.Kind,
        previewing: Bool,
        opensPreview: Bool
    ) {
        if !previewing {
            richPreviewSelection = DiffRichPreviewSelection(kind: kind, path: hunk.path)
        } else if opensPreview {
            richPreviewSelection = DiffRichPreviewSelection(kind: .source, path: hunk.path)
        } else {
            richPreviewSelection = nil
        }
    }

    private func toggleRichView(_ kind: DiffRichPreviewSelection.Kind) {
        if richPreviewSelection?.isActive(kind, path: hunk.path) == true {
            richPreviewSelection = nil
        } else {
            richPreviewSelection = DiffRichPreviewSelection(kind: kind, path: hunk.path)
        }
    }
}
