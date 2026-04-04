import AppKit

final class DiffLayoutManager: NSLayoutManager {
    var lineBgColors: [NSColor] = []

    override func drawBackground(forGlyphRange glyphsToShow: NSRange, at origin: NSPoint) {
        guard let textStorage, let textContainer = textContainers.first else {
            super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
            return
        }

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

        super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
    }
}
