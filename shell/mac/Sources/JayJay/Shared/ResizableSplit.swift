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
                        Rectangle()
                            .fill(Color.primary.opacity(0.08))
                            .frame(width: PaneLayout.dividerWidth)
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

/// An AppKit grip above both panes: a SwiftUI hover cursor loses to the diff's text views, which own the cursor wherever they sit under the mouse.
private struct PaneDivider: NSViewRepresentable {
    static let hitWidth: CGFloat = 9

    @Binding var position: CGFloat
    let range: ClosedRange<CGFloat>
    let onEnded: (CGFloat) -> Void

    func makeNSView(context: Context) -> GripView {
        GripView()
    }

    func updateNSView(_ view: GripView, context: Context) {
        view.position = $position
        view.range = range
        view.onEnded = onEnded
    }

    final class GripView: NSView {
        var position: Binding<CGFloat> = .constant(0)
        var range: ClosedRange<CGFloat> = 0 ... 0
        var onEnded: (CGFloat) -> Void = { _ in }
        private var dragStart: (x: CGFloat, position: CGFloat)?

        override init(frame: NSRect) {
            super.init(frame: frame)
            addTrackingArea(NSTrackingArea(rect: .zero, options: [.cursorUpdate, .mouseEnteredAndExited, .activeInKeyWindow, .inVisibleRect], owner: self))
        }

        @available(*, unavailable)
        required init?(coder: NSCoder) {
            fatalError("init(coder:) has not been implemented")
        }

        override func acceptsFirstMouse(for event: NSEvent?) -> Bool {
            true
        }

        override func cursorUpdate(with event: NSEvent) {
            NSCursor.resizeLeftRight.set()
        }

        override func mouseExited(with event: NSEvent) {
            if dragStart == nil {
                restoreCursorUnderMouse(event)
            }
        }

        override func mouseDown(with event: NSEvent) {
            dragStart = (event.locationInWindow.x, position.wrappedValue)
        }

        override func mouseDragged(with event: NSEvent) {
            guard let dragStart else { return }
            let target = dragStart.position + event.locationInWindow.x - dragStart.x
            position.wrappedValue = min(max(target, range.lowerBound), range.upperBound)
        }

        override func mouseUp(with event: NSEvent) {
            guard dragStart != nil else { return }
            dragStart = nil
            onEnded(position.wrappedValue)
            if !bounds.contains(convert(event.locationInWindow, from: nil)) {
                restoreCursorUnderMouse(event)
            }
        }

        /// The grip overlaps the panes' own cursor areas, so leaving it never re-enters them; ask the view underneath to restore its cursor.
        private func restoreCursorUnderMouse(_ event: NSEvent) {
            guard let update = NSEvent.enterExitEvent(
                with: .cursorUpdate, location: event.locationInWindow, modifierFlags: [], timestamp: event.timestamp,
                windowNumber: event.windowNumber, context: nil, eventNumber: 0, trackingNumber: 0, userData: nil
            ) else { return }
            window?.contentView?.hitTest(event.locationInWindow)?.cursorUpdate(with: update)
        }
    }
}
