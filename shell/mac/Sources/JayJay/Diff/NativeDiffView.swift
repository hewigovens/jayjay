import AppKit
import SwiftUI
import JayJayBindings

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

        let textContainer = NSTextContainer(containerSize: NSSize(width: 0, height: CGFloat.greatestFiniteMagnitude))
        textContainer.widthTracksTextView = true
        textContainer.lineFragmentPadding = 4

        let layoutManager = DiffLayoutManager()
        layoutManager.addTextContainer(textContainer)

        let storage = NSTextStorage()
        storage.addLayoutManager(layoutManager)

        let textView = CopyStrippingTextView(frame: scrollView.bounds, textContainer: textContainer)
        textView.isEditable = false
        textView.isSelectable = true
        textView.autoresizingMask = [.width]
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.textContainerInset = NSSize(width: 4, height: 8)
        textView.drawsBackground = false
        textView.minSize = NSSize(width: 0, height: 0)
        textView.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)

        scrollView.documentView = textView
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? CopyStrippingTextView,
              let layoutManager = textView.layoutManager as? DiffLayoutManager else { return }

        let fontSize = max(10.0, 12.0 * fontScale)
        let font = NSFont.monospacedSystemFont(ofSize: fontSize, weight: .regular)
        let isDark = colorScheme == .dark
        let theme = DiffColors(isDark: isDark)

        let result = NSMutableAttributedString()
        var lineOffsets: [LineOffsetInfo] = []
        var lineBgColors: [NSColor] = []

        for line in diff.lines {
            let lineStart = result.length

            if line.style == .separator {
                result.append(NSAttributedString(string: "  ⋯ \(line.spans.first?.text ?? "")\n", attributes: [
                    .font: font, .foregroundColor: theme.gutterText
                ]))
                lineOffsets.append(LineOffsetInfo(charStart: lineStart, gutterEnd: lineStart, isSeparator: true))
                lineBgColors.append(theme.separatorBg)
                continue
            }

            // Gutter
            let lineNo = (line.newLineNo ?? line.oldLineNo).map { String($0) } ?? ""
            let padded = lineNo.padding(toLength: 4, withPad: " ", startingAt: 0)
            let marker: String
            switch line.style {
            case .added:   marker = "+"
            case .removed: marker = "-"
            default:       marker = " "
            }

            let markerColor = marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText
            result.append(NSAttributedString(string: "\(padded) \(marker) ", attributes: [
                .font: font, .foregroundColor: theme.gutterText
            ]))
            let markerRange = NSRange(location: result.length - 2, length: 1)
            result.addAttribute(.foregroundColor, value: markerColor, range: markerRange)

            let gutterEnd = result.length

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

            lineOffsets.append(LineOffsetInfo(charStart: lineStart, gutterEnd: gutterEnd, isSeparator: false))
            lineBgColors.append(theme.lineBg(line.style))
        }

        if diff.lines.isEmpty {
            result.append(NSAttributedString(string: "No differences",
                                             attributes: [.font: font, .foregroundColor: NSColor.secondaryLabelColor]))
        }

        layoutManager.lineBgColors = lineBgColors
        textView.textStorage?.setAttributedString(result)
        textView.lineOffsets = lineOffsets
    }

    private func spanBackground(span: DiffSpan, theme: DiffColors) -> NSColor {
        switch span.style {
        case .added: theme.addedWordBg
        case .removed: theme.removedWordBg
        default: .clear
        }
    }
}

// MARK: - Layout manager that fills full line-fragment rects with background colors

class DiffLayoutManager: NSLayoutManager {
    var lineBgColors: [NSColor] = []

    override func drawBackground(forGlyphRange glyphsToShow: NSRange, at origin: NSPoint) {
        super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
        guard let textStorage, let textContainer = textContainers.first else { return }

        let fullText = textStorage.string as NSString
        var lineIndex = 0
        var charPos = 0

        // Map character positions to line indices
        while charPos < fullText.length {
            let lineRange = fullText.lineRange(for: NSRange(location: charPos, length: 0))
            let glyphRange = self.glyphRange(forCharacterRange: lineRange, actualCharacterRange: nil)

            // Only draw if this line's glyphs intersect the visible range
            if NSIntersectionRange(glyphRange, glyphsToShow).length > 0,
               lineIndex < lineBgColors.count {
                let color = lineBgColors[lineIndex]
                if color != .clear {
                    var lineRect = lineFragmentRect(forGlyphAt: glyphRange.location, effectiveRange: nil)
                    lineRect.origin.x = 0
                    lineRect.size.width = textContainer.containerSize.width
                    lineRect.origin.x += origin.x
                    lineRect.origin.y += origin.y
                    color.setFill()
                    lineRect.fill()
                }
            }

            lineIndex += 1
            charPos = NSMaxRange(lineRange)
        }
    }
}

// MARK: - Copy stripping

private struct LineOffsetInfo {
    let charStart: Int
    let gutterEnd: Int
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
