import SwiftUI

struct LoadingStatus: View {
    let label: String

    var body: some View {
        HStack(spacing: 8) {
            ProgressView()
                .controlSize(.small)
            Text(label)
                .jayjayFont(12)
                .foregroundStyle(.secondary)
        }
        .accessibilityElement(children: .combine)
    }
}
