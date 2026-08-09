import SwiftUI

struct LoadingHUD: View {
    let accessibilityIdentifier: String

    var body: some View {
        ZStack {
            Color.clear
            HStack(spacing: 10) {
                ProgressView()
                    .controlSize(.small)
                Text("Loading...")
                    .jayjayFont(12, weight: .medium)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            .shadow(color: .black.opacity(0.14), radius: 8, y: 3)
        }
        .contentShape(Rectangle())
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Loading")
        .accessibilityIdentifier(accessibilityIdentifier)
    }
}
