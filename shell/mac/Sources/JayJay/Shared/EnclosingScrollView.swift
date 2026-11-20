import AppKit
import SwiftUI

/// Hands out the AppKit scroll view around a SwiftUI `ScrollView`'s content, which SwiftUI cannot page.
struct EnclosingScrollView: NSViewRepresentable {
    let handle: Handle

    func makeNSView(context _: Context) -> NSView {
        let view = NSView()
        handle.anchor = view
        return view
    }

    func updateNSView(_ view: NSView, context _: Context) {
        handle.anchor = view
    }

    final class Handle {
        fileprivate weak var anchor: NSView?

        var scrollView: NSScrollView? {
            anchor?.enclosingScrollView
        }
    }
}
