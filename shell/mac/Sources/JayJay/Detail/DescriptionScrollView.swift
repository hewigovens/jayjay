import AppKit

final class DescriptionScrollView: NSScrollView {
    let textView = NSTextView()
    private var expanded = false

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
        textView.textColor = .secondaryLabelColor
        textView.textContainerInset = .zero
        textView.textContainer?.lineFragmentPadding = 0
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.textContainer?.widthTracksTextView = true
        documentView = textView
        setAccessibilityIdentifier(AID.Detail.descriptionBody)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setDescription(_ text: String, font: NSFont) {
        let changed = textView.string != text
        guard changed || textView.font != font else { return }
        let selection = textView.selectedRange()
        textView.string = text
        textView.font = font
        if changed {
            contentView.scroll(to: .zero)
        } else {
            textView.setSelectedRange(selection)
        }
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
        let height = textView.string.isEmpty ? 0 : ceil(layout.usedRect(for: container).height)
        if textView.frame.height != height {
            textView.setFrameSize(CGSize(width: width, height: height))
        }
        return height
    }
}
