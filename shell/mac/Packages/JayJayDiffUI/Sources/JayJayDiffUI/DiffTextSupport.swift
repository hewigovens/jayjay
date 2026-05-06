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

    /// Owned buffer for clamped selection rects. Held until the next
    /// rectArray() call. NSTextView reads the returned rects synchronously
    /// before yielding, so reuse across calls is safe.
    private var rectsBuffer: UnsafeMutableBufferPointer<NSRect>?

    deinit {
        rectsBuffer?.deallocate()
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

    // NSLayoutManager coalesces full-line selections into a single
    // multi-line rect, so a per-rect clamp on super's output can't work
    // (there's only one rect to clamp). Recompute the rects ourselves:
    // walk line fragments in the selection and emit one rect per line,
    // each clamped to that line's used-text width.
    override func rectArray(
        forCharacterRange charRange: NSRange,
        withinSelectedCharacterRange selCharRange: NSRange,
        in container: NSTextContainer,
        rectCount: UnsafeMutablePointer<Int>
    ) -> UnsafeMutablePointer<NSRect>? {
        let intersected = NSIntersectionRange(charRange, selCharRange)
        if intersected.length == 0 {
            rectCount.pointee = 0
            return nil
        }
        let glyphsInRange = glyphRange(forCharacterRange: intersected, actualCharacterRange: nil)
        if glyphsInRange.length == 0 {
            rectCount.pointee = 0
            return nil
        }

        var perLine: [NSRect] = []
        enumerateLineFragments(forGlyphRange: glyphsInRange) {
            lineRect, usedRect, _, lineGlyphRange, _ in
            let onLine = NSIntersectionRange(lineGlyphRange, glyphsInRange)
            if onLine.length == 0 { return }

            let firstBox = self.boundingRect(
                forGlyphRange: NSRange(location: onLine.location, length: 1),
                in: container
            )
            let lastBox = self.boundingRect(
                forGlyphRange: NSRange(location: NSMaxRange(onLine) - 1, length: 1),
                in: container
            )
            let xStart = firstBox.origin.x
            let lineEol = NSMaxX(usedRect)
            let xEnd = min(NSMaxX(lastBox), lineEol)
            if xEnd <= xStart { return }
            perLine.append(NSRect(
                x: xStart,
                y: lineRect.minY,
                width: xEnd - xStart,
                height: lineRect.height
            ))
        }

        let buf = ensureRectBuffer(capacity: perLine.count)
        for (i, r) in perLine.enumerated() {
            buf[i] = r
        }
        rectCount.pointee = perLine.count
        return buf.baseAddress
    }

    private func ensureRectBuffer(capacity: Int) -> UnsafeMutableBufferPointer<NSRect> {
        if let buf = rectsBuffer, buf.count >= capacity {
            return buf
        }
        rectsBuffer?.deallocate()
        let needed = max(capacity, 8)
        let buf = UnsafeMutableBufferPointer<NSRect>.allocate(capacity: needed)
        rectsBuffer = buf
        return buf
    }
}
