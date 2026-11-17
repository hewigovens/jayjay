import SwiftUI

/// Leading pane with a draggable divider; `range` derives the drag bounds from the available width and `onEnded` receives the settled width.
/// A hidden leading pane stays mounted and in place under the trailing pane: moving its scroll view out from under the toolbar makes the toolbar items flash.
struct ResizableSplit<Leading: View, Trailing: View>: View {
    @Binding var width: CGFloat
    var isLeadingHidden = false
    let range: (CGFloat) -> ClosedRange<CGFloat>
    let onEnded: (CGFloat) -> Void
    let dividerIdentifier: String
    @ViewBuilder let leading: () -> Leading
    @ViewBuilder let trailing: () -> Trailing
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        GeometryReader { geo in
            let range = range(geo.size.width)
            let position = Binding(get: { min(width, range.upperBound) }, set: { width = $0 })
            ZStack(alignment: .leading) {
                leading()
                    .frame(width: position.wrappedValue)
                    .opacity(isLeadingHidden ? 0 : 1)
                    .allowsHitTesting(!isLeadingHidden)
                    .accessibilityHidden(isLeadingHidden)
                HStack(spacing: 0) {
                    if !isLeadingHidden {
                        PaneSeparator(paneEdge: .trailing)
                    }
                    trailing()
                        .frame(maxWidth: .infinity)
                }
                .padding(.leading, isLeadingHidden ? 0 : position.wrappedValue)
                if !isLeadingHidden {
                    PaneDivider(position: position, range: range, onEnded: onEnded)
                        .frame(width: PaneDivider.hitWidth)
                        .offset(x: position.wrappedValue + (PaneLayout.dividerWidth - PaneDivider.hitWidth) / 2)
                        .accessibilityElement()
                        .accessibilityIdentifier(dividerIdentifier)
                }
            }
            .animation(reduceMotion ? nil : .easeInOut(duration: 0.25), value: isLeadingHidden)
        }
    }
}
