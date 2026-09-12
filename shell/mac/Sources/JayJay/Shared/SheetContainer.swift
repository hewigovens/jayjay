import SwiftUI

/// Reusable sheet container with consistent styling for modal dialogs.
struct SheetContainer<Content: View>: View {
    let title: String
    var subtitle: String?
    var inlineHeaderIcon: String?
    let cancelLabel: String
    var cancelDisabled: Bool = false
    let confirmLabel: String
    var confirmDisabled: Bool = false
    var confirmRole: ButtonRole?
    let onCancel: () -> Void
    let onConfirm: () -> Void
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            content()
            HStack {
                Spacer()
                Button(cancelLabel) { onCancel() }
                    .disabled(cancelDisabled)
                    .keyboardShortcut(.cancelAction)
                Button(confirmLabel, role: confirmRole) { onConfirm() }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(confirmDisabled)
                Spacer()
            }
            .padding(.top, 10)
        }
        .padding(20)
    }

    private var header: some View {
        let layout = inlineHeaderIcon == nil
            ? AnyLayout(VStackLayout(alignment: .leading, spacing: 12))
            : AnyLayout(HStackLayout(spacing: 8))
        return layout {
            HStack(spacing: 8) {
                if let inlineHeaderIcon {
                    Image(systemName: inlineHeaderIcon)
                        .foregroundStyle(.secondary)
                }
                Text(title)
            }
            .jayjayFont(14, weight: .semibold)
            .fixedSize(horizontal: inlineHeaderIcon != nil, vertical: false)
            if inlineHeaderIcon != nil {
                Spacer(minLength: 12)
            }
            if let subtitle {
                Text(subtitle)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(inlineHeaderIcon == nil ? nil : 1)
                    .truncationMode(.middle)
            }
        }
    }
}
