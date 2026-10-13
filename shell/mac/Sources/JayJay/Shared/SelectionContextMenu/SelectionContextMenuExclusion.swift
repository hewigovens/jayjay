import AppKit
import SwiftUI

struct SelectionContextMenuExclusion: NSViewRepresentable {
    func makeNSView(context _: Context) -> SelectionContextMenuExclusionView {
        SelectionContextMenuExclusionView()
    }

    func updateNSView(_: SelectionContextMenuExclusionView, context _: Context) {}
}

final class SelectionContextMenuExclusionView: NSView {}

extension NSView {
    func hasContextMenuExclusion(at windowPoint: NSPoint) -> Bool {
        guard !isHiddenOrHasHiddenAncestor else { return false }
        if self is SelectionContextMenuExclusionView, bounds.contains(convert(windowPoint, from: nil)) {
            return true
        }
        return subviews.contains { $0.hasContextMenuExclusion(at: windowPoint) }
    }
}
