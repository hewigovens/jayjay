import SwiftUI

/// Reusable sheet container with consistent styling for modal dialogs.
struct SheetContainer<Content: View>: View {
    let title: String
    var subtitle: String?
    let cancelLabel: String
    let confirmLabel: String
    var confirmDisabled: Bool = false
    var confirmRole: ButtonRole?
    let onCancel: () -> Void
    let onConfirm: () -> Void
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title)
                .jayjayFont(14, weight: .semibold)
            if let subtitle {
                Text(subtitle)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
            }
            content()
            HStack {
                Spacer()
                Button(cancelLabel) { onCancel() }
                    .keyboardShortcut(.cancelAction)
                Button(confirmLabel, role: confirmRole) { onConfirm() }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(confirmDisabled)
                Spacer()
            }
        }
        .padding(20)
    }
}
