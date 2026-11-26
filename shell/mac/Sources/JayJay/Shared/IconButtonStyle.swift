import SwiftUI

struct IconButtonStyle: ButtonStyle {
    static let minimumSize: CGFloat = 24

    func makeBody(configuration: Configuration) -> some View {
        Label(configuration: configuration)
    }

    private struct Label: View {
        let configuration: Configuration
        @Environment(\.isEnabled) private var isEnabled

        var body: some View {
            configuration.label
                .opacity(isEnabled ? (configuration.isPressed ? 0.6 : 1) : 0.4)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .contentShape(Rectangle())
        }
    }
}

extension View {
    func iconButtonStyle(minimumHeight: CGFloat = IconButtonStyle.minimumSize) -> some View {
        buttonStyle(IconButtonStyle()).minimumHitTarget(height: minimumHeight)
    }

    /// The buttons' styles must fill the size offered, as `IconButtonStyle` and `HeaderActionButtonStyle` do.
    func minimumHitTarget(height: CGFloat = IconButtonStyle.minimumSize) -> some View {
        MinimumHitTargetLayout(minimumHeight: height) { self }
    }
}

/// macOS hit-tests a button only inside its own frame, so the frame grows and overhangs while the parent sees the natural size.
private struct MinimumHitTargetLayout: Layout {
    let minimumHeight: CGFloat

    func sizeThatFits(proposal _: ProposedViewSize, subviews: Subviews, cache _: inout ()) -> CGSize {
        subviews.first?.sizeThatFits(.unspecified) ?? .zero
    }

    func placeSubviews(in bounds: CGRect, proposal _: ProposedViewSize, subviews: Subviews, cache _: inout ()) {
        guard let content = subviews.first else { return }
        content.place(at: CGPoint(x: bounds.midX, y: bounds.midY), anchor: .center, proposal: target(for: content))
    }

    /// Forwards the content's own guides, such as its text baseline, so rows that align on them stay put.
    func explicitAlignment(
        of guide: VerticalAlignment, in bounds: CGRect, proposal _: ProposedViewSize, subviews: Subviews, cache _: inout ()
    ) -> CGFloat? {
        guard let content = subviews.first else { return nil }
        let dimensions = content.dimensions(in: target(for: content))
        return bounds.midY - dimensions.height / 2 + dimensions[guide]
    }

    private func target(for content: LayoutSubview) -> ProposedViewSize {
        let natural = content.sizeThatFits(.unspecified)
        return ProposedViewSize(
            width: max(natural.width, IconButtonStyle.minimumSize),
            height: max(natural.height, minimumHeight)
        )
    }
}
