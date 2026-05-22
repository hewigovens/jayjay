import AppKit
import JayJayCore

public final class SideBySideCoordinator: NSObject, NSSplitViewDelegate {
    weak var leftContainer: DiffTextContainerView?
    weak var rightContainer: DiffTextContainerView?
    private var syncing = false

    // State for rebuild-on-resize: stored each updateNSView call, then re-used
    // whenever the pane width changes enough to change the wrap-column count.
    var diff: FileDiff?
    var font: NSFont?
    var theme: DiffColors?
    private var lastOldCols: UInt32 = 0
    private var lastNewCols: UInt32 = 0

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
        // When either side's content size changes (window resize or splitter drag),
        // the wrap column count may need to change too.
        leftContainer?.onContentLayoutChanged = { [weak self] in
            self?.renderIfNeeded(force: false)
        }
        rightContainer?.onContentLayoutChanged = { [weak self] in
            self?.renderIfNeeded(force: false)
        }
    }

    /// Build (or rebuild) the attributed strings for the wrapped side-by-side view.
    /// `force=true` always rebuilds; `force=false` skips when wrap columns are unchanged.
    func renderIfNeeded(force: Bool) {
        guard let diff = diff,
              let font = font,
              let theme = theme,
              let leftContainer = leftContainer,
              let rightContainer = rightContainer,
              let leftLayout = leftContainer.textView.layoutManager as? DiffLayoutManager,
              let rightLayout = rightContainer.textView.layoutManager as? DiffLayoutManager,
              let leftGutterLayout = leftContainer.gutterTextView.layoutManager as? DiffLayoutManager,
              let rightGutterLayout = rightContainer.gutterTextView.layoutManager as? DiffLayoutManager
        else { return }

        // Pane content widths (post-gutter) and monospace cell advance.
        let oldWidth = max(0, leftContainer.scrollView.contentSize.width)
        let newWidth = max(0, rightContainer.scrollView.contentSize.width)
        let advance = Float(("M" as NSString).size(withAttributes: [.font: font]).width)
        let oldCols = wrapColsForWidth(width: Float(oldWidth), advance: advance)
        let newCols = wrapColsForWidth(width: Float(newWidth), advance: advance)
        if !force && oldCols == lastOldCols && newCols == lastNewCols {
            return
        }
        lastOldCols = oldCols
        lastNewCols = newCols

        // Pre-wrap into visual rows so both panes (and gutters) advance in lock-step.
        let rows = buildSideBySideRows(lines: diff.lines)
        let visualRows = wrapSbsRows(rows: rows, oldCols: oldCols, newCols: newCols).map(\.row)
        renderVisualRows(visualRows, font: font, theme: theme)
    }

    private func renderVisualRows(_ rows: [SideBySideRow], font: NSFont, theme: DiffColors) {
        guard let leftContainer = leftContainer,
              let rightContainer = rightContainer,
              let leftTV = leftContainer.textView as NSTextView?,
              let rightTV = rightContainer.textView as NSTextView?,
              let leftGutterTV = leftContainer.gutterTextView as DiffGutterTextView?,
              let rightGutterTV = rightContainer.gutterTextView as DiffGutterTextView?,
              let leftLayout = leftTV.layoutManager as? DiffLayoutManager,
              let rightLayout = rightTV.layoutManager as? DiffLayoutManager,
              let leftGutterLayout = leftGutterTV.layoutManager as? DiffLayoutManager,
              let rightGutterLayout = rightGutterTV.layoutManager as? DiffLayoutManager
        else { return }

        let leftText = NSMutableAttributedString()
        let rightText = NSMutableAttributedString()
        let leftGutter = NSMutableAttributedString()
        let rightGutter = NSMutableAttributedString()
        var leftEntries: [DiffGutterTextView.Entry] = []
        var rightEntries: [DiffGutterTextView.Entry] = []
        var leftWidth: CGFloat = 0
        var rightWidth: CGFloat = 0
        var leftColors: [NSColor] = []
        var rightColors: [NSColor] = []

        let gutterParagraphStyle = NSMutableParagraphStyle()
        gutterParagraphStyle.alignment = .right
        let gutterAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: theme.gutterText,
            .paragraphStyle: gutterParagraphStyle
        ]
        let trailingPadding: CGFloat = 10

        for row in rows {
            appendTextLine(
                to: leftText,
                spans: row.oldSpans,
                style: row.oldStyle,
                font: font,
                theme: theme,
                bgColors: &leftColors
            )
            appendTextLine(
                to: rightText,
                spans: row.newSpans,
                style: row.newStyle,
                font: font,
                theme: theme,
                bgColors: &rightColors
            )
            appendGutterLine(
                to: leftGutter,
                entries: &leftEntries,
                lineNo: row.oldLineNo,
                style: row.oldStyle,
                attrs: gutterAttrs,
                inset: leftGutterTV.textContainerInset.width,
                trailingPadding: trailingPadding,
                width: &leftWidth
            )
            appendGutterLine(
                to: rightGutter,
                entries: &rightEntries,
                lineNo: row.newLineNo,
                style: row.newStyle,
                attrs: gutterAttrs,
                inset: rightGutterTV.textContainerInset.width,
                trailingPadding: trailingPadding,
                width: &rightWidth
            )
        }

        if rows.isEmpty {
            let attrs: [NSAttributedString.Key: Any] = [
                .font: font,
                .foregroundColor: NSColor.secondaryLabelColor
            ]
            leftText.append(NSAttributedString(string: "No differences", attributes: attrs))
            leftGutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
            rightGutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
        }

        leftLayout.lineBgColors = leftColors
        rightLayout.lineBgColors = rightColors
        leftGutterLayout.lineBgColors = leftColors
        rightGutterLayout.lineBgColors = rightColors
        leftTV.textStorage?.setAttributedString(leftText)
        rightTV.textStorage?.setAttributedString(rightText)
        leftGutterTV.textStorage?.setAttributedString(leftGutter)
        rightGutterTV.textStorage?.setAttributedString(rightGutter)
        leftGutterTV.entries = leftEntries
        rightGutterTV.entries = rightEntries
        leftContainer.updateGutterWidth(max(52, leftWidth))
        rightContainer.updateGutterWidth(max(52, rightWidth))
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
