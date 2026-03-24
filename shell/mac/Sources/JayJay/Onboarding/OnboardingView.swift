import AppKit
import SwiftUI

struct OnboardingView: View {
    let onContinue: () -> Void
    @State private var jjStatus: JJEnvironment.Status?
    @State private var currentPage = 0

    var body: some View {
        VStack(spacing: 0) {
            TabView(selection: $currentPage) {
                welcomePage.tag(0)
                jjCheckPage.tag(1)
                readyPage.tag(2)
            }
            .tabViewStyle(.automatic)

            // Navigation
            HStack {
                if currentPage > 0 {
                    Button("Back") { withAnimation { currentPage -= 1 } }
                }
                Spacer()
                pageIndicator
                Spacer()
                if currentPage < 2 {
                    Button("Next") { withAnimation { currentPage += 1 } }
                        .keyboardShortcut(.defaultAction)
                } else {
                    Button("Get Started") { onContinue() }
                        .keyboardShortcut(.defaultAction)
                        .buttonStyle(.borderedProminent)
                }
            }
            .padding(20)
        }
        .frame(width: 440, height: 420)
        .task { jjStatus = JJEnvironment.check() }
    }

    // MARK: - Pages

    private var welcomePage: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(nsImage: NSApplication.shared.applicationIconImage)
                .resizable()
                .interpolation(.high)
                .frame(width: 96, height: 96)
            Text("Welcome to JayJay")
                .jayjayFont(28, weight: .bold)
            Text(
                "A native GUI for Jujutsu version control.\nBrowse history, review diffs, and manage changes — all from one window."
            )
            .jayjayFont(14)
            .foregroundStyle(.secondary)
            .multilineTextAlignment(.center)
            .frame(maxWidth: 340)
            Spacer()
        }
        .padding(24)
    }

    private var jjCheckPage: some View {
        VStack(spacing: 16) {
            Spacer()
            if let status = jjStatus {
                if status.isInstalled {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.system(size: 48))
                        .foregroundStyle(.green)
                    Text("Jujutsu is installed")
                        .jayjayFont(22, weight: .semibold)
                    if let version = status.version {
                        Text(version)
                            .jayjayFont(13, design: .monospaced)
                            .foregroundStyle(.secondary)
                    }
                    if let path = status.path {
                        Text(path)
                            .jayjayFont(12, design: .monospaced)
                            .foregroundStyle(.tertiary)
                    }
                } else {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: 48))
                        .foregroundStyle(.orange)
                    Text("Jujutsu not found")
                        .jayjayFont(22, weight: .semibold)
                    Text("JayJay requires jj to be installed.\nInstall it with Homebrew or Cargo:")
                        .jayjayFont(14)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)

                    VStack(alignment: .leading, spacing: 8) {
                        installCommand("brew install jj")
                        installCommand("cargo install --locked jj-cli")
                    }
                    .padding(.top, 8)

                    Button("Check Again") { jjStatus = JJEnvironment.check() }
                        .padding(.top, 4)
                }
            } else {
                ProgressView("Checking for jj...")
            }
            Spacer()
        }
        .padding(24)
    }

    private var readyPage: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.green)
            Text("You're all set!")
                .jayjayFont(22, weight: .semibold)

            VStack(alignment: .leading, spacing: 10) {
                tip(icon: "folder", text: "Open any jj repository to get started")
                tip(icon: "keyboard", text: "⌘⇧P command palette, Space to review files")
                tip(icon: "arrow.triangle.branch", text: "Shift-click two commits to compare them")
                tip(icon: "sparkles", text: "AI commit messages via Codex, Claude, or Apple Intelligence")
                tip(icon: "exclamationmark.triangle", text: "Close GitHub Desktop — it may conflict with jj")
            }
            .padding(.top, 8)
            Spacer()
        }
        .padding(24)
    }

    // MARK: - Helpers

    private var pageIndicator: some View {
        HStack(spacing: 6) {
            ForEach(0 ..< 3, id: \.self) { i in
                Circle()
                    .fill(i == currentPage ? Color.accentColor : Color.secondary.opacity(0.3))
                    .frame(width: 6, height: 6)
            }
        }
    }

    private func installCommand(_ cmd: String) -> some View {
        HStack {
            Text(cmd)
                .jayjayFont(13, design: .monospaced)
                .textSelection(.enabled)
            Spacer()
            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(cmd, forType: .string)
            } label: {
                Image(systemName: "doc.on.doc")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(10)
        .background(Color.primary.opacity(0.05), in: RoundedRectangle(cornerRadius: 8))
    }

    private func tip(icon: String, text: String) -> some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .frame(width: 20)
                .foregroundStyle(Color.accentColor)
            Text(text)
                .jayjayFont(13)
        }
    }
}
