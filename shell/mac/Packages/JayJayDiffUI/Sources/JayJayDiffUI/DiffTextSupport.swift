import AppKit

final class DiffLayoutManager: NSLayoutManager {
    var lineBgColors: [NSColor] = []
    var lineStripeColors: [NSColor] = []
    var lineStripeX: CGFloat = 0
    var lineStripeWidth: CGFloat = 0

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
            if NSIntersectionRange(glyphRange, glyphsToShow).length > 0 {
                let lineRect = lineFragmentRect(forGlyphAt: glyphRange.location, effectiveRange: nil)
                if lineIndex < lineBgColors.count {
                    let color = lineBgColors[lineIndex]
                    if color != .clear {
                        var bgRect = lineRect
                        bgRect.origin.x = 0
                        bgRect.size.width = drawWidth
                        bgRect.origin.x += origin.x
                        bgRect.origin.y += origin.y
                        color.setFill()
                        bgRect.fill()
                    }
                }

                if lineStripeWidth > 0,
                   lineIndex < lineStripeColors.count
                {
                    let color = lineStripeColors[lineIndex]
                    if color != .clear {
                        var stripeRect = lineRect
                        stripeRect.origin.x = lineStripeX + origin.x
                        stripeRect.origin.y += origin.y - 0.5
                        stripeRect.size.width = lineStripeWidth
                        stripeRect.size.height += 1
                        color.setFill()
                        stripeRect.fill()
                    }
                }
            }

            lineIndex += 1
            charPos = NSMaxRange(lineRange)
        }

        super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
    }

    // Clamp NSTextView's selection rects to per-line used-text width.
    override func rectArray(
        forCharacterRange charRange: NSRange,
        withinSelectedCharacterRange selCharRange: NSRange,
        in container: NSTextContainer,
        rectCount: UnsafeMutablePointer<Int>
    ) -> UnsafeMutablePointer<NSRect>? {
        guard
            let rects = super.rectArray(
                forCharacterRange: charRange,
                withinSelectedCharacterRange: selCharRange,
                in: container,
                rectCount: rectCount
            )
        else { return nil }

        let count = rectCount.pointee
        for i in 0..<count {
            let rect = rects[i]
            // Probe at rect's center so we always land on the rect's own line.
            // origin.x can sit at line-leading edge before the first glyph,
            // where glyphIndex returns the previous (shorter) line's last
            // glyph — that mis-clamps the highlight on long lines.
            guard rect.width > 0, rect.height > 0 else { continue }
            let center = NSPoint(x: rect.midX, y: rect.midY)
            let glyphIx = glyphIndex(for: center, in: container)
            let usedRect = lineFragmentUsedRect(forGlyphAt: glyphIx, effectiveRange: nil)
            let lineEol = NSMaxX(usedRect)
            if rect.origin.x >= lineEol {
                rects[i].size.width = 0
            } else if NSMaxX(rect) > lineEol {
                rects[i].size.width = lineEol - rect.origin.x
            }
        }
        return rects
    }
}
