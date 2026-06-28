import AppKit

final class DiffLayoutManager: NSLayoutManager {
    var lineBgColors: [NSColor] = []
    var lineStripeColors: [NSColor] = []
    var lineStripeX: CGFloat = 0
    var lineStripeWidth: CGFloat = 0
    var selectedRangeBgColor: NSColor = .selectedTextBackgroundColor
    var findMatchBgColor: NSColor = .findHighlightColor
    /// Keeps the selection highlight off leading marker columns (gutter stripe, note ●); 0 for content views.
    var selectionHighlightLeadingInset: CGFloat = 0
    /// Visual rows (gutter row indices) washed with the hover color — every row of the hovered display line.
    var hoveredRowIndices: Set<Int> = []
    var hoverBgColor: NSColor = .labelColor.withAlphaComponent(0.055)
    /// Character ranges of embedded review-note bubbles; each is drawn as one rounded card behind its rows.
    var noteBubbleRanges: [NSRange] = []
    var noteBubbleFill: NSColor = .clear
    var noteBubbleStroke: NSColor = .clear

    /// No-wrap containers have `containerSize.width == .greatestFiniteMagnitude`, so per-line backgrounds fill the laid-out width (or the text view bounds, whichever is larger).
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

    /// Reused across rectArray() calls; safe because NSTextView reads the returned rects synchronously before yielding.
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

                    if self.hoveredRowIndices.contains(lineIndex) {
                        var hoverRect = lineRect
                        hoverRect.origin.x = origin.x
                        hoverRect.origin.y += origin.y
                        hoverRect.size.width = drawWidth
                        self.hoverBgColor.setFill()
                        hoverRect.fill()
                    }

                    // The selection highlight is inset past the marker columns, so the stripe stays visible on selected lines.
                    if self.lineStripeWidth > 0,
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

        drawNoteBubbles(visibleGlyphRange: glyphsToShow, at: origin)

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

    /// Each bubble is one rounded card hugging its note rows: it starts at the text's leading edge (the anchor line's first character), sizes to its widest row, and caps at the view width.
    private func drawNoteBubbles(visibleGlyphRange: NSRange, at origin: NSPoint) {
        guard !noteBubbleRanges.isEmpty else { return }
        let drawWidth = lineBackgroundFillWidth
        for charRange in noteBubbleRanges {
            let glyphs = glyphRange(forCharacterRange: charRange, actualCharacterRange: nil)
            guard glyphs.length > 0,
                  NSIntersectionRange(glyphs, visibleGlyphRange).length > 0
            else { continue }
            var textRect: NSRect?
            enumerateLineFragments(forGlyphRange: glyphs) { _, usedRect, _, lineGlyphRange, _ in
                guard NSIntersectionRange(lineGlyphRange, glyphs).length > 0 else { return }
                textRect = textRect.map { $0.union(usedRect) } ?? usedRect
            }
            guard let textRect else { continue }
            let bubbleMinX = max(2, textRect.minX - DiffNoteBubbleMetrics.innerPadding)
            let bubbleMaxX = min(textRect.maxX + DiffNoteBubbleMetrics.innerPadding, drawWidth - 6)
            guard bubbleMaxX > bubbleMinX else { continue }
            let rect = NSRect(
                x: bubbleMinX,
                y: textRect.minY,
                width: bubbleMaxX - bubbleMinX,
                height: textRect.height
            )
            .insetBy(dx: 0, dy: -3)
            .offsetBy(dx: origin.x, dy: origin.y)
            let path = NSBezierPath(roundedRect: rect, xRadius: 6, yRadius: 6)
            noteBubbleFill.setFill()
            path.fill()
            noteBubbleStroke.setStroke()
            path.lineWidth = 1
            path.stroke()
        }
    }

    /// NSLayoutManager coalesces full-line selections into a single multi-line rect, so clamping super's output can't work; recompute one rect per line fragment, each clamped to that line's used-text width.
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

        let inset = perLine.compactMap(clampedToLeadingInset).filter { $0.width > 0 }
        if inset.isEmpty {
            rectCount.pointee = 0
            return nil
        }
        let buf = ensureRectBuffer(capacity: inset.count)
        for (i, r) in inset.enumerated() {
            buf[i] = r
        }
        rectCount.pointee = inset.count
        return buf.baseAddress
    }

    func clampedToLeadingInset(_ rect: NSRect) -> NSRect? {
        guard selectionHighlightLeadingInset > 0 else { return rect }
        let minX = max(rect.minX, selectionHighlightLeadingInset)
        guard minX < rect.maxX else { return nil }
        return NSRect(x: minX, y: rect.minY, width: rect.maxX - minX, height: rect.height)
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
