import AppKit

extension NSTextView {
    /// Line heights change with the font, so a kept scroll offset would land on a different line.
    func keepingTopLine(_ relayout: () -> Void) {
        guard let clip = enclosingScrollView?.contentView, let layout = layoutManager, let container = textContainer,
              layout.numberOfGlyphs > 0
        else { return relayout() }
        let top = NSPoint(x: 0, y: max(0, clip.bounds.minY - textContainerOrigin.y))
        let character = layout.characterIndexForGlyph(at: layout.glyphIndex(for: top, in: container))
        relayout()
        layout.ensureLayout(for: container)
        let glyph = layout.glyphIndexForCharacter(at: character)
        let y = layout.lineFragmentRect(forGlyphAt: glyph, effectiveRange: nil).minY + textContainerOrigin.y
        clip.scroll(to: NSPoint(x: clip.bounds.minX, y: min(y, max(0, frame.height - clip.bounds.height))))
        enclosingScrollView?.reflectScrolledClipView(clip)
    }
}
