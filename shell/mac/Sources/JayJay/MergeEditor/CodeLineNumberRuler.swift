import AppKit

final class CodeLineNumberRuler: NSRulerView {
    private var numbersByCharacter = [0: 1]
    private var characterCount = 0

    init(scrollView: NSScrollView, textView: NSTextView) {
        super.init(scrollView: scrollView, orientation: .verticalRuler)
        clipsToBounds = true
        clientView = textView
        ruleThickness = 40
        setAccessibilityElement(true)
        setAccessibilityLabel("Line numbers")
        updateText()
    }

    @available(*, unavailable)
    required init(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var isFlipped: Bool {
        true
    }

    private var font: NSFont {
        (clientView as? NSTextView)?.font ?? .monospacedSystemFont(ofSize: AppSettings.defaultFontSize, weight: .regular)
    }

    func updateText() {
        guard let textView = clientView as? NSTextView else { return }
        numbersByCharacter = [0: 1]
        characterCount = textView.string.utf16.count
        var number = 1
        for (index, unit) in textView.string.utf16.enumerated() where unit == 10 {
            number += 1
            numbersByCharacter[index + 1] = number
        }
        let width = (String(number) as NSString).size(withAttributes: [.font: font]).width
        let thickness = max(40, ceil(width) + 16)
        if ruleThickness != thickness {
            ruleThickness = thickness
        }
        needsDisplay = true
    }

    func lineLabels(in rect: NSRect) -> [(number: Int, origin: NSPoint)] {
        guard let textView = clientView as? NSTextView,
              let layout = textView.layoutManager, let container = textView.textContainer else { return [] }
        let origin = textView.textContainerOrigin
        let containerRect = rect.offsetBy(dx: -origin.x, dy: -origin.y)
        layout.ensureLayout(forBoundingRect: containerRect, in: container)
        let glyphs = layout.glyphRange(forBoundingRect: containerRect, in: container)
        var labels: [(number: Int, origin: NSPoint)] = []
        layout.enumerateLineFragments(forGlyphRange: glyphs) { fragment, _, _, range, _ in
            let character = layout.characterIndexForGlyph(at: range.location)
            guard let number = self.numbersByCharacter[character] else { return }
            labels.append((number, NSPoint(x: origin.x, y: origin.y + fragment.minY)))
        }
        if layout.extraLineFragmentTextContainer === container,
           let number = numbersByCharacter[characterCount],
           layout.extraLineFragmentRect.intersects(containerRect)
        {
            labels.append((number, NSPoint(x: origin.x, y: origin.y + layout.extraLineFragmentRect.minY)))
        }
        return labels
    }

    override func drawHashMarksAndLabels(in _: NSRect) {
        guard let textView = clientView as? NSTextView else { return }
        NSColor.textBackgroundColor.setFill()
        bounds.fill()
        let attributes: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: NSColor.secondaryLabelColor
        ]
        for label in lineLabels(in: textView.visibleRect) {
            let text = String(label.number) as NSString
            let point = convert(label.origin, from: textView)
            text.draw(at: NSPoint(x: ruleThickness - 8 - text.size(withAttributes: attributes).width, y: point.y), withAttributes: attributes)
        }
        NSColor.separatorColor.setFill()
        NSRect(x: bounds.maxX - 1, y: bounds.minY, width: 1, height: bounds.height).fill()
    }
}
