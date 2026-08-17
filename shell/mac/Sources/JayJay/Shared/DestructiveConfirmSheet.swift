import SwiftUI

struct DestructiveConfirmSheet: View {
    let title: String
    let message: String
    let confirmLabel: String
    var width: CGFloat = 340
    var dontAskAgain: Binding<Bool>?
    let onCancel: () -> Void
    let onConfirm: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "trash.circle.fill")
                .font(.system(size: 36))
                .foregroundStyle(.red)
            Text(title)
                .jayjayFont(16, weight: .semibold)
            Text(message)
                .jayjayFont(13)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            if let dontAskAgain {
                Toggle("Don't ask again", isOn: dontAskAgain)
                    .jayjayFont(12)
            }

            HStack(spacing: 12) {
                Button("Cancel", action: onCancel)
                    .keyboardShortcut(.cancelAction)
                Button(confirmLabel, action: onConfirm)
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .tint(.red)
            }
        }
        .padding(24)
        .frame(width: width)
    }
}
