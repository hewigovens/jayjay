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
        let theme = DiffColors(isDark: isDark)

        let result = NSMutableAttributedString()
        var lineOffsets: [LineOffsetInfo] = []

        for line in diff.lines {
            buildAttributedLine(line, isDark: isDark, font: font, theme: theme,
                                result: result, lineOffsets: &lineOffsets)
        }

        if diff.lines.isEmpty {
            result.append(NSAttributedString(string: "No differences",
                                             attributes: [.font: font, .foregroundColor: NSColor.secondaryLabelColor]))
        }

        textView.textStorage?.setAttributedString(result)
        textView.lineOffsets = lineOffsets
    }

    private func buildAttributedLine(_ line: DiffLine, isDark: Bool, font: NSFont, theme: DiffColors,
                                     result: NSMutableAttributedString, lineOffsets: inout [LineOffsetInfo]) {
        let lineStart = result.length

        if line.style == .separator {
            let sepAttrs: [NSAttributedString.Key: Any] = [
                .font: font, .foregroundColor: theme.gutterText,
                .backgroundColor: isDark ? NSColor(white: 0.16, alpha: 1) : NSColor(white: 0.94, alpha: 1)
            ]
            result.append(NSAttributedString(string: "  ⋯ \(line.spans.first?.text ?? "")\n", attributes: sepAttrs))
            lineOffsets.append(LineOffsetInfo(charStart: lineStart, gutterEnd: lineStart, isSeparator: true))
            return
        }

        // Gutter
        let lineNo = (line.newLineNo ?? line.oldLineNo).map { String($0) } ?? ""
        let padded = lineNo.padding(toLength: 4, withPad: " ", startingAt: 0)
        let marker: String
        let lineBg: NSColor
        switch line.style {
        case .added:   marker = "+"; lineBg = theme.addedBg
        case .removed: marker = "-"; lineBg = theme.removedBg
        default:       marker = " "; lineBg = .clear
        }

        let markerColor: NSColor = marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText
        result.append(NSAttributedString(string: "\(padded) \(marker) ", attributes: [
            .font: font, .foregroundColor: theme.gutterText
        ]))
        let markerRange = NSRange(location: result.length - 2, length: 1)
        result.addAttribute(.foregroundColor, value: markerColor, range: markerRange)

        let gutterEnd = result.length

        // Content -- only changed words get background, unchanged parts are clean
        let lineStr = NSMutableAttributedString()
        for span in line.spans {
            let foreground = foregroundColor(span: span, lineStyle: line.style, theme: theme)
            let background = spanBackground(span: span, theme: theme)
            var attrs: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: foreground]
            if background != .clear { attrs[.backgroundColor] = background }
            lineStr.append(NSAttributedString(string: span.text, attributes: attrs))
        }
        if line.spans.isEmpty {
            lineStr.append(NSAttributedString(string: " ", attributes: [.font: font]))
        }
        result.append(lineStr)
        result.append(NSAttributedString(string: "\n", attributes: [.font: font]))
        lineOffsets.append(LineOffsetInfo(charStart: lineStart, gutterEnd: gutterEnd, isSeparator: false))
    }

    private func foregroundColor(span: DiffSpan, lineStyle: DiffSpanStyle, theme: DiffColors) -> NSColor {
        theme.tokenColor(span.token, fallback: theme.lineText(lineStyle))
    }

    private func spanBackground(span: DiffSpan, theme: DiffColors) -> NSColor {
        switch span.style {
        case .added: theme.addedWordBg
        case .removed: theme.removedWordBg
        default: .clear
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
