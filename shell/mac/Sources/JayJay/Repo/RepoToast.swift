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
            WrappedWidth(maxWidth: 400) {
                Text(toast.message)
                    .jayjayFont(13, weight: .medium)
                    .foregroundStyle(colorScheme == .dark ? .white : .black)
            }
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
        .glassEffect(in: RoundedRectangle(cornerRadius: 14, style: .continuous))
    }
}

/// Hugs short content and wraps content wider than `maxWidth`; a `maxWidth` frame would pad short toasts out to it.
private struct WrappedWidth: Layout {
    let maxWidth: CGFloat

    func sizeThatFits(proposal _: ProposedViewSize, subviews: Subviews, cache _: inout ()) -> CGSize {
        guard let content = subviews.first else { return .zero }
        let ideal = content.sizeThatFits(.unspecified)
        return ideal.width <= maxWidth ? ideal : content.sizeThatFits(ProposedViewSize(width: maxWidth, height: nil))
    }

    func placeSubviews(in bounds: CGRect, proposal _: ProposedViewSize, subviews: Subviews, cache _: inout ()) {
        subviews.first?.place(at: bounds.origin, proposal: ProposedViewSize(bounds.size))
    }
}
