import AppKit
import SwiftUI

/// Marks a native list that keeps its own keys: key monitors stand aside while its table is first responder, except for Tab.
struct KeyboardOwner: NSViewRepresentable {
    func makeNSView(context _: Context) -> KeyboardOwnerView {
        KeyboardOwnerView()
    }

    func updateNSView(_: KeyboardOwnerView, context _: Context) {}
}

final class KeyboardOwnerView: NSView {
    @MainActor static let live = NSHashTable<KeyboardOwnerView>.weakObjects()

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        if window == nil {
            Self.live.remove(self)
        } else {
            Self.live.add(self)
        }
    }
}

extension NSView {
    /// SwiftUI hosts the marker beside the list, not above it, so ownership is decided by where the responder sits.
    var isInKeyboardOwner: Bool {
        guard let window else { return false }
        let center = convert(NSPoint(x: bounds.midX, y: bounds.midY), to: nil)
        return KeyboardOwnerView.live.allObjects.contains { owner in
            owner.window === window && !owner.isHiddenOrHasHiddenAncestor && owner.bounds.contains(owner.convert(center, from: nil))
        }
    }
}
