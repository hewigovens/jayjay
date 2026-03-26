import AppKit
import JayJayCore
import SwiftUI

struct DiffGutterContextActions {
    let splitFile: (() -> Void)?
    let moveToWorkingCopy: (() -> Void)?
    let restoreFile: (() -> Void)?
    let abandonChange: (() -> Void)?
    var openDiffEdit: (() -> Void)? = nil
    var selectFile: (() -> Void)? = nil
    var selectHunk: ((ClosedRange<Int>) -> Void)? = nil
    var onLineSelectionChanged: ((ClosedRange<Int>) -> Void)? = nil
    var selectedLineRange: ClosedRange<Int>? = nil
}

struct NativeDiffView: NSViewRepresentable {
    let diff: FileDiff
    var gutterActions: DiffGutterContextActions?

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontSize) private var fontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    func makeNSView(context: Context) -> DiffTextContainerView {
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

        let textContainer = NSTextContainer(containerSize: NSSize(width: 0, height: CGFloat.greatestFiniteMagnitude))
        textContainer.widthTracksTextView = true
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

    func updateNSView(_ containerView: DiffTextContainerView, context: Context) {
        let gutterTextView = containerView.gutterTextView
        let textView = containerView.textView
        guard let gutterLayoutManager = gutterTextView.layoutManager as? DiffLayoutManager,
              let layoutManager = textView.layoutManager as? DiffLayoutManager
        else { return }

        let fontSize = fontSize
        let font = fontFamily.nsFont(size: fontSize)
        let isDark = colorScheme == .dark
        let theme = DiffColors(isDark: isDark)

        let gutterParagraphStyle = NSMutableParagraphStyle()
        gutterParagraphStyle.alignment = .right
        let gutterAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: theme.gutterText,
            .paragraphStyle: gutterParagraphStyle
        ]

        let result = NSMutableAttributedString()
        let gutter = NSMutableAttributedString()
        var gutterEntries: [DiffGutterTextView.Entry] = []
        var gutterWidth: CGFloat = 0
        let markerWidth = ("+" as NSString).size(withAttributes: [.font: font]).width
        let gutterHorizontalInset = gutterTextView.textContainerInset.width
        let gutterGap: CGFloat = 10
        let gutterTrailingPadding: CGFloat = 10
        var lineBgColors: [NSColor] = []

        for line in diff.lines {
            if line.style == .separator {
                result.append(NSAttributedString(string: "⋯ \(line.spans.first?.text ?? "")\n", attributes: [
                    .font: font, .foregroundColor: theme.gutterText
                ]))
                let gutterStart = gutter.length
                gutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
                gutterEntries.append(.init(
                    style: line.style,
                    range: NSRange(location: gutterStart, length: gutter.length - gutterStart)
                ))
                lineBgColors.append(theme.separatorBg)
                continue
            }

            let lineNumber = (line.newLineNo ?? line.oldLineNo).map(String.init) ?? ""
            let marker = switch line.style {
                case .added: "+"
                case .removed: "-"
                default: " "
            }
            let padded = lineNumber.isEmpty ? "" : lineNumber
            let markerColor = marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText
            let gutterLine = NSMutableAttributedString(
                string: padded,
                attributes: gutterAttrs
            )
            let gap = padded.isEmpty ? "" : " "
            gutterLine.append(NSAttributedString(string: gap, attributes: gutterAttrs))
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

            let numberWidth = (padded as NSString).size(withAttributes: gutterAttrs).width
            gutterWidth = max(
                gutterWidth,
                ceil(
                    gutterHorizontalInset +
                        numberWidth +
                        gutterGap +
                        markerWidth +
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
            result.append(NSAttributedString(string: "\n", attributes: [.font: font]))
            lineBgColors.append(theme.lineBg(line.style))
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
        layoutManager.lineBgColors = lineBgColors
        gutterTextView.textStorage?.setAttributedString(gutter)
        gutterTextView.entries = gutterEntries
        gutterTextView.menuProvider = menuProvider(selection:)
        gutterTextView.onSelectionChanged = { selection in
            gutterActions?.onLineSelectionChanged?(selection.lineRange)
        }
        gutterTextView.externalSelection = gutterActions?.selectedLineRange
        textView.textStorage?.setAttributedString(result)
        containerView.updateGutterWidth(max(52, gutterWidth))
    }

    private func menuProvider(selection: DiffGutterSelection) -> [DiffGutterMenuItem] {
        guard let gutterActions else { return [] }

        var items: [DiffGutterMenuItem] = []
        if let selectHunk = gutterActions.selectHunk,
           let hunkRange = expandedHunkRange(containing: selection.lineRange)
        {
            items.append(
                DiffGutterMenuItem(
                    title: "Select Hunk",
                    enabled: true,
                    action: { selectHunk(hunkRange) }
                )
            )
        }
        if let selectFile = gutterActions.selectFile {
            items.append(DiffGutterMenuItem(title: "Select File", enabled: true, action: selectFile))
        }
        if let openDiffEdit = gutterActions.openDiffEdit {
            items.append(
                DiffGutterMenuItem(title: "Open Diff Edit Mode", enabled: true, action: openDiffEdit)
            )
        }
        if !items.isEmpty,
           gutterActions.splitFile != nil || gutterActions.moveToWorkingCopy != nil || gutterActions.restoreFile != nil
                || gutterActions.abandonChange != nil
        {
            items.append(.separator)
        }

        let splitTitle = selection.changedLineCount > 0
            ? "Split Selected Lines"
            : "Split Selected Lines"
        items.append(DiffGutterMenuItem(title: splitTitle, enabled: false, action: nil))

        if let splitFile = gutterActions.splitFile {
            items.append(DiffGutterMenuItem(title: "Split File to New Change", enabled: true, action: splitFile))
        }
        if let moveToWorkingCopy = gutterActions.moveToWorkingCopy {
            items.append(DiffGutterMenuItem(
                title: "Move File to Working Copy",
                enabled: true,
                action: moveToWorkingCopy
            ))
        }
        if let restoreFile = gutterActions.restoreFile {
            items.append(DiffGutterMenuItem(title: "Restore File to Parent", enabled: true, action: restoreFile))
        }
        if let abandonChange = gutterActions.abandonChange {
            if items.last?.action != nil {
                items.append(.separator)
            }
            items.append(DiffGutterMenuItem(title: "Abandon Change", enabled: true, action: abandonChange))
        }

        return items
    }

    private func expandedHunkRange(containing selection: ClosedRange<Int>) -> ClosedRange<Int>? {
        guard !diff.lines.isEmpty else { return nil }
        let isChanged: (DiffLine) -> Bool = { line in
            line.style == .added || line.style == .removed
        }

        let anchor = selection.lowerBound - 1
        guard diff.lines.indices.contains(anchor), isChanged(diff.lines[anchor]) else {
            return selection
        }

        var lower = anchor
        while lower > 0, isChanged(diff.lines[lower - 1]) {
            lower -= 1
        }

        var upper = anchor
        while upper + 1 < diff.lines.count, isChanged(diff.lines[upper + 1]) {
            upper += 1
        }

        return (lower + 1) ... (upper + 1)
    }

    private func spanBackground(span: DiffSpan, theme: DiffColors) -> NSColor {
        switch span.style {
            case .added: theme.addedWordBg
            case .removed: theme.removedWordBg
            default: .clear
        }
    }
}
