import JayJayCore
import SwiftUI

struct DescriptionPreview: View {
    let description: String
    let collapsedHeight: CGFloat
    let expandedHeight: CGFloat
    let expanded: Bool
    let onEdit: (() -> Void)?
    let onToggleExpansion: () -> Void

    var body: some View {
        let title = commitSummary(message: description)
        let details = commitBody(message: description)
        VStack(alignment: .leading, spacing: details.isEmpty ? 0 : 12) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(title)
                    .jayjayFont(.title)
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
            .frame(maxWidth: .infinity, alignment: .leading)
            if !details.isEmpty {
                DescriptionBodyPreview(
                    text: details, collapsedHeight: collapsedHeight, expandedHeight: expandedHeight, expanded: expanded
                )
            }
        }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.Detail.description)
    }
}
