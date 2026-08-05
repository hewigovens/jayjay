import AppKit

extension DiffTextContainerView {
    /// Temporary attributes keep the hover underline out of the text storage, so link styling never dirties the rendered diff.
    func updateLinkHover(at locationInWindow: NSPoint) {
        guard let layoutManager = textView.layoutManager,
              let textContainer = textView.textContainer,
              let storage = textView.textStorage
        else { return }
        var point = textView.convert(locationInWindow, from: nil)
        point.x -= textView.textContainerInset.width
        point.y -= textView.textContainerInset.height

        let glyphIndex = layoutManager.glyphIndex(for: point, in: textContainer)
        let glyphRect = layoutManager.boundingRect(
            forGlyphRange: NSRange(location: glyphIndex, length: 1),
            in: textContainer
        )
        guard glyphRect.contains(point) else {
            clearLinkHover()
            return
        }
        let charIndex = layoutManager.characterIndexForGlyph(at: glyphIndex)
        guard charIndex < storage.length else {
            clearLinkHover()
            return
        }
        var linkRange = NSRange(location: NSNotFound, length: 0)
        let link = storage.attribute(.link, at: charIndex, effectiveRange: &linkRange)
        guard link != nil, linkRange.location != NSNotFound else {
            clearLinkHover()
            return
        }
        if hoveredLinkRange == linkRange {
            return
        }
        clearLinkHover()
        layoutManager.addTemporaryAttribute(
            .underlineStyle,
            value: NSUnderlineStyle.single.rawValue,
            forCharacterRange: linkRange
        )
        hoveredLinkRange = linkRange
    }

    func clearLinkHover() {
        guard let hoveredLinkRange, let layoutManager = textView.layoutManager else {
            hoveredLinkRange = nil
            return
        }
        let length = textView.textStorage?.length ?? 0
        let clamped = NSIntersectionRange(hoveredLinkRange, NSRange(location: 0, length: length))
        if clamped.length > 0 {
            layoutManager.removeTemporaryAttribute(.underlineStyle, forCharacterRange: clamped)
        }
        self.hoveredLinkRange = nil
    }
}
