import AppKit
import SwiftUI

/// Reports a view's leading edge in window coordinates; a toolbar item's SwiftUI geometry only knows its own hosting view.
struct WindowLeadingEdgeReader: NSViewRepresentable {
    let onChange: (CGFloat) -> Void

    func makeNSView(context _: Context) -> ProbeView {
        let view = ProbeView()
        view.onChange = onChange
        return view
    }

    func updateNSView(_ view: ProbeView, context _: Context) {
        view.onChange = onChange
        view.report()
    }

    final class ProbeView: NSView {
        var onChange: (CGFloat) -> Void = { _ in }
        private var reported: CGFloat?

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            report()
        }

        override func layout() {
            super.layout()
            report()
        }

        func report() {
            guard window != nil else { return }
            let edge = convert(bounds, to: nil).minX
            guard edge != reported else { return }
            reported = edge
            DispatchQueue.main.async { [onChange] in onChange(edge) }
        }
    }
}
