import AppKit
import JayJayCore
import SwiftUI

struct FlockActivationSheet: View {
    let flock: FlockModel
    let onDismiss: () -> Void
    @State private var signIn: FlockSignIn?
    @State private var busy = false
    @State private var closed = false
    @State private var message: String?
    @State private var error: String?
    @State private var sync = JayJaySyncToken.standalone()

    var body: some View {
        SheetContainer(
            title: Localized.Settings.flockClaim,
            cancelLabel: Localized.Common.cancel,
            confirmLabel: signIn == nil ? Localized.Settings.flockVerify : Localized.Settings.copyCodeAndOpen,
            confirmDisabled: busy && signIn == nil,
            confirmAccessibilityIdentifier: AID.Settings.flockVerify,
            onCancel: cancel,
            onConfirm: {
                if let signIn, let url = URL(string: signIn.verificationUri()) {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(signIn.userCode(), forType: .string)
                    NSWorkspace.shared.open(url)
                } else {
                    Task { await verify() }
                }
            },
            content: {
                Text(Localized.Settings.flockVerifyDetail)
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                Text(Localized.Settings.flockPrivacyDetail)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                if let signIn {
                    Text(signIn.userCode())
                        .font(.system(size: 22, weight: .semibold, design: .monospaced))
                        .textSelection(.enabled)
                }
                if busy {
                    HStack(spacing: 6) {
                        ProgressView().controlSize(.small)
                        Text(signIn == nil ? Localized.Settings.flockChecking : Localized.Settings.waitingForApproval)
                            .jayjayFont(11)
                            .foregroundStyle(.secondary)
                    }
                }
                if let message {
                    Text(message).jayjayFont(12).fixedSize(horizontal: false, vertical: true)
                }
                if let error {
                    Text(error).jayjayFont(11).foregroundStyle(.red).fixedSize(horizontal: false, vertical: true)
                }
                if !busy {
                    Button(Localized.Settings.flockAnotherAccount) {
                        Task { await verify(newSignIn: true) }
                    }
                    .controlSize(.small)
                    Link(Localized.Settings.flockSponsor, destination: AppMetadata.sponsorURL)
                }
            }
        )
        .multilineTextAlignment(.leading)
        .task { await verify() }
        .onDisappear {
            closed = true
            sync.cancel()
        }
    }

    private func verify(newSignIn: Bool = false) async {
        guard !busy, !closed else { return }
        busy = true
        error = nil
        message = nil
        signIn = nil
        let token = JayJaySyncToken.standalone()
        sync = token
        defer {
            busy = false
            signIn = nil
        }
        do {
            let result = try await check(newSignIn: newSignIn, token: token)
            // Core has already saved an unlock, so it applies even if the sheet just closed.
            if case let .unlocked(member) = result {
                flock.accept(member)
            }
            guard !closed else { return }
            switch result {
                case .unlocked:
                    onDismiss()
                case let .notSponsored(login):
                    message = Localized.Settings.flockNotSponsored(login: login)
                case .signInRequired:
                    error = Localized.Settings.flockSignInFailed
            }
        } catch JayError.Canceled {
        } catch {
            self.error = error.friendlyDescription
        }
    }

    private func check(newSignIn: Bool, token: JayJaySyncToken) async throws -> FlockVerification {
        let sponsorable = AppMetadata.sponsorableLogin
        var result: FlockVerification = .signInRequired
        if !newSignIn {
            result = try await Task.detached(priority: .userInitiated) { try verifyFlock(sponsorable: sponsorable, sync: token) }.value
        }
        guard case .signInRequired = result, !closed else { return result }
        let session = try await Task.detached(priority: .userInitiated) { try FlockSignIn.start(sponsorable: sponsorable) }.value
        guard !closed else { return result }
        signIn = session
        return try await Task.detached(priority: .userInitiated) { try session.wait(sync: token) }.value
    }

    private func cancel() {
        closed = true
        sync.cancel()
        onDismiss()
    }
}
