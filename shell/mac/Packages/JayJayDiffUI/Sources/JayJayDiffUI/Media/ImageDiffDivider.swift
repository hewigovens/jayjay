import AppKit
import SwiftUI

struct ImageDiffDivider: View {
    static let width: CGFloat = 12

    @Binding var fraction: CGFloat
    let availableWidth: CGFloat

    var body: some View {
        MouseHandler(fraction: $fraction, availableWidth: availableWidth)
            .frame(width: Self.width)
            .overlay {
                Rectangle()
                    .fill(Color.primary.opacity(0.15))
                    .frame(width: 1)
                    .allowsHitTesting(false)
            }
            .help("Drag to resize images. Double-click to restore equal widths.")
            .accessibilityElement()
            .accessibilityLabel("Image comparison divider")
            .accessibilityValue("Before \(Int(fraction * 100)) percent")
            .accessibilityAdjustableAction { direction in
                switch direction {
                    case .increment: fraction = min(fraction + 0.05, 0.9)
                    case .decrement: fraction = max(fraction - 0.05, 0.1)
                    @unknown default: break
                }
            }
            .accessibilityAction(named: "Restore equal widths") { fraction = 0.5 }
    }

    private struct MouseHandler: NSViewRepresentable {
        @Binding var fraction: CGFloat
        let availableWidth: CGFloat

        func makeNSView(context: Context) -> MouseView {
            MouseView()
        }

        func updateNSView(_ view: MouseView, context: Context) {
            view.fraction = $fraction
            view.availableWidth = availableWidth
        }
    }

    private final class MouseView: NSView {
        var fraction: Binding<CGFloat> = .constant(0.5)
        var availableWidth: CGFloat = 0
        private var dragStart: (position: NSPoint, fraction: CGFloat)?
        private var didDrag = false

        override func acceptsFirstMouse(for event: NSEvent?) -> Bool {
            true
        }

        override func resetCursorRects() {
            addCursorRect(bounds, cursor: .resizeLeftRight)
        }

        override func mouseDown(with event: NSEvent) {
            dragStart = (event.locationInWindow, fraction.wrappedValue)
            didDrag = false
        }

        override func mouseDragged(with event: NSEvent) {
            guard let dragStart, availableWidth > 0 else { return }
            let dx = event.locationInWindow.x - dragStart.position.x
            let dy = event.locationInWindow.y - dragStart.position.y
            didDrag = didDrag || hypot(dx, dy) > 1
            fraction.wrappedValue = min(max(dragStart.fraction + dx / availableWidth, 0.1), 0.9)
        }

        override func mouseUp(with event: NSEvent) {
            defer { dragStart = nil }
            guard let dragStart, event.clickCount == 2, !didDrag else { return }
            let dx = event.locationInWindow.x - dragStart.position.x
            let dy = event.locationInWindow.y - dragStart.position.y
            if hypot(dx, dy) <= 1 {
                fraction.wrappedValue = 0.5
            }
        }
    }
}
