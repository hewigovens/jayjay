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

    var label: String {
        switch self {
            case .added: "Added"
            case .removed: "Removed"
            case .modified: "Modified"
            case .renamed: "Renamed"
        }
    }
}
