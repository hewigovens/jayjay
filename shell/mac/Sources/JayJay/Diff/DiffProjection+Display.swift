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
                return "Binary property list on disk, previewed as XML"
            }
            return "\(projection.pluginLabel) preview"
        }
        return "\(projection.pluginLabel) preview unavailable"
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
            case .some("delimited"): "Show table preview"
            case .some("ipynb"): "Show notebook preview"
            case .some("plist"): "Show property list preview"
            case .some("sarif"): "Show SARIF report preview"
            default: "Show rich preview"
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
