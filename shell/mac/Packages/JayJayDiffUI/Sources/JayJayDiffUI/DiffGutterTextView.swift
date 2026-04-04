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
    var isDraggingLineSelection = false
    var pendingMenuActions: [(() -> Void)?] = []
    var menuProvider: ((DiffGutterSelection) -> [DiffGutterMenuItem])?
    var onSelectionChanged: ((DiffGutterSelection) -> Void)?
    var toggleLineCheckbox: ((Int) -> Void)?
    var checkboxHitWidth: CGFloat = 0
    var externalSelection: ClosedRange<Int>? {
        didSet { applyExternalSelection() }
    }

    override func mouseDown(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        if checkboxHitWidth > 0,
           point.x <= textContainerInset.width + checkboxHitWidth,
           let lineIndex = lineIndex(at: point),
           entries[safe: lineIndex]?.style.isChanged == true
        {
            toggleLineCheckbox?(lineIndex + 1)
            return
        }

        guard let lineNumber = lineNumber(for: event) else {
            super.mouseDown(with: event)
            return
        }

        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        if modifiers.contains(.shift), let anchor = selectionAnchorLine {
            selectLines(min(anchor, lineNumber) ... max(anchor, lineNumber))
        } else {
            selectionAnchorLine = lineNumber
            selectLines(lineNumber ... lineNumber)
        }
        isDraggingLineSelection = true
    }

    override func mouseDragged(with event: NSEvent) {
        guard isDraggingLineSelection,
              let anchor = selectionAnchorLine,
              let lineNumber = lineNumber(for: event)
        else {
            super.mouseDragged(with: event)
            return
        }

        selectLines(min(anchor, lineNumber) ... max(anchor, lineNumber))
    }

    override func mouseUp(with event: NSEvent) {
        isDraggingLineSelection = false
        super.mouseUp(with: event)
    }

    override func menu(for event: NSEvent) -> NSMenu? {
        if let lineNumber = lineNumber(for: event) {
            let current = selectedLineRange
            if current == nil || !(current!.contains(lineNumber)) {
                selectionAnchorLine = lineNumber
                selectLines(lineNumber ... lineNumber)
            }
        }

        guard let selection = currentSelection,
              let menuProvider
        else { return nil }

        let items = menuProvider(selection)
        guard !items.isEmpty else { return nil }

        pendingMenuActions = items.map(\.action)
        let menu = NSMenu()
        menu.allowsContextMenuPlugIns = false
        menu.autoenablesItems = false
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
        let point = convert(event.locationInWindow, from: nil)
        return lineIndex(at: point)
    }

    private func lineNumber(for event: NSEvent) -> Int? {
        lineIndex(for: event).map { $0 + 1 }
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

        if let lastIndex = entries.indices.last,
           charIndex >= NSMaxRange(entries[lastIndex].range)
        {
            return lastIndex
        }
        return nil
    }

    private func selectLines(_ range: ClosedRange<Int>) {
        let lowerIndex = range.lowerBound - 1
        let upperIndex = range.upperBound - 1
        guard let lower = entries[safe: lowerIndex],
              let upper = entries[safe: upperIndex]
        else { return }
        let selectedRange = NSRange(
            location: lower.range.location,
            length: NSMaxRange(upper.range) - lower.range.location
        )
        setSelectedRange(selectedRange)
        if let selection = currentSelection {
            onSelectionChanged?(selection)
        }
    }

    private func applyExternalSelection() {
        guard let externalSelection else { return }
        guard selectedLineRange != externalSelection else { return }
        selectionAnchorLine = externalSelection.lowerBound
        selectLines(externalSelection)
    }

    private var selectedLineRange: ClosedRange<Int>? {
        let selected = selectedRange()
        guard selected.length > 0 else { return nil }
        guard let lower = entries.firstIndex(where: { NSIntersectionRange($0.range, selected).length > 0 }),
              let upper = entries.lastIndex(where: { NSIntersectionRange($0.range, selected).length > 0 })
        else { return nil }
        return (lower + 1) ... (upper + 1)
    }

    private var currentSelection: DiffGutterSelection? {
        guard let lineRange = selectedLineRange else { return nil }
        let lowerIndex = lineRange.lowerBound - 1
        let upperIndex = lineRange.upperBound - 1
        guard entries.indices.contains(lowerIndex), entries.indices.contains(upperIndex) else { return nil }
        let changedCount = entries[lowerIndex ... upperIndex].reduce(into: 0) { count, entry in
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

private extension DiffSpanStyle {
    var isChanged: Bool {
        self == .added || self == .removed
    }
}
