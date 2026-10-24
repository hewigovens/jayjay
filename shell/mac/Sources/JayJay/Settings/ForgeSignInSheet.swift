import AppKit
import JayJayCore
import SwiftUI

struct ForgeSignInSheet: View {
    let host: RepoHost
    let onDismiss: () -> Void
    let onSignedIn: () -> Void
    @State private var signIn: HostSignIn?
    @State private var error: String?
    @State private var token = JayJaySyncToken.standalone()

    var body: some View {
        SheetContainer(
            title: Localized.Settings.signInTitle(host: repoHostName(host: host)),
            cancelLabel: Localized.Common.cancel,
            confirmLabel: Localized.Settings.copyCodeAndOpen,
            confirmDisabled: signIn == nil,
            confirmAccessibilityIdentifier: AID.Settings.forgeSignInOpen,
            onCancel: cancel,
            onConfirm: openBrowser,
            content: {
                Text(Localized.Settings.signInDetail)
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                if let signIn {
                    Text(signIn.userCode())
                        .font(.system(size: 22, weight: .semibold, design: .monospaced))
                        .textSelection(.enabled)
                        .accessibilityIdentifier(AID.Settings.forgeSignInCode)
                }
                if let error {
                    Text(error)
                        .jayjayFont(11)
                        .foregroundStyle(.red)
                        .fixedSize(horizontal: false, vertical: true)
                } else {
                    HStack(spacing: 6) {
                        ProgressView().controlSize(.small)
                        Text(signIn == nil ? Localized.Settings.requestingCode : Localized.Settings.waitingForApproval)
                            .jayjayFont(11)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        )
        .task { await run() }
        .onDisappear { token.cancel() }
    }

    private func run() async {
        let (host, token) = (host, token)
        do {
            let signIn = try await Task.detached(priority: .userInitiated) {
                try HostSignIn.start(host: host)
            }.value
            self.signIn = signIn
            try await Task.detached(priority: .userInitiated) {
                try signIn.wait(sync: token)
            }.value
            onSignedIn()
            onDismiss()
        } catch JayError.Canceled {
        } catch {
            self.error = error.friendlyDescription
        }
    }

    private func cancel() {
        token.cancel()
        onDismiss()
    }

    private func openBrowser() {
        guard let signIn, let url = URL(string: signIn.verificationUri()) else { return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(signIn.userCode(), forType: .string)
        NSWorkspace.shared.open(url)
    }
}

extension RepoHost: @retroactive Identifiable {
    public var id: Self {
        self
    }
}
