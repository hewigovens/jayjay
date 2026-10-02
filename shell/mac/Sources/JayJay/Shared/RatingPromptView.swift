import SwiftUI

struct RatingPromptView: View {
    let onDismiss: () -> Void
    let onDontShowAgain: () -> Void

    @Environment(\.openURL) private var openURL

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "heart.circle.fill")
                .font(.system(size: 40))
                .foregroundStyle(.pink)

            Text("Enjoying JayJay?")
                .jayjayFont(18, weight: .bold)

            Text("If JayJay is useful to you, a star on GitHub helps others find it, or sponsoring supports development.")
                .jayjayFont(13)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .lineSpacing(2)
                .fixedSize(horizontal: false, vertical: true)

            HStack(spacing: 12) {
                Button("Star on GitHub") { open(AppMetadata.githubURL) }
                    .buttonStyle(.borderedProminent)
                    .keyboardShortcut(.defaultAction)
                Button("Sponsor") { open(AppMetadata.sponsorURL) }
            }
            .padding(.top, 4)

            Button("Don't show again") { onDontShowAgain() }
                .buttonStyle(.plain)
                .jayjayFont(11)
                .foregroundStyle(.tertiary)
        }
        .padding(28)
        .frame(width: 320)
        .background(
            Button("") { onDismiss() }
                .keyboardShortcut(.cancelAction)
                .hidden()
        )
    }

    private func open(_ url: URL) {
        openURL(url)
        onDismiss()
    }
}
