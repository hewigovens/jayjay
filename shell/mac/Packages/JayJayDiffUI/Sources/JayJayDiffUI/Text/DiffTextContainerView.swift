import AppKit

public final class DiffTextContainerView: NSView {
    let gutterScrollView: NSScrollView
    let gutterTextView: DiffGutterTextView
    let scrollView: NSScrollView
    let textView: NSTextView
    var hoveredLinkRange: NSRange?
    private let separatorView = NSView()
    private var isSyncingScroll = false
    private var lastContentWidth: CGFloat = -1
    private var needsWidthPassAfterResize = false
    private(set) var gutterWidth: CGFloat = 0
    var onContentLayoutChanged: (() -> Void)?
    var onContentHeightChanged: ((CGFloat) -> Void)?
    private var lastReportedHeight: CGFloat = -1
    var viewportLineLocations: [DiffViewportLineLocation] = []
    var pendingViewportAnchor: DiffViewportAnchor?
    var pendingRevealFeedback: PendingDiffRevealFeedback?
    var lastSelectionResetGeneration: UInt64?
    var lastRevealFeedbackGeneration: UInt64?
    /// SBS rows are pre-wrapped by `wrap_sbs_rows`, so SBS callers set this `false`
    /// to stop the text container from re-wrapping on top.
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
        // Empty so link ranges keep their string attributes; the default blue would repaint the quiet separator links.
        textView.linkTextAttributes = [:]
        super.init(frame: .zero)
        addTrackingArea(NSTrackingArea(
            rect: .zero,
            options: [.mouseMoved, .mouseEnteredAndExited, .activeInKeyWindow, .inVisibleRect],
            owner: self
        ))

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

        // Re-wrapping the whole document on every live-resize tick stutters the drag; keep the old text layout during the drag and run one width pass at the end.
        if inLiveResize, lastContentWidth >= 0 {
            needsWidthPassAfterResize = true
            return
        }
        applyContentWidth(max(0, scrollView.contentSize.width))
        applyPendingViewportUpdates()
    }

    override public func viewDidEndLiveResize() {
        super.viewDidEndLiveResize()
        guard needsWidthPassAfterResize else { return }
        needsWidthPassAfterResize = false
        applyContentWidth(max(0, scrollView.contentSize.width))
    }

    private func applyContentWidth(_ contentWidth: CGFloat) {
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
        // An infinite container never re-wraps, so width changes need no layout invalidation there.
        if wrapsText {
            textView.layoutManager?.invalidateLayout(
                forCharacterRange: NSRange(location: 0, length: (textView.string as NSString).length),
                actualCharacterRange: nil
            )
        }
        onContentLayoutChanged?()
        reportContentHeightIfNeeded()
    }

    /// Sized-to-content hosts (Diff Edit cards) disable inner scrolling so the outer scroll view owns the wheel; the default scrolling host keeps its own scroller.
    func setFitsContent(_ fitsContent: Bool) {
        let elasticity: NSScrollView.Elasticity = fitsContent ? .none : .automatic
        scrollView.verticalScrollElasticity = elasticity
        gutterScrollView.verticalScrollElasticity = elasticity
        scrollView.hasVerticalScroller = !fitsContent
    }

    func reportContentHeightIfNeeded() {
        guard let onContentHeightChanged else { return }
        let height = max(documentHeight(of: textView), documentHeight(of: gutterTextView))
        guard height > 0, abs(height - lastReportedHeight) > 0.5 else { return }
        lastReportedHeight = height
        // Reported async: updateNSView runs inside a SwiftUI update pass where setting @State is illegal.
        DispatchQueue.main.async {
            onContentHeightChanged(height)
        }
    }

    private func documentHeight(of textView: NSTextView) -> CGFloat {
        guard let layoutManager = textView.layoutManager,
              let textContainer = textView.textContainer
        else { return 0 }
        layoutManager.ensureLayout(for: textContainer)
        return layoutManager.usedRect(for: textContainer).height + textView.textContainerInset.height * 2
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

    public override func mouseMoved(with event: NSEvent) {
        updateLinkHover(at: event.locationInWindow)
    }

    public override func mouseExited(with event: NSEvent) {
        clearLinkHover()
    }
}
