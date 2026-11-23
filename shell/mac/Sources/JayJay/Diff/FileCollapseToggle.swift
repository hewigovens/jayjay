import SwiftUI

struct FileCollapseToggle: View {
    let path: String
    let isCollapsed: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: isCollapsed ? "chevron.right" : "chevron.down")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .frame(width: 12)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier(AID.Diff.fileToggle(path))
        .accessibilityValue(isCollapsed ? "collapsed" : "expanded")
    }
}
