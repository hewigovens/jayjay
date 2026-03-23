import AppKit
import SwiftUI

struct AboutView: View {
    var embedded = false
    var updater: SparkleUpdater?

    var body: some View {
        VStack(spacing: embedded ? 16 : 12) {
            Image(nsImage: NSApplication.shared.applicationIconImage)
                .resizable()
                .interpolation(.high)
                .frame(width: embedded ? 80 : 128, height: embedded ? 80 : 128)

            Text(AppMetadata.appName)
                .font(.system(size: embedded ? 20 : 16, weight: .bold))

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

            HStack(spacing: 8) {
                Text("Love JayJay?")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                Link(destination: AppMetadata.sponsorURL) {
                    Label("Sponsor", systemImage: "heart.fill")
                }
                .controlSize(.small)
            }
        }
        .padding(embedded ? 20 : 24)
        .frame(maxWidth: .infinity)
        .frame(width: embedded ? nil : 300)
        .fixedSize(horizontal: !embedded, vertical: true)
    }
}
