import AppKit

public final class DiffTextContainerView: NSView {
    let gutterScrollView: NSScrollView
    let gutterTextView: DiffGutterTextView
    let scrollView: NSScrollView
    let textView: NSTextView
    private let separatorView = NSView()
    private var isSyncingScroll = false
    private var lastContentWidth: CGFloat = -1
    private(set) var gutterWidth: CGFloat = 0
    var onContentLayoutChanged: (() -> Void)?
    /// SBS uses two independent containers, so per-side wrapping desyncs the panes
    /// and gutters. Until we wrap to the tallest side + insert continuation rows,
    /// SBS callers set this `false` so the text container keeps an unbounded width.
    var wrapsText: Bool = true

    override public var isFlipped: Bool {
        true
    }

    init(
        gutterScrollView: NSScrollView,
        gutterTextView: DiffGutterTextView,
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
    public required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func updateGutterWidth(_ width: CGFloat) {
        guard abs(gutterWidth - width) > 0.5 else { return }
        gutterWidth = width
        needsLayout = true
    }

    override public func layout() {
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

        let contentWidth = max(0, scrollView.contentSize.width)
        if abs(textView.frame.width - contentWidth) > 0.5 {
            textView.frame.size.width = contentWidth
        }

        guard abs(lastContentWidth - contentWidth) > 0.5 else { return }
        lastContentWidth = contentWidth
        // widthTracksTextView=true forces the container width to follow the textView,
        // which would still cause wrapping. Decouple it when wrap is disabled.
        textView.textContainer?.widthTracksTextView = wrapsText
        textView.textContainer?.containerSize = NSSize(
            width: wrapsText ? contentWidth : CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        textView.layoutManager?.invalidateLayout(
            forCharacterRange: NSRange(location: 0, length: (textView.string as NSString).length),
            actualCharacterRange: nil
        )
        onContentLayoutChanged?()
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
