import AppKit

public extension NSTextView {
    func adoptTextKit1() {
        _ = layoutManager
        textStorage?.delegate = TextPresentationFallback.shared
    }
}
