import AppKit

extension DiffGutterTextView {
    func hoverLineNumber(at point: NSPoint) -> Int? {
        guard let layoutManager, let textContainer else { return nil }
        let containerPoint = NSPoint(
            x: point.x - textContainerInset.width,
            y: point.y - textContainerInset.height
        )
        let glyphIndex = layoutManager.glyphIndex(for: containerPoint, in: textContainer)
        // glyphIndex clamps to the last line; reject points below the laid-out text so empty space doesn't hover the last row.
        let fragmentRect = layoutManager.lineFragmentRect(forGlyphAt: glyphIndex, effectiveRange: nil)
        guard containerPoint.y <= fragmentRect.maxY else { return nil }
        let charIndex = layoutManager.characterIndexForGlyph(at: glyphIndex)
        return entry(atCharIndex: charIndex)?.lineNumber
    }

    /// Binary search over `entries` (built in ascending range order).
    private func entry(atCharIndex charIndex: Int) -> Entry? {
        var low = 0
        var high = entries.count - 1
        while low <= high {
            let mid = (low + high) / 2
            let candidate = entries[mid]
            if charIndex < candidate.range.location {
                high = mid - 1
            } else if charIndex >= NSMaxRange(candidate.range) {
                low = mid + 1
            } else {
                return candidate
            }
        }
        return nil
    }

    /// Anchor at the gutter's trailing edge for the line's row, so the popover arrow points at the start of the code line instead of the marker dot.
    func noteAnchorRect(for entry: Entry) -> NSRect {
        guard let layoutManager, let textContainer else { return .zero }
        let glyphRange = layoutManager.glyphRange(forCharacterRange: entry.range, actualCharacterRange: nil)
        var rect = layoutManager.boundingRect(forGlyphRange: glyphRange, in: textContainer)
        rect.origin.y += textContainerInset.height
        rect.size.width = 4
        rect.origin.x = bounds.maxX - rect.width
        return rect
    }

    func isInNoteColumn(_ point: NSPoint) -> Bool {
        guard noteHitWidth > 0 else { return false }
        let x = point.x - textContainerInset.width
        return x >= noteHitStart && x <= noteHitStart + noteHitWidth
    }

    func isInGroupColumn(_ point: NSPoint) -> Bool {
        guard groupHitWidth > 0 else { return false }
        let x = point.x - textContainerInset.width
        return x >= 0 && x <= groupHitWidth
    }

    func isInCheckboxColumn(_ point: NSPoint) -> Bool {
        guard checkboxHitWidth > 0 else { return false }
        let x = point.x - textContainerInset.width
        return x >= checkboxHitStart && x <= checkboxHitStart + checkboxHitWidth
    }

    func entryIndex(at point: NSPoint) -> Int? {
        guard let layoutManager,
              let textContainer
        else { return nil }

        let containerPoint = NSPoint(
            x: point.x - textContainerInset.width,
            y: point.y - textContainerInset.height
        )
        let fraction = UnsafeMutablePointer<CGFloat>.allocate(capacity: 1)
        defer { fraction.deallocate() }
        let glyphIndex = layoutManager.glyphIndex(
            for: containerPoint,
            in: textContainer,
            fractionOfDistanceThroughGlyph: fraction
        )
        let charIndex = layoutManager.characterIndexForGlyph(at: glyphIndex)
        if let found = entries.firstIndex(where: { NSLocationInRange(charIndex, $0.range) }) {
            return found
        }

        if let lastIndex = entries.indices.last,
           charIndex >= NSMaxRange(entries[lastIndex].range)
        {
            return lastIndex
        }
        return nil
    }
}
