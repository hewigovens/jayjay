import AppKit
import SwiftUI

struct ShortcutRecorderFocus: NSViewRepresentable {
    let target: Target

    final class Target {
        weak var view: NSView?
    }

    func makeNSView(context: Context) -> NSView {
        let view = FocusView()
        target.view = view
        return view
    }

    func updateNSView(_: NSView, context: Context) {}

    private final class FocusView: NSView {
        override var acceptsFirstResponder: Bool { true }
        override func hitTest(_ point: NSPoint) -> NSView? { nil }
    }
}
