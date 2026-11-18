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
    @State private var leadingWidth: CGFloat = 0
    @State private var windowOrigin = WindowLeadingEdgeReader.ProbeView.settledEdge(in: NSApp.mainWindow)

    private static var capsulePadding: CGFloat {
        4
    }

    private static var spacing: CGFloat {
        8
    }

    var body: some View {
        WholePointWidth {
            content
        }
        .background(WindowLeadingEdgeReader { windowOrigin = $0 })
    }

    private var content: some View {
        HStack(spacing: Self.spacing) {
            HStack(spacing: 0) {
                leading()
                    .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { leadingWidth = $0 }
                collapsing()
                    .fixedSize()
                    .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { collapsingWidth = $0 }
                    .frame(width: collapsingWidth * reveal.progress, alignment: .leading)
                    .clipped()
                    .opacity(reveal.progress)
                    .allowsHitTesting(reveal.progress == 1)
                    .accessibilityHidden(!reveal.isShown)
            }
            .padding(.horizontal, Self.capsulePadding)
            .glassEffect(.regular.interactive(), in: .capsule)
            trailing()
                .padding(.leading, gap * reveal.progress)
        }
        .fixedSize()
        .padding(.trailing, (collapsingWidth + gap) * (1 - reveal.progress))
    }

    private var gap: CGFloat {
        // From widths that hold still while the capsule animates: a measurement of the capsule itself arrives a frame late.
        let capsule = leadingWidth + collapsingWidth + 2 * Self.capsulePadding
        return max(0, trailingOrigin - windowOrigin - capsule - Self.spacing)
    }
}

/// AppKit seats an item of fractional width on a half point, which moves the title by a pixel between names of different lengths.
private struct WholePointWidth: Layout {
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache _: inout ()) -> CGSize {
        let size = subviews[0].sizeThatFits(proposal)
        return CGSize(width: size.width.rounded(.up), height: size.height)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache _: inout ()) {
        subviews[0].place(at: bounds.origin, anchor: .topLeading, proposal: proposal)
    }
}
