import JayJayCore
import SwiftUI

struct DescriptionPreview: View {
    @Environment(\.jayjayFontSize) private var baseFontSize
    let description: String
    let collapsedHeight: CGFloat
    let expandedHeight: CGFloat
    let expanded: Bool
    let onEdit: (() -> Void)?
    let onToggleExpansion: () -> Void

    var body: some View {
        let title = commitSummary(message: description)
        let details = commitBody(message: description)
        VStack(alignment: .leading, spacing: 0) {
            HStack(alignment: .center, spacing: 8) {
                Text(title)
                    .jayjayFont(20, weight: .semibold)
                    .textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier(AID.Detail.descriptionTitle)
                if let onEdit {
                    Button(action: onEdit) {
                        HStack(alignment: .firstTextBaseline, spacing: 4) {
                            Image(systemName: "pencil")
                                .jayjayFont(12, weight: .semibold)
                            Text("Edit").jayjayFont(12)
                        }
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(.secondary)
                    .fixedSize()
                    .keyboardFocusStop(.editDescription, action: onEdit)
                    .accessibilityLabel("Edit description")
                    .help("Edit description")
                }
                DescriptionExpansionToggle(expanded: expanded, action: onToggleExpansion)
            }
            .padding(.vertical, 6)
            .frame(maxWidth: .infinity, minHeight: PaneLayout.headerHeight, alignment: .leading)
            if !details.isEmpty {
                DescriptionBodyPreview(
                    text: details,
                    collapsedHeight: min(collapsedHeight, PaneLayout.fileRowHeight(baseFontSize: baseFontSize) - 12),
                    expandedHeight: expandedHeight, expanded: expanded
                )
                .padding(.vertical, 6)
                .frame(height: expanded ? nil : PaneLayout.fileRowHeight(baseFontSize: baseFontSize), alignment: .top)
            }
        }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.Detail.description)
    }
}
