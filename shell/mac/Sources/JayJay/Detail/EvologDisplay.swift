import JayJayCore
import SwiftUI

/// Presentation helpers for EvologView.
enum EvologDisplay {
    static func timestamp(_ millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }

    static func operationLabel(_ raw: String) -> String {
        switch evologOperationKind(raw: raw) {
            case .snapshot: String(localized: "snapshot")
            case .describe: String(localized: "describe")
            case .rebase: String(localized: "rebase")
            case .squash: String(localized: "squash")
            case .split: String(localized: "split")
            case .new: String(localized: "new")
            case .rewrite: String(localized: "rewrite")
            case .other: raw
        }
    }

    static func isSnapshot(_ raw: String) -> Bool {
        evologOperationKind(raw: raw) == .snapshot
    }

    static func operationIcon(_ raw: String) -> String {
        switch evologOperationKind(raw: raw) {
            case .snapshot: "camera"
            case .describe: "text.cursor"
            case .rebase: "arrow.uturn.up"
            case .squash: "arrow.down.left.circle"
            case .split: "rectangle.split.2x1"
            case .new: "plus.circle"
            case .rewrite, .other: "circle.dotted"
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
