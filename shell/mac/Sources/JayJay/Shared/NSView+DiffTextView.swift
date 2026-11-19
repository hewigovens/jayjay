import AppKit

extension NSView {
    fileprivate static let diffTextViewID = NSUserInterfaceItemIdentifier("diffTextView")

    var diffTextView: NSTextView? {
        if let textView = self as? NSTextView, textView.identifier == Self.diffTextViewID {
            return textView
        }
        for subview in subviews {
            if let found = subview.diffTextView {
                return found
            }
        }
        return nil
    }
}

extension NSWindow {
    /// The diff holding keyboard focus, or the first one when focus is elsewhere.
    var activeDiffTextView: NSTextView? {
        if let focused = firstResponder as? NSTextView, focused.identifier == NSView.diffTextViewID {
            return focused
        }
        return contentView?.diffTextView
    }
}
