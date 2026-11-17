import SwiftUI

/// A trailing pane that slides in from the window edge at its full width, so its content never reflows while it is revealed.
struct TrailingPaneColumn<Content: View>: View {
    let reveal: PaneReveal
    @Binding var width: CGFloat
    let range: ClosedRange<CGFloat>
    let onResizeEnded: (CGFloat) -> Void
    let content: Content

    init(
        reveal: PaneReveal,
        width: Binding<CGFloat>,
        range: ClosedRange<CGFloat>,
        onResizeEnded: @escaping (CGFloat) -> Void,
        @ViewBuilder content: () -> Content
    ) {
        self.reveal = reveal
        _width = width
        self.range = range
        self.onResizeEnded = onResizeEnded
        self.content = content()
    }

    /// The content stays mounted while hidden: building it during the first reveal would stall that animation.
    var body: some View {
        HStack(spacing: 0) {
            if reveal.isShown {
                PaneSeparator(paneEdge: .leading)
            }
            content
                .frame(width: width)
                .frame(width: max(0, width * reveal.progress - PaneLayout.dividerWidth), alignment: .leading)
                .clipped()
                .opacity(reveal.isShown ? 1 : 0)
                .allowsHitTesting(reveal.isShown)
                .accessibilityHidden(!reveal.isShown)
        }
        .overlay(alignment: .leading) {
            if reveal.isShown {
                PaneDivider(position: $width, range: range, onEnded: onResizeEnded, paneEdge: .trailing)
                    .frame(width: PaneDivider.hitWidth)
                    .offset(x: (PaneLayout.dividerWidth - PaneDivider.hitWidth) / 2)
            }
        }
    }
}
