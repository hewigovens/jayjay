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
        scrollView.hasHorizontalScroller = true
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false

        // No-wrap: wrapping would split one diff line into multiple visual rows and break gutter↔content alignment.
        let textContainer = NSTextContainer(containerSize: NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        ))
        textContainer.widthTracksTextView = false
        textContainer.lineFragmentPadding = 4

        let layoutManager = DiffLayoutManager()
        layoutManager.addTextContainer(textContainer)

        let storage = NSTextStorage()
        storage.addLayoutManager(layoutManager)

        let textView = NSTextView(frame: scrollView.bounds, textContainer: textContainer)
        textView.isEditable = false
        textView.isSelectable = true
        textView.autoresizingMask = [.width]
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = true
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

        let gutterParagraphStyle = NSMutableParagraphStyle()
        let showsLineCheckboxes = selectionActions != nil
        gutterParagraphStyle.alignment = .left
        let gutterAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: theme.gutterText,
            .paragraphStyle: gutterParagraphStyle
        ]

        let result = NSMutableAttributedString()
        let gutter = NSMutableAttributedString()
        var gutterEntries: [DiffGutterTextView.Entry] = []
        var gutterWidth: CGFloat = 0
        let groupWidth = ("  " as NSString).size(withAttributes: [.font: font]).width
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
        var lineBgColors: [NSColor] = []
        var groupStripeColors: [NSColor] = []

        for (index, line) in diff.lines.enumerated() {
            if line.style == .separator {
                result.append(NSAttributedString(string: "⋯ \(line.spans.first?.text ?? "")\n", attributes: [
                    .font: font, .foregroundColor: theme.gutterText
                ]))
                let gutterStart = gutter.length
                gutter.append(NSAttributedString(
                    string: separatorGutterText(
                        maxLineDigits: maxLineDigits,
                        showsLineCheckboxes: showsLineCheckboxes
                    ),
                    attributes: gutterAttrs
                ))
                gutterEntries.append(.init(
                    style: line.style,
                    range: NSRange(location: gutterStart, length: gutter.length - gutterStart)
                ))
                lineBgColors.append(theme.separatorBg)
                groupStripeColors.append(.clear)
                continue
            }

            let marker = switch line.style {
                case .added: "+"
                case .removed: "-"
                default: " "
            }
            let markerColor = marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText
            let gutterLine = NSMutableAttributedString(
                string: groupText(),
                attributes: gutterAttrs
            )
            if showsLineCheckboxes {
                gutterLine.append(NSAttributedString(
                    string: checkboxText(for: index + 1, line: line),
                    attributes: [
                        .font: font,
                        .foregroundColor: checkboxColor(for: index + 1, theme: theme),
                        .paragraphStyle: gutterParagraphStyle
                    ]
                ))
            }
            gutterLine.append(NSAttributedString(
                string: pad(line.oldLineNo.map(String.init) ?? "", toWidth: maxLineDigits),
                attributes: gutterAttrs
            ))
            gutterLine.append(NSAttributedString(string: " ", attributes: gutterAttrs))
            gutterLine.append(NSAttributedString(
                string: pad(line.newLineNo.map(String.init) ?? "", toWidth: maxLineDigits),
                attributes: gutterAttrs
            ))
            gutterLine.append(NSAttributedString(string: " ", attributes: gutterAttrs))
            gutterLine.append(NSAttributedString(string: marker, attributes: [
                .font: font,
                .foregroundColor: markerColor
            ]))
            gutterLine.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
            let gutterStart = gutter.length
            gutter.append(gutterLine)
            gutterEntries.append(.init(
                style: line.style,
                range: NSRange(location: gutterStart, length: gutter.length - gutterStart)
            ))

            let gutterLineWidth = gutterLine.size().width
            gutterWidth = max(
                gutterWidth,
                ceil(
                    gutterHorizontalInset +
                        gutterLineWidth +
                        gutterTrailingPadding +
                        gutterHorizontalInset
                )
            )

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
            lineBgColors.append(theme.lineBg(line.style))
            let displayLine = index + 1
            groupStripeColors.append(groupStripeColor(
                for: line,
                groupRange: expandedHunkRange(containing: displayLine ... displayLine),
                theme: theme
            ))
        }

        if diff.lines.isEmpty {
            result.append(NSAttributedString(
                string: "No differences",
                attributes: [.font: font, .foregroundColor: NSColor.secondaryLabelColor]
            ))
            let gutterStart = gutter.length
            gutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
            gutterEntries.append(.init(
                style: .context,
                range: NSRange(location: gutterStart, length: gutter.length - gutterStart)
            ))
        }

        gutterLayoutManager.lineBgColors = lineBgColors
        gutterLayoutManager.lineStripeColors = groupStripeColors
        gutterLayoutManager.lineStripeX = 0
        gutterLayoutManager.lineStripeWidth = groupStripeWidth
        layoutManager.lineBgColors = lineBgColors
        layoutManager.lineStripeColors = []
        layoutManager.lineStripeWidth = 0
        gutterTextView.textStorage?.setAttributedString(gutter)
        gutterTextView.entries = gutterEntries
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
        gutterTextView.checkboxHitWidth = showsLineCheckboxes ? checkboxWidth : 0
        gutterTextView.onSelectionChanged = { selection in
            gutterActions?.didSelectLines(selection.lineRange)
        }
        gutterTextView.externalSelection = gutterActions?.currentSelectedLineRange
        textView.textStorage?.setAttributedString(result)
        containerView.updateGutterWidth(max(52, gutterWidth))
    }
}
