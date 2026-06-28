import AppKit

extension DiffLayoutManager {
    func drawSelectedRangeHighlights(
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
            let lineRects = selectionRects(forCharacterRange: range, in: container, visibleGlyphRange: visibleGlyphRange)
                .compactMap(clampedToLeadingInset)
            for run in coalescedVerticalRuns(lineRects) {
                let drawRect = run
                    .offsetBy(dx: origin.x, dy: origin.y)
                    .insetBy(dx: -1, dy: 1)
                NSBezierPath(roundedRect: drawRect, xRadius: 2, yRadius: 2).fill()
            }
        }
    }

    /// Merges vertically adjacent, equal-width line rects so a contiguous multi-line selection draws as one block instead of per-line pills with gaps between rows. Ragged selections (differing widths, as in content text) keep their per-line rects.
    private func coalescedVerticalRuns(_ rects: [NSRect]) -> [NSRect] {
        var runs: [NSRect] = []
        for rect in rects.sorted(by: { $0.minY < $1.minY }) {
            if let last = runs.last,
               rect.minY <= last.maxY + 1,
               abs(rect.minX - last.minX) < 0.5,
               abs(rect.maxX - last.maxX) < 0.5
            {
                runs[runs.count - 1] = last.union(rect)
            } else {
                runs.append(rect)
            }
        }
        return runs
    }

    func drawFindMatchHighlights(
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

    func selectionRects(
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
}
