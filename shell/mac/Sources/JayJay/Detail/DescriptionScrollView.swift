import AppKit

final class DescriptionScrollView: NSScrollView {
    let textView = NSTextView()
    private var expanded = false
    private var needsScrollReset = false
    // An animated resize asks for the height at its final width, so sizing must not re-wrap the visible text.
    private let measuringStorage = NSTextStorage()
    private let measuringLayout = NSLayoutManager()
    private let measuringContainer = NSTextContainer()

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
        textView.autoresizingMask = [.width]
        textView.maxSize = CGSize(width: CGFloat.greatestFiniteMagnitude, height: .greatestFiniteMagnitude)
        textView.textContainer?.widthTracksTextView = true
        measuringContainer.lineFragmentPadding = 0
        measuringLayout.addTextContainer(measuringContainer)
        measuringStorage.addLayoutManager(measuringLayout)
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
        measuringStorage.setAttributedString(NSAttributedString(string: text, attributes: [.font: font]))
        if changed {
            contentView.scroll(to: .zero)
        } else {
            textView.setSelectedRange(selection)
        }
    }

    func setExpanded(_ expanded: Bool) {
        guard self.expanded != expanded else { return }
        self.expanded = expanded
        needsScrollReset = true
        needsLayout = true
    }

    /// Scrolling waits for layout so the reset applies to the resized clip view.
    override func layout() {
        super.layout()
        guard needsScrollReset else { return }
        needsScrollReset = false
        contentView.scroll(to: .zero)
        reflectScrolledClipView(contentView)
    }

    func wholeLines(fitting height: CGFloat) -> CGFloat {
        guard let font = textView.font, let lineHeight = textView.layoutManager?.defaultLineHeight(for: font) else { return height }
        return max(1, floor(height / lineHeight)) * lineHeight
    }

    func contentHeight(for width: CGFloat) -> CGFloat {
        guard measuringStorage.length > 0 else { return 0 }
        measuringContainer.size = CGSize(width: width, height: .greatestFiniteMagnitude)
        measuringLayout.ensureLayout(for: measuringContainer)
        return ceil(measuringLayout.usedRect(for: measuringContainer).height)
    }
}
