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

enum DiffProjectionView {
    case source
    case diff
    case rendered
}

enum DiffProjectionDisplayPolicy {
    static func opensAutomatically(_ projection: DiffProjection) -> Bool {
        projection.pluginId == "plist"
    }

    static func defaultView(for projection: DiffProjection) -> DiffProjectionView {
        switch projection.pluginId {
            case "plist", "ipynb": .diff
            case "sarif": .rendered
            default: .source
        }
    }

    static func showsSourceToggle(_ projection: DiffProjection?) -> Bool {
        projection?.pluginId == "ipynb"
    }

    static func requestMode(
        for projection: DiffProjection?,
        view: DiffProjectionView? = nil
    ) -> DiffProjectionMode? {
        guard let projection else { return nil }
        return (view ?? defaultView(for: projection)) == .source ? .raw : .processed
    }

    static func showsBanner(
        for projection: DiffProjection,
        view: DiffProjectionView?
    ) -> Bool {
        let hasDiagnostics = !projection.diagnostics.isEmpty
        let isProcessedPreview = projection.mode == .processed
        return hasDiagnostics || (isProcessedPreview && view != .source)
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

    static func activeHelp(for projection: DiffProjection?) -> String {
        showsSourceToggle(projection) ? Localized.Diff.showNotebookDiff : Localized.Diff.showSourceDiff
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
