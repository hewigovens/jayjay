import AppKit

extension NSView {
    private static let diffTextViewID = NSUserInterfaceItemIdentifier("diffTextView")

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
