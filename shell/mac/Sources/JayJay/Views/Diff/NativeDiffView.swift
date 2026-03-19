import AppKit
import SwiftUI
import JayJayBindings

// MARK: - Known Issues
// Copy stripping uses O(n) newline counting per line during copy.
// For very large diffs (10k+ lines), Cmd+C may have a brief delay.
// Future: pre-compute line→offset mapping for O(1) lookup.

struct NativeDiffView: NSViewRepresentable {
    let diff: FileDiff

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontScale) private var fontScale

    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false

        let textView = CopyStrippingTextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.autoresizingMask = [.width]
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.textContainerInset = NSSize(width: 4, height: 8)
        textView.drawsBackground = false
        textView.textContainer?.widthTracksTextView = true
        textView.textContainer?.lineFragmentPadding = 4

        scrollView.documentView = textView
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? CopyStrippingTextView else { return }

        let fontSize = max(10.0, 12.0 * fontScale)
        let font = NSFont.monospacedSystemFont(ofSize: fontSize, weight: .regular)
        let isDark = colorScheme == .dark
        let theme = DiffTheme(isDark: isDark)

        let result = NSMutableAttributedString()
        var lineOffsets: [LineOffsetInfo] = []

        for line in diff.lines {
            let lineStart = result.length

            if line.style == .separator {
                let sepAttrs: [NSAttributedString.Key: Any] = [
                    .font: font, .foregroundColor: theme.gutterText,
                    .backgroundColor: isDark ? NSColor(white: 0.16, alpha: 1) : NSColor(white: 0.94, alpha: 1),
                ]
                result.append(NSAttributedString(string: "  ⋯ \(line.spans.first?.text ?? "")\n", attributes: sepAttrs))
                lineOffsets.append(LineOffsetInfo(charStart: lineStart, gutterEnd: lineStart, isSeparator: true))
                continue
            }

            // Gutter
            let lineNo = (line.newLineNo ?? line.oldLineNo).map { String($0) } ?? ""
            let padded = lineNo.padding(toLength: 4, withPad: " ", startingAt: 0)
            let marker: String
            let lineBg: NSColor
            switch line.style {
            case .added:   marker = "+"; lineBg = theme.addedBackground
            case .removed: marker = "-"; lineBg = theme.removedBackground
            default:       marker = " "; lineBg = .clear
            }

            let markerColor: NSColor = marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText
            result.append(NSAttributedString(string: "\(padded) \(marker) ", attributes: [
                .font: font, .foregroundColor: theme.gutterText,
            ]))
            let markerRange = NSRange(location: result.length - 2, length: 1)
            result.addAttribute(.foregroundColor, value: markerColor, range: markerRange)

            let gutterEnd = result.length

            // Content
            let lineStr = NSMutableAttributedString()
            for span in line.spans {
                let fg = foregroundColor(span: span, lineStyle: line.style, theme: theme)
                let bg = spanBackground(span: span, theme: theme)
                var attrs: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: fg]
                if bg != .clear { attrs[.backgroundColor] = bg }
                lineStr.append(NSAttributedString(string: span.text, attributes: attrs))
            }
            if line.spans.isEmpty {
                lineStr.append(NSAttributedString(string: " ", attributes: [.font: font]))
            }
            result.append(lineStr)

            if lineBg != .clear {
                result.addAttribute(.backgroundColor, value: lineBg,
                                    range: NSRange(location: lineStart, length: result.length - lineStart))
            }
            result.append(NSAttributedString(string: "\n", attributes: [.font: font]))
            lineOffsets.append(LineOffsetInfo(charStart: lineStart, gutterEnd: gutterEnd, isSeparator: false))
        }

        if diff.lines.isEmpty {
            result.append(NSAttributedString(string: "No differences",
                                             attributes: [.font: font, .foregroundColor: NSColor.secondaryLabelColor]))
        }

        textView.textStorage?.setAttributedString(result)
        textView.lineOffsets = lineOffsets
    }

    private func foregroundColor(span: NativeDiffSpan, lineStyle: DiffSpanStyle, theme: DiffTheme) -> NSColor {
        switch span.token {
        case .comment: return theme.comment
        case .keyword: return theme.keyword
        case .stringLit: return theme.string
        case .number: return theme.number
        case .type: return theme.type
        case .function: return theme.function
        case .attribute: return theme.attribute
        case .operator: return theme.operator
        case .punctuation: return theme.punctuation
        default: break
        }
        switch lineStyle {
        case .added: return theme.addedText
        case .removed: return theme.removedText
        default: return theme.contextText
        }
    }

    private func spanBackground(span: NativeDiffSpan, theme: DiffTheme) -> NSColor {
        switch span.style {
        case .added: return theme.addedWordBackground
        case .removed: return theme.removedWordBackground
        default: return .clear
        }
    }
}

// MARK: - Copy stripping

private struct LineOffsetInfo {
    let charStart: Int   // Character offset where this line begins in the text storage
    let gutterEnd: Int   // Character offset where the gutter ends (content starts)
    let isSeparator: Bool
}

private class CopyStrippingTextView: NSTextView {
    var lineOffsets: [LineOffsetInfo] = []

    override func copy(_ sender: Any?) {
        let sel = selectedRange()
        guard sel.length > 0 else { return }

        let fullText = (textStorage?.string ?? "") as NSString
        var parts: [String] = []
        let end = NSMaxRange(sel)

        for info in lineOffsets {
            let lineEnd = fullText.lineRange(for: NSRange(location: info.charStart, length: 0))
            guard NSMaxRange(lineEnd) > sel.location && info.charStart < end else {
                if info.charStart >= end { break }
                continue
            }
            if info.isSeparator { continue }

            let contentStart = max(info.gutterEnd, sel.location)
            let contentEnd = min(NSMaxRange(lineEnd), end)
            if contentStart < contentEnd {
                parts.append(fullText.substring(with: NSRange(location: contentStart, length: contentEnd - contentStart)))
            }
        }

        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(parts.joined(), forType: .string)
    }
}

// MARK: - Theme

struct DiffTheme {
    let isDark: Bool
    var gutterText: NSColor { isDark ? NSColor(white: 0.45, alpha: 1) : NSColor(white: 0.65, alpha: 1) }
    var contextText: NSColor { isDark ? NSColor(white: 0.85, alpha: 1) : NSColor(white: 0.15, alpha: 1) }
    var addedText: NSColor { isDark ? NSColor(red: 0.47, green: 0.91, blue: 0.53, alpha: 1) : NSColor(red: 0.08, green: 0.47, blue: 0.17, alpha: 1) }
    var addedBackground: NSColor { isDark ? NSColor(red: 0.07, green: 0.15, blue: 0.12, alpha: 1) : NSColor(red: 0.85, green: 0.98, blue: 0.88, alpha: 1) }
    var addedWordBackground: NSColor { isDark ? NSColor(red: 0.15, green: 0.42, blue: 0.22, alpha: 0.5) : NSColor(red: 0.67, green: 0.93, blue: 0.73, alpha: 0.6) }
    var removedText: NSColor { isDark ? NSColor(red: 1, green: 0.48, blue: 0.45, alpha: 1) : NSColor(red: 0.82, green: 0.17, blue: 0.14, alpha: 1) }
    var removedBackground: NSColor { isDark ? NSColor(red: 0.18, green: 0.08, blue: 0.08, alpha: 1) : NSColor(red: 1, green: 0.93, blue: 0.94, alpha: 1) }
    var removedWordBackground: NSColor { isDark ? NSColor(red: 0.52, green: 0.15, blue: 0.15, alpha: 0.5) : NSColor(red: 1, green: 0.78, blue: 0.78, alpha: 0.6) }
    var keyword: NSColor { isDark ? NSColor(red: 1, green: 0.48, blue: 0.45, alpha: 1) : NSColor(red: 0.84, green: 0.23, blue: 0.29, alpha: 1) }
    var string: NSColor { isDark ? NSColor(red: 0.65, green: 0.84, blue: 1, alpha: 1) : NSColor(red: 0.01, green: 0.18, blue: 0.38, alpha: 1) }
    var comment: NSColor { isDark ? NSColor(red: 0.55, green: 0.58, blue: 0.63, alpha: 1) : NSColor(red: 0.42, green: 0.45, blue: 0.49, alpha: 1) }
    var number: NSColor { isDark ? NSColor(red: 0.47, green: 0.75, blue: 1, alpha: 1) : NSColor(red: 0, green: 0.36, blue: 0.77, alpha: 1) }
    var type: NSColor { isDark ? NSColor(red: 0.82, green: 0.66, blue: 1, alpha: 1) : NSColor(red: 0.44, green: 0.26, blue: 0.76, alpha: 1) }
    var function: NSColor { isDark ? NSColor(red: 0.82, green: 0.66, blue: 1, alpha: 1) : NSColor(red: 0.44, green: 0.26, blue: 0.76, alpha: 1) }
    var attribute: NSColor { isDark ? NSColor(red: 0.82, green: 0.66, blue: 1, alpha: 1) : NSColor(red: 0.44, green: 0.26, blue: 0.76, alpha: 1) }
    var `operator`: NSColor { keyword }
    var punctuation: NSColor { contextText }
}
