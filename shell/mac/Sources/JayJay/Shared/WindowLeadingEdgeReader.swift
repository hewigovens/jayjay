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
        /// Per window, so a view rebuilt by a workspace switch starts from its own window's edge instead of zero or a full-screen window's.
        private static var settledEdges: [Int: CGFloat] = [:]
        var onChange: (CGFloat) -> Void = { _ in }
        private var reported: CGFloat?
        private var pending: DispatchWorkItem?

        static func settledEdge(in window: NSWindow?) -> CGFloat {
            window.flatMap { settledEdges[$0.windowNumber] } ?? 0
        }

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            report()
        }

        override func hitTest(_: NSPoint) -> NSView? {
            nil
        }

        override func layout() {
            super.layout()
            report()
        }

        /// Waits for the toolbar to settle: while an item resizes it sits centred on its old frame, and feeding that edge back into the item's width never comes to rest.
        func report() {
            pending?.cancel()
            let work = DispatchWorkItem { [weak self] in
                guard let self, let window else { return }
                let edge = convert(bounds, to: nil).minX
                if let reported, abs(edge - reported) < 1 {
                    return
                }
                reported = edge
                Self.settledEdges[window.windowNumber] = edge
                onChange(edge)
            }
            pending = work
            // A window with no edge yet has nothing to hold still, so its first one is not worth the wait.
            let isFirst = window.map { Self.settledEdges[$0.windowNumber] == nil } ?? false
            DispatchQueue.main.asyncAfter(deadline: .now() + (isFirst ? 0 : 0.05), execute: work)
        }
    }
}
