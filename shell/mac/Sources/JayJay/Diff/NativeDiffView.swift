import AppKit
import JayJayCore
import SwiftUI

struct NativeDiffView: NSViewRepresentable {
    let diff: FileDiff

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

        let gutterTextView = NSTextView(frame: gutterScrollView.bounds, textContainer: gutterContainer)
        gutterTextView.isEditable = false
        gutterTextView.isSelectable = false
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
                gutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
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
            gutter.append(gutterLine)

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
            gutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
        }

        gutterLayoutManager.lineBgColors = lineBgColors
        layoutManager.lineBgColors = lineBgColors
        gutterTextView.textStorage?.setAttributedString(gutter)
        textView.textStorage?.setAttributedString(result)
        containerView.updateGutterWidth(max(52, gutterWidth))
    }

    private func spanBackground(span: DiffSpan, theme: DiffColors) -> NSColor {
        switch span.style {
            case .added: theme.addedWordBg
            case .removed: theme.removedWordBg
            default: .clear
        }
    }
}
