import AppKit

public final class NativeDiffContextCoordinator: NSObject, NSTextViewDelegate {
    var onExpandContext: ((DiffContextExpansionRequest) -> Void)?

    public func textView(
        _ textView: NSTextView,
        clickedOnLink link: Any,
        at charIndex: Int
    ) -> Bool {
        guard let request = DiffContextExpansionLink.request(from: link),
              let onExpandContext
        else { return false }
        onExpandContext(request)
        return true
    }
}
