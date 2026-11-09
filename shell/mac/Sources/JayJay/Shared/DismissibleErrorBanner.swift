import JayJayCore
import SwiftUI

struct DismissibleErrorBanner: View {
    let message: String
    let onDismiss: () -> Void

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: "exclamationmark.triangle")
                .foregroundStyle(.orange)
            Text(message)
                .jayjayFont(12)
                .textSelection(.enabled)
            Spacer(minLength: 8)
            Button(Localized.Repo.dismiss, action: onDismiss)
                .buttonStyle(.plain)
                .jayjayFont(12)
                .foregroundStyle(Color.accentColor)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
    }
}
