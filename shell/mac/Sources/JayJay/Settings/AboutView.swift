import AppKit
import SwiftUI

struct AboutView: View {
    var embedded = false
    var updater: SparkleUpdater?

    @State private var bouncing = false
    @State private var clickCount = 0
    @State private var showEasterEgg = false

    var body: some View {
        VStack(spacing: embedded ? 16 : 12) {
            Image(nsImage: NSApplication.shared.applicationIconImage)
                .resizable()
                .interpolation(.high)
                .frame(width: embedded ? 80 : 128, height: embedded ? 80 : 128)
                .scaleEffect(bouncing ? 1.15 : 1.0)
                .rotationEffect(.degrees(bouncing ? -8 : 0))
                .animation(.interpolatingSpring(stiffness: 300, damping: 8), value: bouncing)
                .onTapGesture {
                    bouncing = true
                    clickCount += 1
                    playChirp()
                    Task { try? await Task.sleep(for: .seconds(0.3))
                        bouncing = false
                    }
                    if clickCount >= 5 {
                        showEasterEgg = true
                        clickCount = 0
                    }
                }

            Text(AppMetadata.appName)
                .font(.system(size: embedded ? 20 : 16, weight: .bold))

            if showEasterEgg {
                Text("🐦‍⬛ Jay Jay Jay!")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(.blue)
                    .transition(.scale.combined(with: .opacity))
            }

            Text(AppMetadata.tagline)
                .font(.system(size: 12))
                .foregroundStyle(.secondary)

            Text(AppMetadata.detailedVersionLabel)
                .font(.system(size: 11))
                .foregroundStyle(.secondary)

            if embedded, let updater {
                Spacer()
                Toggle("Check for updates automatically", isOn: Binding(
                    get: { updater.autoChecksEnabled },
                    set: { updater.autoChecksEnabled = $0 }
                ))
                .toggleStyle(.switch)
                .controlSize(.small)
            }

            VStack(spacing: 6) {
                Text("Love JayJay?")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                HStack(spacing: 8) {
                    Link(destination: AppMetadata.sponsorURL) {
                        Label("Sponsor", systemImage: "heart.fill")
                    }
                    .controlSize(.small)
                    Link(destination: AppMetadata.githubURL) {
                        Label("Star on GitHub", systemImage: "star.fill")
                    }
                    .controlSize(.small)
                }
            }
        }
        .animation(.easeInOut(duration: 0.3), value: showEasterEgg)
        .padding(embedded ? 20 : 24)
        .frame(maxWidth: .infinity)
        .frame(width: embedded ? nil : 300)
        .fixedSize(horizontal: !embedded, vertical: true)
    }

    private func playChirp() {
        if let url = Bundle.main.url(forResource: "jay", withExtension: "aiff"),
           let sound = NSSound(contentsOf: url, byReference: true)
        {
            sound.play()
        } else {
            NSSound(named: NSSound.Name("Frog"))?.play()
        }
    }
}
