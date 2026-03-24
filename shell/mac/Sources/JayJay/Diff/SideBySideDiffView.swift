import AppKit
import JayJayCore
import SwiftUI

/// GitHub Desktop-style two-column diff: left = old, right = new, synced scroll.
struct SideBySideDiffView: View {
    let diff: FileDiff

    var body: some View {
        SideBySideRepresentable(diff: diff)
    }
}

private struct SideBySideRepresentable: NSViewRepresentable {
    let diff: FileDiff

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontSize) private var fontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> NSSplitView {
        let split = NSSplitView()
        split.isVertical = true
        split.dividerStyle = .thin
        split.delegate = context.coordinator

        let left = makeContainer()
        let right = makeContainer()
        split.addSubview(left)
        split.addSubview(right)

        context.coordinator.leftContainer = left
        context.coordinator.rightContainer = right
        context.coordinator.startObserving()

        return split
    }

    func updateNSView(_ split: NSSplitView, context: Context) {
        guard let leftContainer = context.coordinator.leftContainer,
              let rightContainer = context.coordinator.rightContainer,
              let leftTV = leftContainer.textView as NSTextView?,
              let rightTV = rightContainer.textView as NSTextView?,
              let leftGutterTV = leftContainer.gutterTextView as DiffGutterTextView?,
              let rightGutterTV = rightContainer.gutterTextView as DiffGutterTextView?,
              let leftLayout = leftTV.layoutManager as? DiffLayoutManager,
              let rightLayout = rightTV.layoutManager as? DiffLayoutManager,
              let leftGutterLayout = leftGutterTV.layoutManager as? DiffLayoutManager,
              let rightGutterLayout = rightGutterTV.layoutManager as? DiffLayoutManager
        else { return }

        let font = fontFamily.nsFont(size: fontSize)
        let theme = DiffColors(isDark: colorScheme == .dark)
        let rows = buildRows(from: diff.lines)

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
        let markerWidth = ("+" as NSString).size(withAttributes: [.font: font]).width
        let gutterGap: CGFloat = 10
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
                marker: row.oldMarker,
                style: row.oldStyle,
                attrs: gutterAttrs,
                font: font,
                markerWidth: markerWidth,
                inset: leftGutterTV.textContainerInset.width,
                gap: gutterGap,
                trailingPadding: trailingPadding,
                width: &leftWidth,
                theme: theme
            )
            appendGutterLine(
                to: rightGutter,
                entries: &rightEntries,
                lineNo: row.newLineNo,
                marker: row.newMarker,
                style: row.newStyle,
                attrs: gutterAttrs,
                font: font,
                markerWidth: markerWidth,
                inset: rightGutterTV.textContainerInset.width,
                gap: gutterGap,
                trailingPadding: trailingPadding,
                width: &rightWidth,
                theme: theme
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

    private func makeContainer() -> DiffTextContainerView {
        let gutterContainer = NSTextContainer(
            containerSize: NSSize(width: 0, height: CGFloat.greatestFiniteMagnitude)
        )
        gutterContainer.widthTracksTextView = true
        gutterContainer.lineFragmentPadding = 0

        let gutterLayout = DiffLayoutManager()
        gutterLayout.addTextContainer(gutterContainer)

        let gutterStorage = NSTextStorage()
        gutterStorage.addLayoutManager(gutterLayout)

        let gutterScrollView = NSScrollView()
        gutterScrollView.hasVerticalScroller = false
        gutterScrollView.hasHorizontalScroller = false
        gutterScrollView.autohidesScrollers = true
        gutterScrollView.drawsBackground = false

        let gutterTextView = DiffGutterTextView(frame: gutterScrollView.bounds, textContainer: gutterContainer)
        gutterTextView.isEditable = false
        gutterTextView.isSelectable = false
        gutterTextView.isVerticallyResizable = true
        gutterTextView.isHorizontallyResizable = false
        gutterTextView.autoresizingMask = [.width]
        gutterTextView.textContainerInset = NSSize(width: 8, height: 6)
        gutterTextView.drawsBackground = false
        gutterTextView.minSize = NSSize(width: 0, height: 0)
        gutterTextView.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        gutterScrollView.documentView = gutterTextView

        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false

        let textContainer = NSTextContainer(
            containerSize: NSSize(width: 0, height: CGFloat.greatestFiniteMagnitude)
        )
        textContainer.widthTracksTextView = true
        textContainer.lineFragmentPadding = 0

        let layoutManager = DiffLayoutManager()
        layoutManager.addTextContainer(textContainer)

        let storage = NSTextStorage()
        storage.addLayoutManager(layoutManager)

        let textView = NSTextView(frame: scrollView.bounds, textContainer: textContainer)
        textView.isEditable = false
        textView.isSelectable = true
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.autoresizingMask = [.width]
        textView.textContainerInset = NSSize(width: 4, height: 6)
        textView.drawsBackground = false
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

    private func appendTextLine(
        to str: NSMutableAttributedString,
        spans: [DiffSpan],
        style: DiffSpanStyle,
        font: NSFont,
        theme: DiffColors,
        bgColors: inout [NSColor]
    ) {
        if style == .separator {
            str.append(NSAttributedString(string: "⋯ \(spans.first?.text ?? "")\n", attributes: [
                .font: font,
                .foregroundColor: theme.gutterText
            ]))
            bgColors.append(theme.separatorBg)
            return
        }

        if spans.isEmpty {
            str.append(NSAttributedString(string: "\n", attributes: [.font: font]))
        } else {
            for span in spans {
                let foreground = tokenColor(
                    span.token,
                    fallback: style == .added ? theme.addedText : style == .removed ? theme.removedText : theme
                        .contextText,
                    theme: theme
                )
                var attrs: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: foreground]
                switch span.style {
                    case .added:
                        attrs[.backgroundColor] = theme.addedWordBg
                    case .removed:
                        attrs[.backgroundColor] = theme.removedWordBg
                    default:
                        break
                }
                str.append(NSAttributedString(string: span.text, attributes: attrs))
            }
            str.append(NSAttributedString(string: "\n", attributes: [.font: font]))
        }

        bgColors.append(lineBg(style, theme: theme))
    }

    private func appendGutterLine(
        to str: NSMutableAttributedString,
        entries: inout [DiffGutterTextView.Entry],
        lineNo: String,
        marker: String,
        style: DiffSpanStyle,
        attrs: [NSAttributedString.Key: Any],
        font: NSFont,
        markerWidth: CGFloat,
        inset: CGFloat,
        gap: CGFloat,
        trailingPadding: CGFloat,
        width: inout CGFloat,
        theme: DiffColors
    ) {
        if style == .separator {
            let start = str.length
            str.append(NSAttributedString(string: "\n", attributes: attrs))
            entries.append(.init(style: style, range: NSRange(location: start, length: str.length - start)))
            return
        }

        let padded = lineNo.isEmpty ? "" : lineNo
        let markerColor = marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText
        let line = NSMutableAttributedString(string: padded, attributes: attrs)
        let spacing = padded.isEmpty ? "" : " "
        line.append(NSAttributedString(string: spacing, attributes: attrs))
        line.append(NSAttributedString(string: marker, attributes: [
            .font: font,
            .foregroundColor: markerColor
        ]))
        line.append(NSAttributedString(string: "\n", attributes: attrs))
        let start = str.length
        str.append(line)
        entries.append(.init(style: style, range: NSRange(location: start, length: str.length - start)))

        let numberWidth = (padded as NSString).size(withAttributes: attrs).width
        width = max(width, ceil(inset + numberWidth + gap + markerWidth + trailingPadding + inset))
    }

    private func lineBg(_ style: DiffSpanStyle, theme: DiffColors) -> NSColor {
        switch style {
            case .added:
                theme.addedBg
            case .removed:
                theme.removedBg
            case .separator:
                theme.separatorBg
            default:
                .clear
        }
    }

    private func tokenColor(_ token: SyntaxToken, fallback: NSColor, theme: DiffColors) -> NSColor {
        switch token {
            case .comment:
                theme.comment
            case .keyword, .operator:
                theme.keyword
            case .stringLit:
                theme.string
            case .number:
                theme.number
            case .type, .function, .attribute:
                theme.type
            default:
                fallback
        }
    }

    final class Coordinator: NSObject, NSSplitViewDelegate {
        weak var leftContainer: DiffTextContainerView?
        weak var rightContainer: DiffTextContainerView?
        private var syncing = false

        func splitView(
            _ splitView: NSSplitView,
            constrainMinCoordinate proposedMinimumPosition: CGFloat,
            ofSubviewAt dividerIndex: Int
        ) -> CGFloat {
            100
        }

        func splitView(_ splitView: NSSplitView, resizeSubviewsWithOldSize oldSize: NSSize) {
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
}

// MARK: - Row model

private struct SBSRow {
    var oldLineNo: String
    var oldMarker: String
    var oldSpans: [DiffSpan]
    var oldStyle: DiffSpanStyle
    var newLineNo: String
    var newMarker: String
    var newSpans: [DiffSpan]
    var newStyle: DiffSpanStyle
}

private func buildRows(from lines: [DiffLine]) -> [SBSRow] {
    var rows: [SBSRow] = []
    var i = 0
    while i < lines.count {
        let line = lines[i]
        switch line.style {
            case .context:
                rows.append(SBSRow(
                    oldLineNo: line.oldLineNo.map(String.init) ?? "",
                    oldMarker: " ",
                    oldSpans: line.spans,
                    oldStyle: .context,
                    newLineNo: line.newLineNo.map(String.init) ?? "",
                    newMarker: " ",
                    newSpans: line.spans,
                    newStyle: .context
                ))
                i += 1
            case .separator:
                rows.append(SBSRow(
                    oldLineNo: "",
                    oldMarker: "",
                    oldSpans: line.spans,
                    oldStyle: .separator,
                    newLineNo: "",
                    newMarker: "",
                    newSpans: line.spans,
                    newStyle: .separator
                ))
                i += 1
            case .removed:
                var removed: [DiffLine] = []
                while i < lines.count, lines[i].style == .removed {
                    removed.append(lines[i])
                    i += 1
                }
                var added: [DiffLine] = []
                while i < lines.count, lines[i].style == .added {
                    added.append(lines[i])
                    i += 1
                }
                for j in 0 ..< max(removed.count, added.count) {
                    let removedLine = j < removed.count ? removed[j] : nil
                    let addedLine = j < added.count ? added[j] : nil
                    rows.append(SBSRow(
                        oldLineNo: removedLine?.oldLineNo.map(String.init) ?? "",
                        oldMarker: removedLine != nil ? "-" : " ",
                        oldSpans: removedLine?.spans ?? [],
                        oldStyle: removedLine != nil ? .removed : .context,
                        newLineNo: addedLine?.newLineNo.map(String.init) ?? "",
                        newMarker: addedLine != nil ? "+" : " ",
                        newSpans: addedLine?.spans ?? [],
                        newStyle: addedLine != nil ? .added : .context
                    ))
                }
            case .added:
                rows.append(SBSRow(
                    oldLineNo: "",
                    oldMarker: " ",
                    oldSpans: [],
                    oldStyle: .context,
                    newLineNo: line.newLineNo.map(String.init) ?? "",
                    newMarker: "+",
                    newSpans: line.spans,
                    newStyle: .added
                ))
                i += 1
            default:
                i += 1
        }
    }
    return rows
}
