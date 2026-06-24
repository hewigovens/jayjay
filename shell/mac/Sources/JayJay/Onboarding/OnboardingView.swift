import AppKit
import SwiftUI

struct OnboardingView: View {
    static let preferredSize = NSSize(width: 480, height: 460)

    let onContinue: () -> Void
    @State private var jjStatus: JJEnvironment.Status?
    @State private var currentPage = OnboardingPage.welcome

    var body: some View {
        VStack(spacing: 0) {
            pageContent
                .frame(maxWidth: .infinity, maxHeight: .infinity)

            pageIndicator
                .padding(.bottom, 2)

            HStack {
                if let previousPage = currentPage.previous {
                    Button("Back") { currentPage = previousPage }
                }
                Spacer()
                if let nextPage = currentPage.next {
                    Button("Next") { currentPage = nextPage }
                        .keyboardShortcut(.defaultAction)
                } else {
                    Button("Get Started") { onContinue() }
                        .keyboardShortcut(.defaultAction)
                        .buttonStyle(.borderedProminent)
                }
            }
            .padding(20)
        }
        .frame(width: Self.preferredSize.width, height: Self.preferredSize.height)
        .task { jjStatus = JJEnvironment.check() }
    }

    // MARK: - Pages

    @ViewBuilder
    private var pageContent: some View {
        switch currentPage {
            case .welcome:
                welcomePage
            case .jjCheck:
                jjCheckPage
            case .ready:
                readyPage
        }
    }

    private var pageIndicator: some View {
        HStack(spacing: 8) {
            ForEach(OnboardingPage.allCases) { page in
                pageIndicatorButton(for: page)
            }
        }
        .accessibilityElement(children: .contain)
    }

    private func pageIndicatorButton(for page: OnboardingPage) -> some View {
        let isCurrentPage = page == currentPage

        return Circle()
            .fill(isCurrentPage ? Color.accentColor : Color.primary.opacity(0.22))
            .frame(width: isCurrentPage ? 8 : 6, height: isCurrentPage ? 8 : 6)
            .frame(width: 18, height: 18)
            .contentShape(Rectangle())
            .onTapGesture {
                currentPage = page
            }
            .accessibilityAddTraits(.isButton)
            .accessibilityLabel(page.title)
            .accessibilityValue(isCurrentPage ? "Current page" : "")
            .accessibilityAction {
                currentPage = page
            }
            .help(page.title)
    }

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
                tip(icon: "keyboard", text: "⌘⇧P finds actions; type help split for feature help")
                tip(icon: "checkmark.circle", text: "Press Space to mark the selected file reviewed")
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

private enum OnboardingPage: Int, CaseIterable, Identifiable {
    case welcome
    case jjCheck
    case ready

    var id: Int {
        rawValue
    }

    var title: String {
        switch self {
            case .welcome:
                "Welcome"
            case .jjCheck:
                "Jujutsu Check"
            case .ready:
                "Ready"
        }
    }

    var previous: OnboardingPage? {
        OnboardingPage(rawValue: rawValue - 1)
    }

    var next: OnboardingPage? {
        OnboardingPage(rawValue: rawValue + 1)
    }
}
