import AppKit

/// Adds the same height to every line fragment, so views laid out separately (gutter and text) stay row-aligned.
open class LineSpacingLayoutManager: NSLayoutManager, NSLayoutManagerDelegate {
    public private(set) var extraLineHeight: CGFloat = 0

    override public init() {
        super.init()
        delegate = self
    }

    public required init?(coder: NSCoder) {
        super.init(coder: coder)
        delegate = self
    }

    public func setLineSpacing(_ multiple: Double, for font: NSFont) {
        let extra = (defaultLineHeight(for: font) * (max(multiple, 1) - 1)).rounded()
        guard extra != extraLineHeight else { return }
        extraLineHeight = extra
        invalidateLayout(forCharacterRange: NSRange(location: 0, length: textStorage?.length ?? 0), actualCharacterRange: nil)
    }

    /// The empty line after a trailing newline skips the delegate below.
    override open func setExtraLineFragmentRect(_ fragmentRect: NSRect, usedRect: NSRect, textContainer container: NSTextContainer) {
        var fragment = fragmentRect
        var used = usedRect
        if !fragment.isEmpty {
            fragment.size.height += extraLineHeight
            used.size.height += extraLineHeight
        }
        super.setExtraLineFragmentRect(fragment, usedRect: used, textContainer: container)
    }

    public func layoutManager(
        _: NSLayoutManager,
        shouldSetLineFragmentRect lineFragmentRect: UnsafeMutablePointer<NSRect>,
        lineFragmentUsedRect: UnsafeMutablePointer<NSRect>,
        baselineOffset: UnsafeMutablePointer<CGFloat>,
        in _: NSTextContainer,
        forGlyphRange _: NSRange
    ) -> Bool {
        guard extraLineHeight > 0 else { return false }
        lineFragmentRect.pointee.size.height += extraLineHeight
        lineFragmentUsedRect.pointee.size.height += extraLineHeight
        baselineOffset.pointee += extraLineHeight / 2
        return true
    }
}
