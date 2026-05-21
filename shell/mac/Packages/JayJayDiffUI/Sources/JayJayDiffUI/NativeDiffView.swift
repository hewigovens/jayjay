import AppKit
import JayJayCore
import SwiftUI

public struct NativeDiffView: NSViewRepresentable {
    public let diff: FileDiff
    public var gutterActions: (any DiffGutterContextActions)?

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.diffFontSize) private var fontSize
    @Environment(\.diffFontFamily) private var fontFamily

    public init(diff: FileDiff, gutterActions: (any DiffGutterContextActions)? = nil) {
        self.diff = diff
        self.gutterActions = gutterActions
    }

    public func makeNSView(context: Context) -> DiffTextContainerView {
        let gutterContainer = NSTextContainer(
            containerSize: NSSize(width: 0, height: CGFloat.greatestFiniteMagnitude)
        )
        gutterContainer.widthTracksTextView = true
        gutterContainer.lineFragmentPadding = 0

        let gutterLayoutManager = DiffLayoutManager()
        gutterLayoutManager.addTextContainer(gutterContainer)

        let gutterStorage = NSTextStorage()
        gutterStorage.addLayoutManager(gutterLayoutManager)

        let gutterScrollView = NSScrollView()
        gutterScrollView.hasVerticalScroller = false
        gutterScrollView.hasHorizontalScroller = false
        gutterScrollView.autohidesScrollers = true
        gutterScrollView.drawsBackground = false

        let gutterTextView = DiffGutterTextView(frame: gutterScrollView.bounds, textContainer: gutterContainer)
        gutterTextView.isEditable = false
        gutterTextView.isSelectable = true
        gutterTextView.isVerticallyResizable = true
        gutterTextView.isHorizontallyResizable = false
        gutterTextView.autoresizingMask = [.width]
        gutterTextView.textContainerInset = NSSize(width: 8, height: 8)
        gutterTextView.drawsBackground = false
        gutterTextView.minSize = NSSize(width: 0, height: 0)
        gutterTextView.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        gutterScrollView.documentView = gutterTextView

        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false

        let textContainer = NSTextContainer(containerSize: NSSize(
            width: 0,
            height: CGFloat.greatestFiniteMagnitude
        ))
        textContainer.widthTracksTextView = true
        textContainer.lineFragmentPadding = 4
        textContainer.lineBreakMode = .byWordWrapping

        let layoutManager = DiffLayoutManager()
        layoutManager.addTextContainer(textContainer)

        let storage = NSTextStorage()
        storage.addLayoutManager(layoutManager)

        let textView = NSTextView(frame: scrollView.bounds, textContainer: textContainer)
        textView.isEditable = false
        textView.isSelectable = true
        textView.autoresizingMask = [.width]
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.textContainerInset = NSSize(width: 4, height: 8)
        textView.drawsBackground = false
        textView.usesFindBar = true
        textView.isIncrementalSearchingEnabled = true
        textView.identifier = NSUserInterfaceItemIdentifier("diffTextView")
        textView.minSize = NSSize(width: 0, height: 0)
        textView.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)

        scrollView.documentView = textView
        return DiffTextContainerView(
            gutterScrollView: gutterScrollView,
            gutterTextView: gutterTextView,
            scrollView: scrollView,
            textView: textView
        )
    }

    public func updateNSView(_ containerView: DiffTextContainerView, context: Context) {
        let gutterTextView = containerView.gutterTextView
        let textView = containerView.textView
        guard let gutterLayoutManager = gutterTextView.layoutManager as? DiffLayoutManager,
              let layoutManager = textView.layoutManager as? DiffLayoutManager
        else { return }

        let fontSize = fontSize
        let font = NSFont(name: fontFamily, size: fontSize) ?? .monospacedSystemFont(ofSize: fontSize, weight: .regular)
        let isDark = colorScheme == .dark
        let theme = DiffColors(isDark: isDark)
        let selectionActions = gutterActions as? any DiffGutterSelectionActions
        let reviewActions = gutterActions as? any DiffGutterReviewActions

        let gutterParagraphStyle = NSMutableParagraphStyle()
        let showsLineCheckboxes = selectionActions != nil
        let showsReviewCheckboxes = !showsLineCheckboxes && (reviewActions?.reviewCheckboxesEnabled == true)
        // Only diff-edit's per-line checkbox column needs its own slot; the
        // review pill lives in the always-present leftmost (group) column.
        let showsCheckboxColumn = showsLineCheckboxes
        gutterParagraphStyle.alignment = .left
        let gutterAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: theme.gutterText,
            .paragraphStyle: gutterParagraphStyle
        ]

        // First-line map drives the ✓ glyph; per-line map drives click hit-test.
        var firstLineOfGroup: [Int: UInt32] = [:]
        var groupIndexAtLineNumber: [Int: UInt32] = [:]
        var currentGroup: UInt32 = 0
        var inGroup = false
        for (index, line) in diff.lines.enumerated() {
            let isChanged = line.style == .added || line.style == .removed
            if isChanged {
                if !inGroup {
                    firstLineOfGroup[index + 1] = currentGroup
                    inGroup = true
                }
                groupIndexAtLineNumber[index + 1] = currentGroup
            } else if inGroup {
                inGroup = false
                currentGroup += 1
            }
        }

        let result = NSMutableAttributedString()
        // Review mode reserves a third char so the ✓ glyph fits.
        let leftColumnText = showsReviewCheckboxes ? " ✓ " : "  "
        let groupWidth = (leftColumnText as NSString).size(withAttributes: [.font: font]).width
        let groupStripeWidth: CGFloat = 6
        let checkboxWidth = max(
            ("✓ " as NSString).size(withAttributes: [.font: font]).width,
            ("□ " as NSString).size(withAttributes: [.font: font]).width
        )
        let gutterHorizontalInset = gutterTextView.textContainerInset.width
        let gutterTrailingPadding: CGFloat = 10
        let maxLineDigits = diff.lines.reduce(into: 1) { digits, line in
            let lineNumber = max(line.oldLineNo ?? 0, line.newLineNo ?? 0)
            digits = max(digits, String(lineNumber).count)
        }
        var contentLineBgColors: [NSColor] = []

        for line in diff.lines {
            if line.style == .separator {
                result.append(NSAttributedString(string: "⋯ \(line.spans.first?.text ?? "")\n", attributes: [
                    .font: font, .foregroundColor: theme.gutterText
                ]))
                contentLineBgColors.append(theme.separatorBg)
                continue
            }

            // Content spans with word-level highlighting
            for span in line.spans {
                let foreground = theme.tokenColor(span.token, fallback: theme.lineText(line.style))
                var attrs: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: foreground]
                let wordBg = spanBackground(span: span, theme: theme)
                if wordBg != .clear { attrs[.backgroundColor] = wordBg }
                result.append(NSAttributedString(string: span.text, attributes: attrs))
            }
            if line.spans.isEmpty {
                result.append(NSAttributedString(string: " ", attributes: [.font: font]))
            }
            if line.noEofNewline {
                let dim: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: theme.gutterText]
                result.append(NSAttributedString(string: "  ⊘", attributes: dim))
                var arrowAttrs = dim
                // ↵ sits lower than ⊘ in most monospace fonts; nudge it up to match visual center.
                arrowAttrs[.baselineOffset] = 1.5
                result.append(NSAttributedString(string: "↵", attributes: arrowAttrs))
                result.append(NSAttributedString(string: "  no newline at EOF", attributes: dim))
            }
            result.append(NSAttributedString(string: "\n", attributes: [.font: font]))
            contentLineBgColors.append(theme.lineBg(line.style))
        }

        if diff.lines.isEmpty {
            result.append(NSAttributedString(
                string: "No differences",
                attributes: [.font: font, .foregroundColor: NSColor.secondaryLabelColor]
            ))
            contentLineBgColors.append(.clear)
        }

        layoutManager.lineBgColors = contentLineBgColors
        layoutManager.lineStripeColors = []
        layoutManager.lineStripeWidth = 0
        gutterTextView.menuProvider = menuProvider(selection:)
        gutterTextView.groupRangeProvider = { lineNumber in
            expandedHunkRange(containing: lineNumber ... lineNumber)
        }
        if let selectionActions {
            gutterTextView.activateGroup = { selectionActions.selectChangeGroup($0) }
        } else {
            gutterTextView.activateGroup = nil
        }
        gutterTextView.groupHitWidth = groupWidth
        gutterTextView.toggleLineCheckbox = selectionActions.map { actions in
            { actions.toggleLineCheckbox($0) }
        }
        gutterTextView.checkboxHitStart = groupWidth
        gutterTextView.checkboxHitWidth = showsCheckboxColumn ? checkboxWidth : 0
        if showsReviewCheckboxes, let reviewActions {
            gutterTextView.groupIndexAtLineNumber = groupIndexAtLineNumber
            gutterTextView.toggleReviewCheckbox = { groupIdx in
                reviewActions.toggleHunkReviewed(groupIndex: groupIdx)
            }
        } else {
            gutterTextView.groupIndexAtLineNumber = [:]
            gutterTextView.toggleReviewCheckbox = nil
        }
        gutterTextView.onSelectionChanged = { selection in
            gutterActions?.didSelectLines(selection.lineRange)
        }
        textView.textStorage?.setAttributedString(result)

        let renderGutter = { [weak containerView] in
            guard let containerView else { return }

            let logicalLineCount = max(diff.lines.count, 1)
            let gutterWidth = renderWrappedGutter(
                gutterTextView: gutterTextView,
                gutterLayoutManager: gutterLayoutManager,
                context: NativeDiffGutterRenderContext(
                    visualLineCounts: layoutManager.visualLineCounts(logicalLineCount: logicalLineCount),
                    font: font,
                    theme: theme,
                    gutterAttrs: gutterAttrs,
                    gutterParagraphStyle: gutterParagraphStyle,
                    maxLineDigits: maxLineDigits,
                    showsReviewCheckboxes: showsReviewCheckboxes,
                    showsCheckboxColumn: showsCheckboxColumn,
                    firstLineOfGroup: firstLineOfGroup,
                    groupIndexAtLineNumber: groupIndexAtLineNumber,
                    reviewActions: reviewActions,
                    groupStripeWidth: groupStripeWidth,
                    gutterHorizontalInset: gutterHorizontalInset,
                    gutterTrailingPadding: gutterTrailingPadding,
                    currentSelectedLineRange: gutterActions?.currentSelectedLineRange
                )
            )
            containerView.updateGutterWidth(max(52, gutterWidth))
        }

        containerView.onContentLayoutChanged = renderGutter
        renderGutter()
    }
}
