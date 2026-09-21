import JayJayCore
import SwiftUI

extension HunkType {
    @ViewBuilder var icon: some View {
        switch self {
            case .added: Image(systemName: "plus.circle.fill")
            case .removed: Image(systemName: "minus.circle.fill")
            case .modified:
                Image(systemName: "circle.fill")
                    .overlay {
                        Text("~")
                            .fontWeight(.bold)
                            .scaleEffect(0.8)
                            .blendMode(.destinationOut)
                    }
                    .compositingGroup()
            case .renamed: Image(systemName: "arrow.right.circle.fill")
        }
    }

    func badge(_ colorScheme: ColorScheme) -> some View {
        Image(systemName: "square.fill")
            .foregroundStyle(iconColor.opacity(colorScheme == .light ? 0.15 : 0.14))
            .overlay {
                Text(statusSymbol)
                    .fontWeight(.semibold)
                    .scaleEffect(0.75)
                    .foregroundStyle(colorScheme == .light ? lightBadgeColor : iconColor)
            }
    }

    private var lightBadgeColor: Color {
        switch self {
            case .added: Color(red: 28 / 255, green: 124 / 255, blue: 58 / 255)
            case .removed: Color(red: 180 / 255, green: 35 / 255, blue: 24 / 255)
            case .modified, .renamed: Color(red: 23 / 255, green: 92 / 255, blue: 211 / 255)
        }
    }

    private var statusSymbol: String {
        switch self {
            case .added: "+"
            case .removed: "−"
            case .modified: "~"
            case .renamed: "→"
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
