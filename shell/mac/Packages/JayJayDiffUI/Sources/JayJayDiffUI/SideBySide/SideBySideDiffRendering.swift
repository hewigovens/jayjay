import AppKit
import JayJayCore

extension SideBySideRepresentable {
    func makeContainer() -> DiffTextContainerView {
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

        let textView = DiffTextView(frame: scrollView.bounds, textContainer: textContainer)
        textView.isEditable = false
        textView.isSelectable = true
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.autoresizingMask = [.width]
        textView.textContainerInset = NSSize(width: 4, height: 6)
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
}

func appendTextLine(
    to str: NSMutableAttributedString,
    spans: [DiffSpan],
    style: DiffSpanStyle,
    conflictKind: ConflictLineKind,
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

    if let label = conflictLabel(spans: spans, kind: conflictKind) {
        str.append(NSAttributedString(
            string: "\(conflictDisplayLine(label: label, kind: conflictKind))\n",
            attributes: conflictLineAttributes(kind: conflictKind, font: font, theme: theme)
        ))
    } else if spans.isEmpty {
        str.append(NSAttributedString(string: "\n", attributes: [.font: font]))
    } else {
        for span in spans {
            let attrs = diffSpanAttributes(
                for: span,
                lineStyle: style,
                conflictKind: conflictKind,
                font: font,
                theme: theme
            )
            str.append(NSAttributedString(string: span.text, attributes: attrs))
        }
        str.append(NSAttributedString(string: "\n", attributes: [.font: font]))
    }

    bgColors.append(theme.lineBg(style, conflictKind: conflictKind))
}

func appendGutterLine(
    to str: NSMutableAttributedString,
    entries: inout [DiffGutterTextView.Entry],
    lineNo: String,
    style: DiffSpanStyle,
    attrs: [NSAttributedString.Key: Any],
    inset: CGFloat,
    trailingPadding: CGFloat,
    width: inout CGFloat
) {
    if style == .separator {
        let start = str.length
        str.append(NSAttributedString(string: "\n", attributes: attrs))
        entries.append(.init(
            style: style,
            range: NSRange(location: start, length: str.length - start),
            lineNumber: entries.count + 1
        ))
        return
    }

    let padded = lineNo.isEmpty ? "" : lineNo
    let line = NSMutableAttributedString(string: padded, attributes: attrs)
    line.append(NSAttributedString(string: "\n", attributes: attrs))
    let start = str.length
    str.append(line)
    entries.append(.init(
        style: style,
        range: NSRange(location: start, length: str.length - start),
        lineNumber: entries.count + 1
    ))

    let numberWidth = (padded as NSString).size(withAttributes: attrs).width
    width = max(width, ceil(inset + numberWidth + trailingPadding + inset))
}
