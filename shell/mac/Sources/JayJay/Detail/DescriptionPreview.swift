import JayJayCore
import SwiftUI

struct DescriptionPreview: View {
    let description: String
    let maximumHeight: CGFloat
    let expanded: Bool
    let onEdit: (() -> Void)?
    let onOverflowChanged: (Bool) -> Void
    @Environment(\.jayjayFontSize) private var baseFontSize

    var body: some View {
        let title = commitSummary(message: description)
        let details = commitBody(message: description)
        VStack(alignment: .leading, spacing: details.isEmpty ? 0 : 6) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(title)
                    .font(.system(size: 14 * baseFontSize / 12, weight: .semibold))
                    .textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier(AID.Detail.descriptionTitle)
                if let onEdit {
                    Button(action: onEdit) {
                        Label {
                            Text("Edit").font(.system(size: baseFontSize))
                        } icon: {
                            Image(systemName: "pencil")
                                .font(.system(size: 14 * baseFontSize / 12, weight: .semibold))
                        }
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(.secondary)
                    .fixedSize()
                    .accessibilityLabel("Edit description")
                    .help("Edit description")
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            DescriptionBodyPreview(
                text: details, maximumHeight: maximumHeight, expanded: expanded,
                onOverflowChanged: onOverflowChanged
            )
            .accessibilityHidden(details.isEmpty)
        }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.Detail.description)
    }
}
