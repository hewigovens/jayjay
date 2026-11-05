import JayJayCore
import SwiftUI

extension HunkType {
    @ViewBuilder var icon: some View {
        switch self {
            case .added: Image(systemName: "plus.circle.fill")
            case .removed: Image(systemName: "minus.circle.fill")
            case .modified: Self.circled(.plusMinus)
            case .renamed: Image(systemName: "arrow.right.circle.fill")
            case .copied: Self.circled(.copy)
        }
    }

    /// Matches the plus in plus.circle.fill.
    private static func circled(_ glyph: StatusGlyphShape.Glyph) -> some View {
        Image(systemName: "circle.fill")
            .overlay {
                StatusGlyphShape(glyph: glyph, gridFraction: 0.727)
                    .blendMode(.destinationOut)
            }
            .compositingGroup()
    }

    func badge(_ colorScheme: ColorScheme, colors: AppColors) -> some View {
        let iconColor = colors.fileColor(for: self)
        return Image(systemName: "square.fill")
            .foregroundStyle(iconColor.opacity(colorScheme == .light ? 0.15 : 0.14))
            .overlay {
                badgeGlyph
                    .foregroundStyle(colors.fileBadgeColor(for: self))
            }
    }

    @ViewBuilder private var badgeGlyph: some View {
        switch self {
            case .added: badgeSymbol("plus")
            case .removed: badgeSymbol("minus")
            case .modified: StatusGlyphShape(glyph: .plusMinus, gridFraction: Self.badgeGridFraction)
            case .renamed: badgeSymbol("arrow.right")
            case .copied: StatusGlyphShape(glyph: .copy, gridFraction: Self.badgeGridFraction)
        }
    }

    /// square.fill's ink is 84% of its frame.
    private static let badgeGridFraction: CGFloat = 0.837

    private func badgeSymbol(_ name: String) -> some View {
        Image(systemName: name)
            .fontWeight(.semibold)
            .scaleEffect(0.6)
    }

    var label: String {
        switch self {
            case .added: Localized.Diff.added
            case .removed: Localized.Diff.removed
            case .modified: Localized.Repo.modified
            case .renamed: Localized.Diff.renamed
            case .copied: Localized.Diff.copied
        }
    }
}
