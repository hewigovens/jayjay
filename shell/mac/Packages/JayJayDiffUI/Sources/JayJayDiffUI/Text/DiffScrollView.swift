import AppKit

final class DiffScrollView: NSScrollView {
    var forwardsScrollWheel = false

    override func scrollWheel(with event: NSEvent) {
        guard forwardsScrollWheel, let outerScrollView else {
            super.scrollWheel(with: event)
            return
        }
        outerScrollView.scrollWheel(with: event)
    }

    private var outerScrollView: NSScrollView? {
        var ancestor = superview
        while let view = ancestor {
            if let scrollView = view as? NSScrollView {
                return scrollView
            }
            ancestor = view.superview
        }
        return nil
    }
}
