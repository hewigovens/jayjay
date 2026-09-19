import SwiftUI

struct DescriptionExpansionToggle: View {
    let expanded: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: expanded ? "arrow.down.and.line.horizontal.and.arrow.up" : "arrow.up.and.line.horizontal.and.arrow.down")
                .jayjayFont(12, weight: .semibold)
        }
        .buttonStyle(.plain)
        .foregroundStyle(.secondary)
        .keyboardFocusStop(.expandDescription, action: action)
        .accessibilityIdentifier(AID.Detail.descriptionExpansion)
        .accessibilityLabel(expanded ? "Collapse description" : "Expand description")
        .help(expanded ? "Collapse description" : "Expand description")
    }
}
