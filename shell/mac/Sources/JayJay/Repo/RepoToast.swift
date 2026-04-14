import SwiftUI

struct RepoToastAction {
    let title: String
    let perform: () -> Void
}

struct RepoToastState: Identifiable {
    let id = UUID()
    let message: String
    var action: RepoToastAction?
}

struct RepoToastView: View {
    let toast: RepoToastState
    let dismiss: () -> Void
    let colorScheme: ColorScheme

    var body: some View {
        HStack(spacing: 12) {
            Text(toast.message)
                .jayjayFont(13, weight: .medium)
                .foregroundStyle(colorScheme == .dark ? .white : .black)
            if let action = toast.action {
                Button(action.title) {
                    dismiss()
                    action.perform()
                }
                .buttonStyle(.plain)
                .jayjayFont(12, weight: .semibold)
                .foregroundStyle(Color.accentColor)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.12), in: Capsule())
            }
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 14)
        .background(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .fill(colorScheme == .dark ? Color.black.opacity(0.75) : Color.white.opacity(0.9))
                .shadow(color: .black.opacity(0.2), radius: 12, y: 6)
        )
    }
}
