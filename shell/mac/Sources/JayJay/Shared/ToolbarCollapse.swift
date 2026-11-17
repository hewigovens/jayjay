import SwiftUI

/// One toolbar item whose middle controls narrow to nothing as `reveal` hides, sliding what follows them into their place.
/// The item keeps its full width throughout: one that changes size re-lays out the toolbar mid-animation and can strand items in the overflow menu.
struct ToolbarCollapse<Leading: View, Collapsing: View, Trailing: View>: View {
    let reveal: PaneReveal
    /// Where the trailing content starts while revealed, from the window's leading edge.
    let trailingOrigin: CGFloat
    @ViewBuilder let leading: () -> Leading
    @ViewBuilder let collapsing: () -> Collapsing
    @ViewBuilder let trailing: () -> Trailing
    @State private var collapsingWidth: CGFloat = 0
    @State private var capsuleWidth: CGFloat = 0
    @State private var windowOrigin: CGFloat = 0

    private static var spacing: CGFloat {
        8
    }

    var body: some View {
        HStack(spacing: Self.spacing) {
            HStack(spacing: 0) {
                leading()
                collapsing()
                    .fixedSize()
                    .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { collapsingWidth = $0 }
                    .frame(width: collapsingWidth * reveal.progress, alignment: .leading)
                    .clipped()
                    .opacity(reveal.progress)
                    .allowsHitTesting(reveal.progress == 1)
                    .accessibilityHidden(!reveal.isShown)
            }
            .padding(.horizontal, 4)
            .glassEffect(.regular.interactive(), in: .capsule)
            .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { capsuleWidth = $0 }
            trailing()
                .padding(.leading, gap * reveal.progress)
        }
        .fixedSize()
        .padding(.trailing, (collapsingWidth + gap) * (1 - reveal.progress))
        .background(WindowLeadingEdgeReader { windowOrigin = $0 })
    }

    private var gap: CGFloat {
        let shownCapsule = capsuleWidth + collapsingWidth * (1 - reveal.progress)
        return max(0, trailingOrigin - windowOrigin - shownCapsule - Self.spacing)
    }
}
