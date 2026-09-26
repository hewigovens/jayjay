import SwiftUI

struct OverviewPanel<Content: View>: View {
    let accessibilityIdentifier: String
    @ViewBuilder let content: Content

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                content
            }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .frame(width: 400)
        .accessibilityIdentifier(accessibilityIdentifier)
    }
}

struct OverviewPanelHeader<Title: View>: View {
    let onClose: () -> Void
    @ViewBuilder let title: Title

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            title
                .jayjayFont(13, weight: .semibold)
                .frame(maxWidth: .infinity, alignment: .leading)
            Button(action: onClose) {
                Image(systemName: "xmark")
                    .jayjayFont(10, weight: .semibold)
            }
            .buttonStyle(.plain)
            .foregroundStyle(.secondary)
            .help("Close")
        }
    }
}

struct OverviewPanelSection<Content: View>: View {
    let title: String
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .jayjayFont(10, weight: .semibold)
                .foregroundStyle(.tertiary)
                .textCase(.uppercase)
            content
        }
    }
}

struct OverviewMetaLine: View {
    let parts: [(text: String, color: Color)]

    var body: some View {
        FlowLayout(spacing: 5, lineSpacing: 2) {
            ForEach(Array(parts.enumerated()), id: \.offset) { index, part in
                if index > 0 {
                    Text("·").foregroundStyle(.tertiary)
                }
                Text(part.text).foregroundStyle(part.color)
            }
        }
        .jayjayFont(11)
    }
}
