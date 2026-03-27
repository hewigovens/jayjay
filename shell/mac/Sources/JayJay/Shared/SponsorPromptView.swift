import SwiftUI

struct SponsorPromptView: View {
    let onDismiss: () -> Void
    let onDontShowAgain: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "heart.circle.fill")
                .font(.system(size: 40))
                .foregroundStyle(.pink)

            Text("Enjoying JayJay?")
                .jayjayFont(18, weight: .bold)

            Text("JayJay is 100% free and open source.\nIf it's useful to you, consider supporting development.")
                .jayjayFont(13)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .lineSpacing(2)

            HStack(spacing: 12) {
                Button("Maybe Later") { onDismiss() }
                    .keyboardShortcut(.cancelAction)
                Link(destination: AppMetadata.sponsorURL) {
                    Text("Sponsor on GitHub")
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
            }
            .padding(.top, 4)

            Button("Don't show again") { onDontShowAgain() }
                .buttonStyle(.plain)
                .jayjayFont(11)
                .foregroundStyle(.tertiary)
        }
        .padding(28)
        .frame(width: 320)
    }
}

struct SponsorPromptModifier: ViewModifier {
    private static let promptInterval = 20

    let signal: Int
    let settings: AppSettings
    @Binding var isPresented: Bool

    func body(content: Content) -> some View {
        content
            .onChange(of: signal) {
                settings.sponsorActionCount += 1
                if settings.sponsorActionCount >= settings.sponsorNextPromptCount,
                   !settings.sponsorDismissed,
                   !isPresented
                {
                    settings.sponsorNextPromptCount = settings.sponsorActionCount + Self.promptInterval
                    isPresented = true
                }
            }
            .sheet(isPresented: $isPresented) {
                SponsorPromptView(
                    onDismiss: { isPresented = false },
                    onDontShowAgain: {
                        settings.sponsorDismissed = true
                        isPresented = false
                    }
                )
            }
    }
}
