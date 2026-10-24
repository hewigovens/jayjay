import AppKit
import JayJayCore
import SwiftUI

struct OnboardingView: View {
    static let preferredSize = NSSize(width: 480, height: 460)

    let onContinue: () -> Void
    @Environment(AppSettings.self) private var settings
    @State private var jjStatus: JJEnvironment.Status?
    @State private var currentPage = OnboardingPage.welcome
    @State private var identity: UserIdentity?
    @State private var isSettingUpIdentity = false
    @State private var isIdentitySkipped = false

    var body: some View {
        VStack(spacing: 0) {
            pageContent
                .frame(maxWidth: .infinity, maxHeight: .infinity)

            pageIndicator
                .padding(.bottom, 2)

            HStack {
                if let previousPage = currentPage.previous {
                    Button(Localized.Onboarding.back) { currentPage = previousPage }
                }
                Spacer()
                if let nextPage = currentPage.next {
                    Button(Localized.Onboarding.next) { currentPage = nextPage }
                        .keyboardShortcut(.defaultAction)
                } else {
                    Button(Localized.Onboarding.getStarted) { onContinue() }
                        .keyboardShortcut(.defaultAction)
                        .buttonStyle(.borderedProminent)
                }
            }
            .padding(20)
        }
        .frame(width: Self.preferredSize.width, height: Self.preferredSize.height)
        .task {
            identity = await UserIdentity.configured()
            jjStatus = JJEnvironment.check()
        }
        .identitySetup($identity, isPresented: $isSettingUpIdentity)
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
            .accessibilityValue(isCurrentPage ? Localized.Onboarding.currentPage : "")
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
            Text(Localized.Menu.welcomeToJayjay)
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
                    Text(Localized.Onboarding.jujutsuIsInstalled)
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
                    identityLine
                        .padding(.top, 8)
                } else {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: 48))
                        .foregroundStyle(.orange)
                    Text(Localized.Onboarding.jujutsuNotFound)
                        .jayjayFont(22, weight: .semibold)
                    Text(Localized.Onboarding.requiresJj(manager: "Homebrew"))
                        .jayjayFont(14)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)

                    VStack(alignment: .leading, spacing: 8) {
                        installCommand("brew install jj")
                        installCommand("cargo install --locked jj-cli")
                    }
                    .padding(.top, 8)

                    Button(Localized.Onboarding.checkAgain) { jjStatus = JJEnvironment.check() }
                        .padding(.top, 4)
                }
            } else {
                ProgressView(Localized.Onboarding.checkingForJjEllipsis)
            }
            Spacer()
        }
        .padding(24)
    }

    @ViewBuilder
    private var identityLine: some View {
        if let identity {
            Text(Localized.Onboarding.identity(identity: identity.label))
                .jayjayFont(13)
                .foregroundStyle(.secondary)
        } else if isIdentitySkipped {
            Text(Localized.Onboarding.youCanSetYourJjIdentity)
                .jayjayFont(13)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true)
        } else {
            HStack(spacing: 8) {
                Text(Localized.Onboarding.identityNotSet)
                    .jayjayFont(13)
                    .foregroundStyle(.secondary)
                Button(Localized.Common.setUpEllipsis) { isSettingUpIdentity = true }
                Button(Localized.Repo.skip) { isIdentitySkipped = true }
            }
        }
    }

    private var readyPage: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.green)
            Text(Localized.Onboarding.youreAllSetSentence)
                .jayjayFont(22, weight: .semibold)

            VStack(alignment: .leading, spacing: 10) {
                tip(icon: "folder", text: Localized.Onboarding.openAnyJjRepositoryToGet)
                tip(icon: "keyboard", text: Localized.Onboarding.paletteTip(shortcut: AppShortcut.commandPalette.symbol))
                tip(icon: "checkmark.circle", text: Localized.Onboarding.pressSpaceToMarkTheSelected)
                tip(icon: "arrow.triangle.branch", text: Localized.Onboarding.shiftClickSelectsARangeClick)
                tip(icon: "exclamationmark.triangle", text: Localized.Onboarding.closeGithubDesktopItMayConflict)
                statsToggle
                    .padding(.top, 8)
            }
            .padding(.top, 8)
            Spacer()
        }
        .padding(24)
    }

    private var statsToggle: some View {
        HStack(spacing: 10) {
            Toggle(Localized.Telemetry.optIn, isOn: Binding(
                get: { settings.sendsAnonymousStats },
                set: { settings.sendsAnonymousStats = $0 }
            ))
            .labelsHidden()
            .frame(width: 20)
            Text(Localized.Telemetry.optIn)
                .jayjayFont(13)
                .onTapGesture { settings.sendsAnonymousStats.toggle() }
        }
        .help(Localized.Telemetry.optInDetail)
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
                Localized.Onboarding.welcome
            case .jjCheck:
                Localized.Onboarding.jujutsuCheck
            case .ready:
                Localized.Onboarding.ready
        }
    }

    var previous: OnboardingPage? {
        OnboardingPage(rawValue: rawValue - 1)
    }

    var next: OnboardingPage? {
        OnboardingPage(rawValue: rawValue + 1)
    }
}
