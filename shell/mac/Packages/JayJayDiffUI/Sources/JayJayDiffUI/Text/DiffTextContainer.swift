import AppKit

final class DiffTextContainer: NSTextContainer {
    private var badgeWidths: [Int: CGFloat] = [:]

    override var isSimpleRectangularTextContainer: Bool {
        badgeWidths.isEmpty
    }

    func reserveMovedBadgeSpace(for badges: [DiffMovedBadge], in text: NSString, font: NSFont) {
        let starts = badges.isEmpty ? [] : text.lineStarts()
        let widths = Dictionary(uniqueKeysWithValues: badges.compactMap { badge -> (Int, CGFloat)? in
            guard badge.line < starts.count else { return nil }
            return (starts[badge.line], DiffMovedBadgeButton.reservedWidth(for: badge.title, font: font))
        })
        guard widths != badgeWidths else { return }
        badgeWidths = widths
        layoutManager?.textContainerChangedGeometry(self)
    }

    override func lineFragmentRect(
        forProposedRect proposedRect: NSRect,
        at characterIndex: Int,
        writingDirection baseWritingDirection: NSWritingDirection,
        remaining remainingRect: UnsafeMutablePointer<NSRect>?
    ) -> NSRect {
        var rect = super.lineFragmentRect(
            forProposedRect: proposedRect,
            at: characterIndex,
            writingDirection: baseWritingDirection,
            remaining: remainingRect
        )
        if let width = badgeWidths[characterIndex] {
            rect.size.width = max(1, rect.width - width)
        }
        return rect
    }
}
