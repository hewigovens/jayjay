import AppKit
import Foundation
import SwiftUI

struct AboutView: View {
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        ZStack {
            backgroundGradient.ignoresSafeArea()

            VStack(spacing: 22) {
                Image(nsImage: NSApplication.shared.applicationIconImage)
                    .resizable()
                    .interpolation(.high)
                    .frame(width: 118, height: 118)
                    .shadow(color: Color.black.opacity(0.18), radius: 18, y: 10)

                VStack(spacing: 8) {
                    Text(AppMetadata.appName)
                        .jayjayFont(30, weight: .bold)
                    Text(AppMetadata.tagline)
                        .jayjayFont(14, weight: .medium)
                        .foregroundStyle(Color.secondary)
                    Text(AppMetadata.compactVersionLabel)
                        .jayjayFont(13, weight: .semibold, design: .monospaced)
                        .foregroundStyle(Color.primary.opacity(0.76))
                }

                VStack(spacing: 10) {
                    Text(sponsorAttributedText)
                        .jayjayFont(13, weight: .medium)
                        .multilineTextAlignment(.center)
                        .frame(maxWidth: 320)
                }
                .padding(18)
                .frame(maxWidth: .infinity)
                .background(
                    RoundedRectangle(cornerRadius: 24, style: .continuous)
                        .fill(cardFill)
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 24, style: .continuous)
                        .stroke(cardStroke, lineWidth: 1)
                )
            }
            .padding(28)
            .frame(width: 420)
        }
        .frame(width: 420, height: 460)
    }

    private var backgroundGradient: LinearGradient {
        if colorScheme == .dark {
            LinearGradient(
                colors: [
                    Color(red: 0.12, green: 0.16, blue: 0.28),
                    Color(red: 0.14, green: 0.22, blue: 0.4),
                    Color(red: 0.11, green: 0.15, blue: 0.26)
                ],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        } else {
            LinearGradient(
                colors: [
                    Color(red: 0.95, green: 0.98, blue: 1.0),
                    Color(red: 0.84, green: 0.91, blue: 1.0),
                    Color(red: 0.73, green: 0.82, blue: 0.99)
                ],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        }
    }

    private var cardFill: Color {
        colorScheme == .dark ? Color.white.opacity(0.08) : Color.white.opacity(0.72)
    }

    private var cardStroke: Color {
        colorScheme == .dark ? Color.white.opacity(0.14) : Color.white.opacity(0.78)
    }

    private var sponsorAttributedText: AttributedString {
        let string = NSMutableAttributedString(
            string: "Love JayJay ❤️ on GitHub Sponsors"
        )

        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.alignment = .center

        string.addAttributes(
            [
                .foregroundColor: NSColor.labelColor.withAlphaComponent(0.82),
                .paragraphStyle: paragraphStyle
            ],
            range: NSRange(location: 0, length: string.length)
        )

        let sponsorRange = (string.string as NSString).range(of: "GitHub Sponsors")
        string.addAttributes(
            [
                .link: AppMetadata.sponsorURL,
                .foregroundColor: NSColor.systemBlue
            ],
            range: sponsorRange
        )

        return AttributedString(string)
    }
}
