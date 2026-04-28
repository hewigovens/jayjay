import JayJayCore
import SwiftUI

/// Pure display helpers for EvologView — formatters, label/icon mappings.
enum EvologDisplay {
    static func timestamp(_ millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }

    /// Shorten verbose jj operation strings for display. Falls back to the raw value.
    static func operationLabel(_ raw: String) -> String {
        if raw.hasPrefix("snapshot working copy") { return "snapshot" }
        if raw.hasPrefix("describe commit ") { return "describe" }
        if raw.hasPrefix("rebase commit ") { return "rebase" }
        if raw.hasPrefix("squash commits ") { return "squash" }
        if raw.hasPrefix("split commit ") { return "split" }
        if raw.hasPrefix("new empty commit") { return "new" }
        return raw.isEmpty ? "rewrite" : raw
    }

    static func operationIcon(_ raw: String) -> String {
        switch operationLabel(raw) {
            case "snapshot": "camera"
            case "describe": "text.cursor"
            case "rebase": "arrow.uturn.up"
            case "squash": "arrow.down.left.circle"
            case "split": "rectangle.split.2x1"
            case "new": "plus.circle"
            default: "circle.dotted"
        }
    }

    static func hunkIcon(_ type: HunkType) -> String {
        switch type {
            case .added: "plus.circle"
            case .removed: "minus.circle"
            case .renamed: "arrow.right.circle"
            case .modified: "pencil.circle"
        }
    }

    static func hunkColor(_ type: HunkType) -> Color {
        switch type {
            case .added: .green
            case .removed: .red
            case .renamed: .blue
            case .modified: .orange
        }
    }
}

extension EvologEntry {
    /// Synthesize a ChangeInfo for an evolog entry whose interdiff against head is empty.
    func asPlaceholderInfo() -> ChangeInfo {
        ChangeInfo(
            changeId: changeId,
            commitId: commitId,
            description: description,
            author: "",
            email: "",
            timestampMillis: timestampMillis,
            parents: [],
            bookmarks: [],
            isWorkingCopy: false,
            hasConflict: false,
            isEmpty: false,
            isImmutable: false,
            isDivergent: false
        )
    }
}
