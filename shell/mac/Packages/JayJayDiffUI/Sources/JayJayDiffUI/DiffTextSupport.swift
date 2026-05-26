import AppKit

final class DiffLayoutManager: NSLayoutManager {
    var lineBgColors: [NSColor] = []
    var lineStripeColors: [NSColor] = []
    var lineStripeX: CGFloat = 0
    var lineStripeWidth: CGFloat = 0
    var selectedRangeBgColor: NSColor = .selectedTextBackgroundColor
    var findMatchBgColor: NSColor = .findHighlightColor

    /// Width to fill for per-line background colors. No-wrap containers have
    /// `containerSize.width == .greatestFiniteMagnitude`, so we use the laid-out
    /// width (or the text view bounds, whichever is larger).
    var lineBackgroundFillWidth: CGFloat {
        guard let textContainer = textContainers.first else { return 0 }
        ensureLayout(for: textContainer)
        return max(usedRect(for: textContainer).width, textContainer.textView?.bounds.width ?? 0)
    }

    func visualLineCounts(logicalLineCount: Int) -> [Int] {
        guard logicalLineCount > 0,
              let textStorage,
              let textContainer = textContainers.first
        else { return [] }

        ensureLayout(for: textContainer)

        let text = textStorage.string as NSString
        var counts: [Int] = []
        var charPos = 0
        while counts.count < logicalLineCount {
            guard charPos < text.length else {
                counts.append(1)
                continue
            }

            let lineRange = text.lineRange(for: NSRange(location: charPos, length: 0))
            let glyphRange = glyphRange(forCharacterRange: lineRange, actualCharacterRange: nil)
            var fragmentCount = 0
            enumerateLineFragments(forGlyphRange: glyphRange) { _, _, _, lineGlyphRange, _ in
                if NSIntersectionRange(lineGlyphRange, glyphRange).length > 0 {
                    fragmentCount += 1
                }
            }
            counts.append(max(1, fragmentCount))
            charPos = NSMaxRange(lineRange)
        }
        return counts
    }

    /// Owned buffer for clamped selection rects. Held until the next
    /// rectArray() call. NSTextView reads the returned rects synchronously
    /// before yielding, so reuse across calls is safe.
    private var rectsBuffer: UnsafeMutableBufferPointer<NSRect>?

    deinit {
        rectsBuffer?.deallocate()
    }

    override func drawBackground(forGlyphRange glyphsToShow: NSRange, at origin: NSPoint) {
        guard let textStorage, let textContainer = textContainers.first else {
            super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
            return
        }

        let drawWidth = lineBackgroundFillWidth
        let selectedCharRanges: [NSRange] = (textContainer.textView?.selectedRanges as? [NSValue])?
            .map(\.rangeValue)
            .filter { $0.length > 0 } ?? []

        let fullText = textStorage.string as NSString
        var lineIndex = 0
        var charPos = 0

        while charPos < fullText.length {
            let lineRange = fullText.lineRange(for: NSRange(location: charPos, length: 0))
            let glyphRange = glyphRange(forCharacterRange: lineRange, actualCharacterRange: nil)
            if NSIntersectionRange(glyphRange, glyphsToShow).length > 0 {
                enumerateLineFragments(forGlyphRange: glyphRange) { lineRect, _, _, lineGlyphRange, _ in
                    guard NSIntersectionRange(lineGlyphRange, glyphsToShow).length > 0 else { return }

                    if lineIndex < self.lineBgColors.count {
                        let color = self.lineBgColors[lineIndex]
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

                    // Selection takes over the stripe column while active.
                    let isSelected = selectedCharRanges.contains { selRange in
                        NSIntersectionRange(lineRange, selRange).length > 0
                    }
                    if !isSelected,
                       self.lineStripeWidth > 0,
                       lineIndex < self.lineStripeColors.count
                    {
                        let color = self.lineStripeColors[lineIndex]
                        if color != .clear {
                            // Overlap ±1pt so adjacent stripes have no sub-pixel seams.
                            var stripeRect = lineRect
                            stripeRect.origin.x = self.lineStripeX + origin.x
                            stripeRect.origin.y += origin.y - 1
                            stripeRect.size.width = self.lineStripeWidth
                            stripeRect.size.height += 2
                            color.setFill()
                            stripeRect.fill()
                        }
                    }
                }
            }

            lineIndex += 1
            charPos = NSMaxRange(lineRange)
        }

        super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
        if let textView = textContainer.textView as? DiffTextView,
           textView.showsFindHighlights,
           let findString = textView.activeFindQuery
        {
            drawFindMatchHighlights(
                findMatchRanges(findString, visibleGlyphRange: glyphsToShow),
                visibleGlyphRange: glyphsToShow,
                in: textContainer,
                at: origin
            )
        }
        drawSelectedRangeHighlights(
            selectedCharRanges,
            visibleGlyphRange: glyphsToShow,
            in: textContainer,
            at: origin
        )
    }

    /// NSLayoutManager coalesces full-line selections into a single
    /// multi-line rect, so a per-rect clamp on super's output can't work
    /// (there's only one rect to clamp). Recompute the rects ourselves:
    /// walk line fragments in the selection and emit one rect per line,
    /// each clamped to that line's used-text width.
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
        let perLine = selectionRects(forCharacterRange: intersected, in: container)
        if perLine.isEmpty {
            rectCount.pointee = 0
            return nil
        }

        let buf = ensureRectBuffer(capacity: perLine.count)
        for (i, r) in perLine.enumerated() {
            buf[i] = r
        }
        rectCount.pointee = perLine.count
        return buf.baseAddress
    }

    private func drawSelectedRangeHighlights(
        _ ranges: [NSRange],
        visibleGlyphRange: NSRange,
        in container: NSTextContainer,
        at origin: NSPoint
    ) {
        guard !ranges.isEmpty else { return }
        let backgroundColor = (container.textView as? DiffTextView)?.selectionHighlightBackgroundColor
            ?? selectedRangeBgColor
        backgroundColor.setFill()
        for range in ranges {
            for rect in selectionRects(forCharacterRange: range, in: container, visibleGlyphRange: visibleGlyphRange) {
                let drawRect = rect
                    .offsetBy(dx: origin.x, dy: origin.y)
                    .insetBy(dx: -1, dy: 1)
                NSBezierPath(roundedRect: drawRect, xRadius: 2, yRadius: 2).fill()
            }
        }
    }

    private func drawFindMatchHighlights(
        _ ranges: [NSRange],
        visibleGlyphRange: NSRange,
        in container: NSTextContainer,
        at origin: NSPoint
    ) {
        guard !ranges.isEmpty else { return }
        findMatchBgColor.setFill()
        for range in ranges {
            for rect in selectionRects(forCharacterRange: range, in: container, visibleGlyphRange: visibleGlyphRange) {
                let drawRect = rect
                    .offsetBy(dx: origin.x, dy: origin.y)
                    .insetBy(dx: -1, dy: 1)
                NSBezierPath(roundedRect: drawRect, xRadius: 2, yRadius: 2).fill()
            }
        }
    }

    func findMatchRanges(_ findString: String, visibleGlyphRange: NSRange? = nil) -> [NSRange] {
        guard let textStorage else { return [] }
        let text = textStorage.string as NSString
        let findLength = (findString as NSString).length
        guard findLength > 0, text.length >= findLength else { return [] }

        let visibleCharRange = visibleGlyphRange
            .map { characterRange(forGlyphRange: $0, actualGlyphRange: nil) }
            ?? NSRange(location: 0, length: text.length)
        guard visibleCharRange.location != NSNotFound, visibleCharRange.length > 0 else { return [] }

        let searchStart = max(0, visibleCharRange.location - findLength + 1)
        let searchEnd = min(text.length, NSMaxRange(visibleCharRange) + findLength - 1)
        guard searchEnd > searchStart else { return [] }

        var ranges: [NSRange] = []
        var location = searchStart
        while location < searchEnd {
            let remaining = NSRange(location: location, length: searchEnd - location)
            let found = text.range(of: findString, options: [.caseInsensitive], range: remaining)
            guard found.location != NSNotFound else { break }
            if NSIntersectionRange(found, visibleCharRange).length > 0 {
                ranges.append(found)
            }
            location = max(NSMaxRange(found), found.location + 1)
        }
        return ranges
    }

    private func selectionRects(
        forCharacterRange charRange: NSRange,
        in container: NSTextContainer,
        visibleGlyphRange: NSRange? = nil
    ) -> [NSRect] {
        let glyphsInRange = glyphRange(forCharacterRange: charRange, actualCharacterRange: nil)
        guard glyphsInRange.length > 0 else { return [] }
        let glyphsToDraw = visibleGlyphRange.map { NSIntersectionRange(glyphsInRange, $0) } ?? glyphsInRange
        guard glyphsToDraw.length > 0 else { return [] }

        var perLine: [NSRect] = []
        enumerateLineFragments(forGlyphRange: glyphsToDraw) {
            lineRect, usedRect, _, lineGlyphRange, _ in
            let onLine = NSIntersectionRange(lineGlyphRange, glyphsToDraw)
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
        return perLine
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
