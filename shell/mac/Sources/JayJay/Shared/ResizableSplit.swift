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
                        PaneDivider(position: position, range: range, onEnded: onEnded)
                            .accessibilityElement()
                            .accessibilityIdentifier(dividerIdentifier)
                    }
                    trailing()
                        .frame(maxWidth: .infinity)
                }
                .padding(.leading, isLeadingHidden ? 0 : position.wrappedValue)
            }
            .animation(reduceMotion ? nil : .easeInOut(duration: 0.25), value: isLeadingHidden)
        }
    }
}

private struct PaneDivider: View {
    @Binding var position: CGFloat
    let range: ClosedRange<CGFloat>
    let onEnded: (CGFloat) -> Void
    @State private var dragStart: CGFloat?

    var body: some View {
        Rectangle()
            .fill(Color.primary.opacity(0.08))
            .frame(width: PaneLayout.dividerWidth)
            .contentShape(Rectangle().inset(by: -3))
            .onHover {
                if $0 {
                    NSCursor.resizeLeftRight.push()
                } else {
                    NSCursor.pop()
                }
            }
            .gesture(
                DragGesture(minimumDistance: 1, coordinateSpace: .global)
                    .onChanged {
                        let start = dragStart ?? position
                        dragStart = start
                        position = min(max(start + $0.translation.width, range.lowerBound), range.upperBound)
                    }
                    .onEnded { _ in
                        dragStart = nil
                        onEnded(position)
                    }
            )
    }
}
