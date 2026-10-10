import JayJayCore
import SwiftUI

extension HunkType {
    @ViewBuilder var icon: some View {
        switch self {
            case .added: Image(systemName: "plus.circle.fill")
            case .removed: Image(systemName: "minus.circle.fill")
            case .modified: Image(systemName: "plusminus.circle.fill")
            case .renamed: Image(systemName: "arrow.right.circle.fill")
            case .copied:
                // SF Symbols has no circled copy glyph.
                Image(systemName: "circle.fill")
                    .overlay {
                        Image(systemName: "square.on.square")
                            .fontWeight(.bold)
                            .scaleEffect(0.55)
                            .blendMode(.destinationOut)
                    }
                    .compositingGroup()
        }
    }

    func badge(_ colorScheme: ColorScheme) -> some View {
        Image(systemName: "square.fill")
            .foregroundStyle(iconColor.opacity(colorScheme == .light ? 0.15 : 0.14))
            .overlay {
                Image(systemName: badgeSymbol)
                    .fontWeight(.semibold)
                    .scaleEffect(0.6)
                    .foregroundStyle(colorScheme == .light ? lightBadgeColor : iconColor)
            }
    }

    private var lightBadgeColor: Color {
        switch self {
            case .added, .copied: Color(red: 28 / 255, green: 124 / 255, blue: 58 / 255)
            case .removed: Color(red: 180 / 255, green: 35 / 255, blue: 24 / 255)
            case .modified, .renamed: Color(red: 23 / 255, green: 92 / 255, blue: 211 / 255)
        }
    }

    private var badgeSymbol: String {
        switch self {
            case .added: "plus"
            case .removed: "minus"
            case .modified: "plusminus"
            case .renamed: "arrow.right"
            case .copied: "square.on.square"
        }
    }

    var iconColor: Color {
        switch self {
            case .added, .copied: .green
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
            case .copied: "Copied"
        }
    }
}
