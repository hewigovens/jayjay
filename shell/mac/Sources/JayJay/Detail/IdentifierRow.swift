import SwiftUI

struct IdentifierRow<Content: View>: View {
    let label: String
    @ViewBuilder let content: () -> Content

    var body: some View {
        GridRow(alignment: .firstTextBaseline) {
            Text(label)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .gridColumnAlignment(.trailing)
            HStack(alignment: .firstTextBaseline, spacing: 6, content: content)
        }
    }
}
