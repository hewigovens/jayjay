import SwiftUI

struct FileCollapseToggle: View {
    let path: String
    let isCollapsed: Bool
    /// The space from the glyph back to its row's leading edge, which the click target extends across.
    let leadingReach: CGFloat
    var rowHeight = IconButtonStyle.minimumSize
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: isCollapsed ? "chevron.right" : "chevron.down")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .frame(width: Self.width)
                .padding(.leading, leadingReach)
                .padding(.trailing, Self.trailingReach)
        }
        .iconButtonStyle(minimumHeight: rowHeight)
        .padding(.leading, -leadingReach)
        .padding(.trailing, -Self.trailingReach)
        .accessibilityIdentifier(AID.Diff.fileToggle(path))
        .accessibilityValue(isCollapsed ? "collapsed" : "expanded")
    }

    private static let width: CGFloat = 12
    private static let trailingReach = (IconButtonStyle.minimumSize - width) / 2
}
