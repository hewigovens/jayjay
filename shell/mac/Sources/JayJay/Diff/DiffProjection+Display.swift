import JayJayCore

extension DiffProjection {
    var iconName: String {
        switch pluginId {
            case "delimited": "tablecells"
            case "plist": "doc.text"
            case "sarif": "checklist"
            default: "sparkles"
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
