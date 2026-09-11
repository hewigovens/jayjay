import AppKit

final class DescriptionScrollView: NSScrollView {
    let textView = NSTextView()
    let editButton = NSButton(title: "Edit", target: nil, action: nil)
    var onEdit: (() -> Void)?
    private let content = DescriptionDocumentView()
    private var bodyFont: NSFont?
    private var titleFont: NSFont?
    private var expanded = false
    private var showsEditButton = false

    init() {
        super.init(frame: .zero)
        drawsBackground = false
        hasVerticalScroller = true
        autohidesScrollers = true
        scrollerStyle = .overlay
        textView.isEditable = false
        textView.isSelectable = true
        textView.isRichText = false
        textView.drawsBackground = false
        textView.textColor = .labelColor
        textView.textContainerInset = .zero
        textView.textContainer?.lineFragmentPadding = 0
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.textContainer?.widthTracksTextView = true
        editButton.isBordered = false
        editButton.imagePosition = .imageLeading
        editButton.contentTintColor = .secondaryLabelColor
        editButton.target = self
        editButton.action = #selector(editDescription)
        editButton.setAccessibilityLabel("Edit description")
        editButton.toolTip = "Edit description"
        content.addSubview(textView)
        content.addSubview(editButton)
        documentView = content
        setAccessibilityIdentifier(AID.Detail.description)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setDescription(_ description: String, font: NSFont, titleFont: NSFont) {
        let text = description.trimmingCharacters(in: .newlines)
        let changed = textView.string != text
        let canEdit = onEdit != nil
        guard changed || bodyFont != font || self.titleFont != titleFont || showsEditButton != canEdit else { return }
        showsEditButton = canEdit
        editButton.isHidden = !canEdit
        editButton.font = .systemFont(ofSize: titleFont.pointSize * 12 / 14)
        editButton.image = NSImage(systemSymbolName: "pencil", accessibilityDescription: nil)?
            .withSymbolConfiguration(.init(pointSize: titleFont.pointSize, weight: .semibold))
        editButton.sizeToFit()
        let selection = textView.selectedRange()
        let attributed = NSMutableAttributedString(string: text, attributes: [
            .font: font,
            .foregroundColor: NSColor.secondaryLabelColor
        ])
        let titleRange = (text as NSString).lineRange(for: NSRange(location: 0, length: 0))
        attributed.addAttributes([.font: titleFont, .foregroundColor: NSColor.labelColor], range: titleRange)
        if canEdit {
            // Keep the inline button outside the overlay scroller's hit area.
            let paragraph = NSMutableParagraphStyle()
            paragraph.tailIndent = -(editButton.frame.width + 8 + 20)
            attributed.addAttribute(.paragraphStyle, value: paragraph, range: titleRange)
        }
        textView.textStorage?.setAttributedString(attributed)
        bodyFont = font
        self.titleFont = titleFont
        if changed {
            contentView.scroll(to: .zero)
        } else {
            textView.setSelectedRange(selection)
        }
    }

    @objc private func editDescription() {
        onEdit?()
    }

    func setExpanded(_ expanded: Bool) {
        guard self.expanded != expanded else { return }
        self.expanded = expanded
        contentView.scroll(to: .zero)
    }

    func contentHeight(for width: CGFloat) -> CGFloat {
        guard let container = textView.textContainer, let layout = textView.layoutManager else { return 0 }
        if textView.frame.width != width {
            textView.setFrameSize(CGSize(width: width, height: textView.frame.height))
        }
        layout.ensureLayout(for: container)
        var height = ceil(layout.usedRect(for: container).height)
        if showsEditButton {
            var origin = CGPoint.zero
            if layout.numberOfGlyphs > 0 {
                let line = layout.lineFragmentUsedRect(forGlyphAt: 0, effectiveRange: nil)
                origin = CGPoint(x: line.maxX + 8, y: max(0, line.midY - editButton.frame.height / 2))
            }
            editButton.setFrameOrigin(origin)
            height = max(height, ceil(editButton.frame.maxY))
        }
        content.setFrameSize(CGSize(width: width, height: height))
        if textView.frame.height != height {
            textView.setFrameSize(CGSize(width: width, height: height))
        }
        return height
    }
}

private final class DescriptionDocumentView: NSView {
    override var isFlipped: Bool {
        true
    }
}
