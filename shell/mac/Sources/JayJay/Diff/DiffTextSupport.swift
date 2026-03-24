import AppKit

final class DiffTextContainerView: NSView {
    let gutterScrollView: NSScrollView
    let gutterTextView: NSTextView
    let scrollView: NSScrollView
    let textView: NSTextView
    private let separatorView = NSView()
    private var isSyncingScroll = false
    private(set) var gutterWidth: CGFloat = 0

    override var isFlipped: Bool {
        true
    }

    init(
        gutterScrollView: NSScrollView,
        gutterTextView: NSTextView,
        scrollView: NSScrollView,
        textView: NSTextView
    ) {
        self.gutterScrollView = gutterScrollView
        self.gutterTextView = gutterTextView
        self.scrollView = scrollView
        self.textView = textView
        super.init(frame: .zero)

        separatorView.wantsLayer = true
        separatorView.layer?.backgroundColor = NSColor.separatorColor.withAlphaComponent(0.22).cgColor

        addSubview(gutterScrollView)
        addSubview(separatorView)
        addSubview(scrollView)

        gutterScrollView.contentView.postsBoundsChangedNotifications = true
        scrollView.contentView.postsBoundsChangedNotifications = true
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(gutterScrolled),
            name: NSView.boundsDidChangeNotification,
            object: gutterScrollView.contentView
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(contentScrolled),
            name: NSView.boundsDidChangeNotification,
            object: scrollView.contentView
        )
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func updateGutterWidth(_ width: CGFloat) {
        gutterWidth = width
        needsLayout = true
    }

    override func layout() {
        super.layout()

        let gutter = gutterWidth
        gutterScrollView.frame = NSRect(x: 0, y: 0, width: gutter, height: bounds.height)
        separatorView.frame = NSRect(x: gutter, y: 0, width: 1, height: bounds.height)
        scrollView.frame = NSRect(
            x: gutter + 1,
            y: 0,
            width: max(0, bounds.width - gutter - 1),
            height: bounds.height
        )
    }

    @objc private func gutterScrolled(_ notification: Notification) {
        syncScroll(from: gutterScrollView, to: scrollView)
    }

    @objc private func contentScrolled(_ notification: Notification) {
        syncScroll(from: scrollView, to: gutterScrollView)
    }

    private func syncScroll(from source: NSScrollView, to destination: NSScrollView) {
        guard !isSyncingScroll else { return }
        isSyncingScroll = true
        var origin = destination.contentView.bounds.origin
        origin.y = source.contentView.bounds.origin.y
        destination.contentView.scroll(to: origin)
        destination.reflectScrolledClipView(destination.contentView)
        isSyncingScroll = false
    }

    deinit {
        NotificationCenter.default.removeObserver(self)
    }
}

final class DiffLayoutManager: NSLayoutManager {
    var lineBgColors: [NSColor] = []

    override func drawBackground(forGlyphRange glyphsToShow: NSRange, at origin: NSPoint) {
        guard let textStorage, let textContainer = textContainers.first else {
            super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
            return
        }

        let fullText = textStorage.string as NSString
        var lineIndex = 0
        var charPos = 0

        while charPos < fullText.length {
            let lineRange = fullText.lineRange(for: NSRange(location: charPos, length: 0))
            let glyphRange = glyphRange(forCharacterRange: lineRange, actualCharacterRange: nil)
            if NSIntersectionRange(glyphRange, glyphsToShow).length > 0,
               lineIndex < lineBgColors.count
            {
                let color = lineBgColors[lineIndex]
                if color != .clear {
                    var lineRect = lineFragmentRect(forGlyphAt: glyphRange.location, effectiveRange: nil)
                    lineRect.origin.x = 0
                    lineRect.size.width = textContainer.containerSize.width
                    lineRect.origin.x += origin.x
                    lineRect.origin.y += origin.y
                    color.setFill()
                    lineRect.fill()
                }
            }

            lineIndex += 1
            charPos = NSMaxRange(lineRange)
        }

        super.drawBackground(forGlyphRange: glyphsToShow, at: origin)
    }
}
