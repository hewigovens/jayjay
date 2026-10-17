import JayJayCore
import SwiftUI

extension Notification.Name {
    static let jjIdentitySaved = Notification.Name("jayjay.jjIdentitySaved")
}

extension View {
    /// Presents the sheet while `isPresented` and keeps `identity` current after any save.
    func identitySetup(
        _ identity: Binding<UserIdentity?>,
        isPresented: Binding<Bool>,
        onSaved: @escaping () async -> Void = {}
    ) -> some View {
        sheet(isPresented: isPresented) {
            IdentitySheet(onDismiss: { isPresented.wrappedValue = false })
        }
        .onReceive(NotificationCenter.default.publisher(for: .jjIdentitySaved)) { notification in
            identity.wrappedValue = notification.object as? UserIdentity
            Task { await onSaved() }
        }
    }
}

struct IdentitySheet: View {
    let onDismiss: () -> Void
    @State private var name = ""
    @State private var email = ""
    /// Set once the prefill lands; the fields stay disabled until then.
    @State private var path: String?
    @State private var error: String?
    @State private var isSaving = false

    var body: some View {
        SheetContainer(
            title: "Set Up Identity",
            subtitle: path,
            cancelLabel: "Cancel",
            cancelDisabled: isSaving,
            confirmLabel: "Save",
            confirmDisabled: identity == nil || isSaving,
            confirmAccessibilityIdentifier: AID.Settings.identitySave,
            onCancel: onDismiss,
            onConfirm: save,
            content: {
                Text("jj records this name and email on the changes you create.")
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                TextField("Name", text: $name, prompt: Text("Your Name"))
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier(AID.Settings.identityName)
                    .disabled(path == nil)
                TextField("Email", text: $email, prompt: Text("you@example.com"))
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier(AID.Settings.identityEmail)
                    .disabled(path == nil)
                if let error {
                    Text(error)
                        .jayjayFont(11)
                        .foregroundStyle(.red)
                }
            }
        )
        .frame(width: 400)
        .interactiveDismissDisabled(isSaving)
        .task { await load() }
    }

    private var identity: UserIdentity? {
        let name = name.trimmingCharacters(in: .whitespaces)
        let email = email.trimmingCharacters(in: .whitespaces)
        guard path != nil, !name.isEmpty, !email.isEmpty else { return nil }
        return UserIdentity(name: name, email: email)
    }

    private func load() async {
        let loaded = await Task.detached(priority: .userInitiated) {
            Result { try (suggestedUserIdentity(), userIdentityConfigPath()) }
        }.value
        switch loaded {
            case let .success((suggested, configPath)):
                name = suggested.name
                email = suggested.email
                path = configPath
            case let .failure(failure):
                error = failure.friendlyDescription
        }
    }

    private func save() {
        guard let identity, !isSaving else { return }
        isSaving = true
        error = nil
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result { try saveUserIdentity(identity: identity) }
            }.value
            isSaving = false
            switch result {
                case .success:
                    NotificationCenter.default.post(name: .jjIdentitySaved, object: identity)
                    onDismiss()
                case let .failure(failure):
                    error = failure.friendlyDescription
            }
        }
    }
}
