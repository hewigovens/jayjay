import SwiftUI

/// An AppKit grip above both panes: a SwiftUI hover cursor loses to the diff's text views, which own the cursor wherever they sit under the mouse.
struct PaneDivider: NSViewRepresentable {
    static let hitWidth: CGFloat = 9

    @Binding var position: CGFloat
    let range: ClosedRange<CGFloat>
    let onEnded: (CGFloat) -> Void
    /// The side of the grip the sized pane is on; a trailing pane grows as the grip moves left.
    var paneEdge = HorizontalEdge.leading

    func makeNSView(context: Context) -> GripView {
        GripView()
    }

    func updateNSView(_ view: GripView, context: Context) {
        view.position = $position
        view.range = range
        view.onEnded = onEnded
        view.direction = paneEdge == .leading ? 1 : -1
    }

    final class GripView: NSView {
        var position: Binding<CGFloat> = .constant(0)
        var range: ClosedRange<CGFloat> = 0 ... 0
        var onEnded: (CGFloat) -> Void = { _ in }
        var direction: CGFloat = 1
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
            let target = dragStart.position + direction * (event.locationInWindow.x - dragStart.x)
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
