import AppKit
import JayJayCore

public struct CodeHighlighting {
    public let font: NSFont

    private let colors: DiffColors

    public init(font: NSFont, isDark: Bool) {
        self.font = font
        colors = DiffColors(isDark: isDark)
    }

    public var baseAttributes: [NSAttributedString.Key: Any] {
        let paragraph = NSMutableParagraphStyle()
        paragraph.defaultTabInterval = ceil(font.maximumAdvancement.width) * 4
        paragraph.tabStops = []
        return [
            .font: font,
            .foregroundColor: colors.contextText,
            .paragraphStyle: paragraph
        ]
    }

    public func apply(
        to storage: NSMutableAttributedString,
        text: String,
        highlightedLines: [[DiffSpan]],
        lineStyles: [DiffSpanStyle] = []
    ) {
        guard storage.string == text else { return }
        let fullRange = NSRange(location: 0, length: (text as NSString).length)
        if fullRange.length > 0 {
            storage.setAttributes(baseAttributes, range: fullRange)
        }

        let string = text as NSString
        for (lineIndex, (lineRange, spans)) in zip(contentLineRanges(in: string), highlightedLines).enumerated() {
            let highlightedLength = spans.reduce(0) { $0 + ($1.text as NSString).length }
            guard highlightedLength == lineRange.length else { continue }
            let lineStyle = lineStyles.indices.contains(lineIndex) ? lineStyles[lineIndex] : .context
            applyBackground(colors.inlineBackground(lineStyle: lineStyle, spanStyle: .unchanged), to: storage, range: lineRange)
            var location = lineRange.location
            for span in spans {
                let length = (span.text as NSString).length
                guard length > 0 else { continue }
                let range = NSRange(location: location, length: length)
                storage.addAttribute(
                    .foregroundColor,
                    value: colors.tokenColor(span.token, fallback: colors.contextText),
                    range: range
                )
                applyBackground(colors.inlineBackground(lineStyle: lineStyle, spanStyle: span.style), to: storage, range: range)
                location += length
            }
        }
    }

    private func applyBackground(_ color: NSColor, to storage: NSMutableAttributedString, range: NSRange) {
        if color != .clear, range.length > 0 {
            storage.addAttribute(.backgroundColor, value: color, range: range)
        }
    }

    private func contentLineRanges(in text: NSString) -> [NSRange] {
        guard text.length > 0 else { return [] }
        var ranges: [NSRange] = []
        var location = 0
        while location < text.length {
            var start = 0
            var end = 0
            var contentsEnd = 0
            text.getLineStart(&start, end: &end, contentsEnd: &contentsEnd, for: NSRange(location: location, length: 0))
            ranges.append(NSRange(location: start, length: contentsEnd - start))
            location = end
        }
        return ranges
    }
}
