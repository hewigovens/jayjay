import AppKit

final class DiffLayoutManager: NSLayoutManager {
    var lineBgColors: [NSColor] = []

    /// Width to fill for per-line background colors. No-wrap containers have
    /// `containerSize.width == .greatestFiniteMagnitude`, so we use the laid-out
    /// width (or the text view bounds, whichever is larger).
    var lineBackgroundFillWidth: CGFloat {
        guard let textContainer = textContainers.first else { return 0 }
        ensureLayout(for: textContainer)
        return max(usedRect(for: textContainer).width, textContainer.textView?.bounds.width ?? 0)
    }

    override func drawBackground(forGlyphRange glyphsToShow: NSRange, at origin: NSPoint) {
        guard let textStorage, textContainers.first != nil else {
            super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
            return
        }

        let drawWidth = lineBackgroundFillWidth

        let fullText = textStorage.string as NSString
        var lineIndex = 0
        var charPos = 0

        while charPos < fullText.length {
            let lineRange = fullText.lineRange(for: NSRange(location: charPos, length: 0))
            let glyphRange = glyphRange(forCharacterRange: lineRange, actualCharacterRange: nil)
            if NSIntersectionRange(glyphRange, glyphsToShow).length > 0,
               lineIndex < lineBgColors.count
            {
                let color = lineBgColors[lineIndex]
                if color != .clear {
                    var lineRect = lineFragmentRect(forGlyphAt: glyphRange.location, effectiveRange: nil)
                    lineRect.origin.x = 0
                    lineRect.size.width = drawWidth
                    lineRect.origin.x += origin.x
                    lineRect.origin.y += origin.y
                    color.setFill()
                    lineRect.fill()
                }
            }

            lineIndex += 1
            charPos = NSMaxRange(lineRange)
        }

        super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
    }
}
