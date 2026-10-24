import JayJayCore
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

            Text(Localized.Repo.enjoyingJayjay)
                .jayjayFont(18, weight: .bold)

            Text(Localized.Repo.ratingPromptDetail)
                .jayjayFont(13)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .lineSpacing(2)
                .fixedSize(horizontal: false, vertical: true)

            HStack(spacing: 12) {
                Button(Localized.Settings.starOnGithub) { open(AppMetadata.githubURL) }
                    .buttonStyle(.borderedProminent)
                    .keyboardShortcut(.defaultAction)
                Button(Localized.Settings.sponsor) { open(AppMetadata.sponsorURL) }
            }
            .padding(.top, 4)

            Button(Localized.Repo.dontShowAgain) { onDontShowAgain() }
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
