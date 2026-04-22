import AppKit
import JayJayCore
import SwiftUI

public enum DiffGutterCheckboxState {
    case selected
    case unselected
}

public struct DiffGutterContextActions {
    public var openDiffEdit: (() -> Void)?
    public var selectFile: (() -> Void)?
    public var selectHunk: ((ClosedRange<Int>) -> Void)?
    public var onLineSelectionChanged: ((ClosedRange<Int>) -> Void)?
    public var selectedLineRange: ClosedRange<Int>?
    public var lineCheckboxState: ((Int) -> DiffGutterCheckboxState?)?
    public var toggleLineCheckbox: ((Int) -> Void)?
    public var abandonSelectedLines: (() -> Void)?

    public init(
        openDiffEdit: (() -> Void)? = nil,
        selectFile: (() -> Void)? = nil,
        selectHunk: ((ClosedRange<Int>) -> Void)? = nil,
        onLineSelectionChanged: ((ClosedRange<Int>) -> Void)? = nil,
        selectedLineRange: ClosedRange<Int>? = nil,
        lineCheckboxState: ((Int) -> DiffGutterCheckboxState?)? = nil,
        toggleLineCheckbox: ((Int) -> Void)? = nil,
        abandonSelectedLines: (() -> Void)? = nil
    ) {
        self.openDiffEdit = openDiffEdit
        self.selectFile = selectFile
        self.selectHunk = selectHunk
        self.onLineSelectionChanged = onLineSelectionChanged
        self.selectedLineRange = selectedLineRange
        self.lineCheckboxState = lineCheckboxState
        self.toggleLineCheckbox = toggleLineCheckbox
        self.abandonSelectedLines = abandonSelectedLines
    }
}

public struct NativeDiffView: NSViewRepresentable {
    public let diff: FileDiff
    public var gutterActions: DiffGutterContextActions?

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.diffFontSize) private var fontSize
    @Environment(\.diffFontFamily) private var fontFamily

    public init(diff: FileDiff, gutterActions: DiffGutterContextActions? = nil) {
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

        let gutterParagraphStyle = NSMutableParagraphStyle()
        let showsLineCheckboxes = gutterActions?.lineCheckboxState != nil
        gutterParagraphStyle.alignment = showsLineCheckboxes ? .left : .right
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
        let checkboxWidth = ("[x]" as NSString).size(withAttributes: [.font: font]).width
        let gutterHorizontalInset = gutterTextView.textContainerInset.width
        let gutterGap: CGFloat = 10
        let gutterTrailingPadding: CGFloat = 10
        let maxLineDigits = diff.lines.reduce(into: 1) { digits, line in
            let lineNumber = max(line.oldLineNo ?? 0, line.newLineNo ?? 0)
            digits = max(digits, String(lineNumber).count)
        }
        var lineBgColors: [NSColor] = []

        for (index, line) in diff.lines.enumerated() {
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
            let padded = pad(lineNumber, toWidth: maxLineDigits)
            let markerColor = marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText
            let gutterLine = NSMutableAttributedString(
                string: checkboxText(for: index + 1, line: line),
                attributes: [
                    .font: font,
                    .foregroundColor: checkboxColor(for: index + 1, theme: theme),
                    .paragraphStyle: gutterParagraphStyle
                ]
            )
            gutterLine.append(NSAttributedString(
                string: padded,
                attributes: gutterAttrs
            ))
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
                        (showsLineCheckboxes ? checkboxWidth + gutterGap : 0) +
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
        gutterTextView.toggleLineCheckbox = gutterActions?.toggleLineCheckbox
        gutterTextView.checkboxHitWidth = showsLineCheckboxes ? checkboxWidth + gutterGap : 0
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
        if let abandonSelectedLines = gutterActions.abandonSelectedLines {
            if !items.isEmpty {
                items.append(.separator)
            }
            items.append(
                DiffGutterMenuItem(
                    title: "Abandon Selected Lines",
                    enabled: selection.changedLineCount > 0,
                    action: selection.changedLineCount > 0 ? abandonSelectedLines : nil
                )
            )
        }
        if !items.isEmpty,
           items.last?.action == nil,
           items.last?.title.isEmpty == true
        {
            _ = items.popLast()
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

    private func checkboxText(for lineNumber: Int, line: DiffLine) -> String {
        guard line.isChanged else { return "" }
        guard let state = gutterActions?.lineCheckboxState?(lineNumber) else { return "" }
        switch state {
            case .selected:
                return "[x] "
            case .unselected:
                return "[ ] "
        }
    }

    private func checkboxColor(for lineNumber: Int, theme: DiffColors) -> NSColor {
        guard let state = gutterActions?.lineCheckboxState?(lineNumber) else {
            return theme.gutterText
        }
        switch state {
            case .selected:
                return .controlAccentColor
            case .unselected:
                return theme.gutterText
        }
    }

    private func pad(_ value: String, toWidth width: Int) -> String {
        guard value.count < width else { return value }
        return String(repeating: " ", count: width - value.count) + value
    }
}

private extension DiffLine {
    var isChanged: Bool {
        style == .added || style == .removed
    }
}
