import JayJayCore
import SwiftUI

struct FlockCodeSheet: View {
    let flock: FlockModel
    let onDismiss: () -> Void
    @State private var code = ""
    @State private var name = ""
    @State private var busy = false
    @State private var error: String?

    var body: some View {
        SheetContainer(
            title: Localized.Settings.flockCodeTitle,
            cancelLabel: Localized.Common.cancel,
            confirmLabel: Localized.Settings.flockCodeUnlock,
            confirmDisabled: busy || isIncomplete,
            confirmAccessibilityIdentifier: AID.Settings.flockCodeUnlock,
            onCancel: onDismiss,
            onConfirm: { Task { await redeem() } },
            content: {
                Text(Localized.Settings.flockCodeDetail)
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                TextField(Localized.Settings.flockCodeField, text: $code)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(size: 12, design: .monospaced))
                    .accessibilityIdentifier(AID.Settings.flockCodeField)
                TextField(Localized.Settings.flockCodeName, text: $name)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier(AID.Settings.flockCodeName)
                if let error {
                    Text(error).jayjayFont(11).foregroundStyle(.red).fixedSize(horizontal: false, vertical: true)
                }
                Link(Localized.Settings.flockSponsor, destination: AppMetadata.sponsorURL)
            }
        )
        .multilineTextAlignment(.leading)
    }

    private var isIncomplete: Bool {
        code.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            || name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    private func redeem() async {
        busy = true
        error = nil
        defer { busy = false }
        let code = code
        let name = name
        do {
            if let member = try await Task.detached(priority: .userInitiated, operation: { try redeemFlockCode(code: code, name: name) }).value {
                flock.accept(member)
                onDismiss()
            } else {
                error = Localized.Settings.flockCodeInvalid
            }
        } catch {
            self.error = error.friendlyDescription
        }
    }
}
