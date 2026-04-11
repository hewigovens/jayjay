import AppKit

public final class SideBySideCoordinator: NSObject, NSSplitViewDelegate {
    weak var leftContainer: DiffTextContainerView?
    weak var rightContainer: DiffTextContainerView?
    private var syncing = false

    public func splitView(
        _ splitView: NSSplitView,
        constrainMinCoordinate proposedMinimumPosition: CGFloat,
        ofSubviewAt dividerIndex: Int
    ) -> CGFloat {
        100
    }

    public func splitView(_ splitView: NSSplitView, resizeSubviewsWithOldSize oldSize: NSSize) {
        let dividerThickness = splitView.dividerThickness
        let halfWidth = (splitView.bounds.width - dividerThickness) / 2
        if splitView.subviews.count >= 2 {
            splitView.subviews[0].frame = NSRect(x: 0, y: 0, width: halfWidth, height: splitView.bounds.height)
            splitView.subviews[1].frame = NSRect(
                x: halfWidth + dividerThickness,
                y: 0,
                width: halfWidth,
                height: splitView.bounds.height
            )
        }
    }

    func startObserving() {
        leftContainer?.scrollView.contentView.postsBoundsChangedNotifications = true
        rightContainer?.scrollView.contentView.postsBoundsChangedNotifications = true
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(leftScrolled),
            name: NSView.boundsDidChangeNotification,
            object: leftContainer?.scrollView.contentView
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(rightScrolled),
            name: NSView.boundsDidChangeNotification,
            object: rightContainer?.scrollView.contentView
        )
    }

    @objc private func leftScrolled(_ notification: Notification) {
        guard !syncing,
              let origin = leftContainer?.scrollView.contentView.bounds.origin,
              let right = rightContainer?.scrollView
        else { return }
        syncing = true
        right.contentView.scroll(to: origin)
        right.reflectScrolledClipView(right.contentView)
        syncing = false
    }

    @objc private func rightScrolled(_ notification: Notification) {
        guard !syncing,
              let origin = rightContainer?.scrollView.contentView.bounds.origin,
              let left = leftContainer?.scrollView
        else { return }
        syncing = true
        left.contentView.scroll(to: origin)
        left.reflectScrolledClipView(left.contentView)
        syncing = false
    }

    deinit {
        NotificationCenter.default.removeObserver(self)
    }
}
