import SwiftUI

/// SwiftUI copies the first `.help` inside a toolbar item onto the item itself, which then shows over every other control in it.
struct ToolbarItemToolTipSilencer: NSViewRepresentable {
    func makeNSView(context _: Context) -> ProbeView {
        ProbeView()
    }

    func updateNSView(_: ProbeView, context _: Context) {}

    final class ProbeView: NSView {
        private var observation: NSKeyValueObservation?

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            let item = window?.toolbar?.items.first { item in item.view.map(isDescendant(of:)) ?? false }
            observation = item?.observe(\.toolTip, options: .initial) { item, _ in
                if item.toolTip != nil {
                    item.toolTip = nil
                }
            }
        }

        override func hitTest(_: NSPoint) -> NSView? {
            nil
        }
    }
}
