import AppKit
import JayJayCore

struct DiffGutterSelection {
    let lineRange: ClosedRange<Int>
    let changedLineCount: Int
}

struct DiffGutterMenuItem {
    let title: String
    let enabled: Bool
    let action: (() -> Void)?

    static let separator = DiffGutterMenuItem(title: "", enabled: false, action: nil)
}

final class DiffGutterTextView: NSTextView {
    struct Entry {
        let style: DiffSpanStyle
        let range: NSRange
    }

    var entries: [Entry] = []
    var selectionAnchorLine: Int?
    var pendingMenuActions: [(() -> Void)?] = []
    var menuProvider: ((DiffGutterSelection) -> [DiffGutterMenuItem])?

    override func mouseDown(with event: NSEvent) {
        guard let lineIndex = lineIndex(for: event) else {
            super.mouseDown(with: event)
            return
        }

        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        if modifiers.contains(.shift), let anchor = selectionAnchorLine {
            selectLines(min(anchor, lineIndex) ... max(anchor, lineIndex))
        } else {
            selectionAnchorLine = lineIndex
            selectLines(lineIndex ... lineIndex)
        }
    }

    override func menu(for event: NSEvent) -> NSMenu? {
        if let lineIndex = lineIndex(for: event) {
            let current = selectedLineRange
            if current == nil || !(current!.contains(lineIndex)) {
                selectionAnchorLine = lineIndex
                selectLines(lineIndex ... lineIndex)
            }
        }

        guard let selection = currentSelection,
              let menuProvider
        else { return nil }

        let items = menuProvider(selection)
        guard !items.isEmpty else { return nil }

        pendingMenuActions = items.map(\.action)
        let menu = NSMenu()
        for (index, item) in items.enumerated() {
            if item.action == nil, item.title.isEmpty {
                menu.addItem(.separator())
                continue
            }
            let menuItem = NSMenuItem(
                title: item.title,
                action: item.action == nil ? nil : #selector(runMenuAction(_:)),
                keyEquivalent: ""
            )
            menuItem.target = self
            menuItem.tag = index
            menuItem.isEnabled = item.enabled && item.action != nil
            menu.addItem(menuItem)
        }
        return menu
    }

    @objc private func runMenuAction(_ sender: NSMenuItem) {
        guard sender.tag < pendingMenuActions.count else { return }
        pendingMenuActions[sender.tag]?()
    }

    private func lineIndex(for event: NSEvent) -> Int? {
        let pointInWindow = event.locationInWindow
        let point = convert(pointInWindow, from: nil)
        return lineIndex(at: point)
    }

    private func lineIndex(at point: NSPoint) -> Int? {
        guard let layoutManager,
              let textContainer
        else { return nil }

        let containerPoint = NSPoint(
            x: point.x - textContainerInset.width,
            y: point.y - textContainerInset.height
        )
        let fraction = UnsafeMutablePointer<CGFloat>.allocate(capacity: 1)
        defer { fraction.deallocate() }
        let glyphIndex = layoutManager.glyphIndex(
            for: containerPoint,
            in: textContainer,
            fractionOfDistanceThroughGlyph: fraction
        )
        let charIndex = layoutManager.characterIndexForGlyph(at: glyphIndex)
        if let found = entries.firstIndex(where: { NSLocationInRange(charIndex, $0.range) }) {
            return found
        }

        // Clicks below the final newline can map just past the last range.
        if let lastIndex = entries.indices.last,
           charIndex >= NSMaxRange(entries[lastIndex].range)
        {
            return lastIndex
        }
        return nil
    }

    private func selectLines(_ range: ClosedRange<Int>) {
        guard let lower = entries[safe: range.lowerBound],
              let upper = entries[safe: range.upperBound]
        else { return }
        let selectedRange = NSRange(
            location: lower.range.location,
            length: NSMaxRange(upper.range) - lower.range.location
        )
        setSelectedRange(selectedRange)
    }

    private var selectedLineRange: ClosedRange<Int>? {
        let selected = selectedRange()
        guard selected.length > 0 else { return nil }
        guard let lower = entries.firstIndex(where: { NSIntersectionRange($0.range, selected).length > 0 }),
              let upper = entries.lastIndex(where: { NSIntersectionRange($0.range, selected).length > 0 })
        else { return nil }
        return lower ... upper
    }

    private var currentSelection: DiffGutterSelection? {
        guard let lineRange = selectedLineRange else { return nil }
        let changedCount = entries[lineRange].reduce(into: 0) { count, entry in
            if entry.style == .added || entry.style == .removed {
                count += 1
            }
        }
        return DiffGutterSelection(lineRange: lineRange, changedLineCount: changedCount)
    }
}

private extension Array {
    subscript(safe index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}

final class DiffTextContainerView: NSView {
    let gutterScrollView: NSScrollView
    let gutterTextView: DiffGutterTextView
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
