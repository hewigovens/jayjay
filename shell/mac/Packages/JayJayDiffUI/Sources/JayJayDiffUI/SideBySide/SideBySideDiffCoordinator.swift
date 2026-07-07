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

    func renderIfNeeded(force: Bool) {
        guard let diff,
              let font,
              let theme,
              let left = leftContainer?.paneViews(),
              let right = rightContainer?.paneViews()
        else { return }

        // Pane content widths (post-gutter) and monospace cell advance.
        let advance = Float(("M" as NSString).size(withAttributes: [.font: font]).width)
        let oldCols = wrapColsForWidth(
            width: Float(max(0, left.container.scrollView.contentSize.width)),
            advance: advance
        )
        let newCols = wrapColsForWidth(
            width: Float(max(0, right.container.scrollView.contentSize.width)),
            advance: advance
        )
        if !force, oldCols == lastOldCols, newCols == lastNewCols {
            return
        }
        lastOldCols = oldCols
        lastNewCols = newCols

        // Pre-wrap into visual rows so both panes (and gutters) advance in lock-step.
        let rows = buildSideBySideRows(lines: diff.lines)
        let visualRows = wrapSbsRows(rows: rows, oldCols: oldCols, newCols: newCols).map(\.row)
        renderVisualRows(visualRows, font: font, theme: theme, left: left, right: right)
    }

    private func renderVisualRows(
        _ rows: [SideBySideRow],
        font: NSFont,
        theme: DiffColors,
        left: PaneViews,
        right: PaneViews
    ) {
        let gutterParagraphStyle = NSMutableParagraphStyle()
        gutterParagraphStyle.alignment = .right
        let gutterAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: theme.gutterText,
            .paragraphStyle: gutterParagraphStyle
        ]
        let trailingPadding: CGFloat = 10
        left.textView.applyFindSelectionColors(theme)
        left.textLayout.selectedRangeBgColor = .selectedTextBackgroundColor
        left.textLayout.findMatchBgColor = .findHighlightColor
        right.textView.applyFindSelectionColors(theme)
        right.textLayout.selectedRangeBgColor = .selectedTextBackgroundColor
        right.textLayout.findMatchBgColor = .findHighlightColor

        var leftAcc = PaneAccumulator(pane: left)
        var rightAcc = PaneAccumulator(pane: right)
        for row in rows {
            leftAcc.append(
                row.old,
                font: font,
                theme: theme,
                gutterAttrs: gutterAttrs,
                trailingPadding: trailingPadding
            )
            rightAcc.append(
                row.new,
                font: font,
                theme: theme,
                gutterAttrs: gutterAttrs,
                trailingPadding: trailingPadding
            )
        }

        if rows.isEmpty {
            let attrs: [NSAttributedString.Key: Any] = [
                .font: font,
                .foregroundColor: NSColor.secondaryLabelColor
            ]
            leftAcc.text.append(NSAttributedString(string: "No differences", attributes: attrs))
            leftAcc.gutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
            rightAcc.gutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
        }

        leftAcc.commit()
        rightAcc.commit()
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

/// Mutable accumulator for one pane's render — owns the attributed buffers,
/// gutter entries, line colors, and committed gutter width. `commit()` writes
/// everything back to the pane's text and gutter views.
private struct PaneAccumulator {
    let pane: PaneViews
    let text = NSMutableAttributedString()
    let gutter = NSMutableAttributedString()
    var entries: [DiffGutterTextView.Entry] = []
    var width: CGFloat = 0
    var colors: [NSColor] = []
    var stripes: [NSColor] = []

    mutating func append(
        _ side: RowSide,
        font: NSFont,
        theme: DiffColors,
        gutterAttrs: [NSAttributedString.Key: Any],
        trailingPadding: CGFloat
    ) {
        appendTextLine(
            to: text,
            spans: side.spans,
            style: side.style,
            conflictKind: side.conflictKind,
            font: font,
            theme: theme,
            bgColors: &colors
        )
        stripes.append(conflictStripe(conflictKind: side.conflictKind, theme: theme))
        appendGutterLine(
            to: gutter,
            entries: &entries,
            lineNo: side.lineNo,
            style: side.style,
            attrs: gutterAttrs,
            inset: pane.gutterTextView.textContainerInset.width,
            trailingPadding: trailingPadding,
            width: &width
        )
    }

    func commit() {
        pane.textLayout.lineBgColors = colors
        pane.textLayout.lineStripeColors = stripes
        pane.textLayout.lineStripeX = 0
        pane.textLayout.lineStripeWidth = 3
        pane.gutterLayout.lineBgColors = colors
        pane.gutterLayout.lineStripeColors = stripes
        pane.gutterLayout.lineStripeX = 0
        pane.gutterLayout.lineStripeWidth = 3
        pane.textView.textStorage?.setAttributedString(text)
        pane.gutterTextView.textStorage?.setAttributedString(gutter)
        pane.gutterTextView.entries = entries
        pane.container.updateGutterWidth(max(DiffGutterMetrics.minimumUnifiedWidth, width))
    }
}
