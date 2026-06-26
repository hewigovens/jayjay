import JayJayCore
import SwiftUI

extension HunkType {
    var iconName: String {
        switch self {
            case .added: "plus.circle.fill"
            case .removed: "minus.circle.fill"
            case .modified: "pencil.circle.fill"
            case .renamed: "arrow.right.circle.fill"
        }
    }

    var iconColor: Color {
        switch self {
            case .added: .green
            case .removed: .red
            case .modified: FileStatusColors.modified
            case .renamed: .blue
        }
    }
}

extension DiffHunk {
    /// A byte-identical rename: the core cleared both sides because the content is unchanged, so there is nothing to diff and we must not re-load it as a fresh add.
    var isContentFreeRename: Bool {
        hunkType == .renamed
            && oldContent == nil && newContent == nil
            && oldPreview == nil && newPreview == nil
    }
}
