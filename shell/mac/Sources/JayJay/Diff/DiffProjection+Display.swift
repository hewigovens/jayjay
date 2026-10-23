import JayJayCore

extension DiffProjection {
    func identityPart(mode: DiffProjectionMode? = nil) -> String {
        let activeMode = mode ?? self.mode
        return "\(pluginId):v\(pluginVersion):\(activeMode.identityKey)"
    }
}

extension DiffProjectionMode {
    var identityKey: String {
        switch self {
            case .raw: "raw"
            case .processed: "processed"
        }
    }
}

enum DiffProjectionDisplayPolicy {
    static func opensAutomatically(_ projection: DiffProjection) -> Bool {
        projection.pluginId == "plist"
    }

    static func requestMode(
        for projection: DiffProjection?,
        richView: Bool
    ) -> DiffProjectionMode? {
        guard let projection else { return nil }
        if opensAutomatically(projection) {
            return .processed
        }
        return richView ? .processed : .raw
    }

    static func showsBanner(
        for projection: DiffProjection,
        richView: Bool
    ) -> Bool {
        let hasDiagnostics = !projection.diagnostics.isEmpty
        let isProcessedPreview = projection.mode == .processed
        return hasDiagnostics || (isProcessedPreview && (richView || opensAutomatically(projection)))
    }

    static func title(for projection: DiffProjection) -> String {
        if projection.diagnostics.isEmpty {
            if projection.pluginId == "plist" {
                return Localized.Diff.binaryPropertyListOnDiskPreviewed
            }
            return Localized.Diff.projectionPreview(plugin: projection.pluginLabel)
        }
        return Localized.Diff.projectionPreviewUnavailable(plugin: projection.pluginLabel)
    }

    static func iconName(for projection: DiffProjection?) -> String {
        switch projection?.pluginId {
            case .some("delimited"): "tablecells"
            case .some("plist"): "doc.text"
            case .some("sarif"): "checklist"
            default: "sparkles"
        }
    }

    static func help(for projection: DiffProjection?) -> String {
        switch projection?.pluginId {
            case .some("delimited"): Localized.Diff.showTablePreview
            case .some("ipynb"): Localized.Diff.showNotebookPreview
            case .some("plist"): Localized.Diff.showPropertyListPreview
            case .some("sarif"): Localized.Diff.showSarifReportPreview
            default: Localized.Diff.showRichPreview
        }
    }
}

extension DiffRenderKind {
    var iconName: String {
        switch self {
            case .text: "doc.text"
            case .markdown: "text.alignleft"
            case .table: "tablecells"
        }
    }
}
