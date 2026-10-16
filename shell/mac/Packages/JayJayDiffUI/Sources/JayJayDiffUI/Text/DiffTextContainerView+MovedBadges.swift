import AppKit

extension DiffTextContainerView {
    func setMovedBadges(_ badges: [DiffMovedBadge], font: NSFont, onJump: @escaping (Int) -> Void) {
        movedBadges = badges
        movedBadgeFont = font
        onMovedBadge = onJump
        layoutMovedBadges()
    }

    func layoutMovedBadges() {
        guard !movedBadges.isEmpty || !movedBadgeButtons.isEmpty else { return }
        while movedBadgeButtons.count < movedBadges.count {
            let button = DiffMovedBadgeButton()
            textView.addSubview(button)
            movedBadgeButtons.append(button)
        }
        let text = textView.string as NSString
        let starts = movedBadges.isEmpty ? [] : text.lineStarts()
        for (index, button) in movedBadgeButtons.enumerated() {
            guard index < movedBadges.count,
                  let rect = lineRect(movedBadges[index].line, in: text, starts: starts)
            else {
                button.isHidden = true
                continue
            }
            let badge = movedBadges[index]
            let size = button.configure(title: badge.title, font: movedBadgeFont) { [weak self] in
                self?.onMovedBadge?(badge.target)
            }
            button.frame = NSRect(
                x: textView.bounds.width - size.width - DiffMovedBadgeButton.trailingInset,
                y: rect.midY - size.height / 2,
                width: size.width,
                height: size.height
            )
            button.isHidden = false
        }
    }

    func revealLine(_ index: Int) {
        let text = textView.string as NSString
        guard let range = characterRange(ofLine: index, in: text, starts: text.lineStarts()) else { return }
        textView.scrollRangeToVisible(range)
        textView.showFindIndicator(for: range)
    }

    private func lineRect(_ index: Int, in text: NSString, starts: [Int]) -> NSRect? {
        guard let range = characterRange(ofLine: index, in: text, starts: starts),
              let layoutManager = textView.layoutManager
        else { return nil }
        let glyph = layoutManager.glyphIndexForCharacter(at: range.location)
        let rect = layoutManager.lineFragmentRect(forGlyphAt: glyph, effectiveRange: nil)
        return rect.offsetBy(dx: textView.textContainerOrigin.x, dy: textView.textContainerOrigin.y)
    }

    private func characterRange(ofLine index: Int, in text: NSString, starts: [Int]) -> NSRange? {
        guard index < starts.count else { return nil }
        let start = starts[index]
        let end = index + 1 < starts.count ? starts[index + 1] : text.length
        var range = NSRange(location: start, length: end - start)
        if range.length > 0, text.character(at: NSMaxRange(range) - 1) == 0x0A {
            range.length -= 1
        }
        return range
    }
}
