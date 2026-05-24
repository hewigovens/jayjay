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

public final class DiffGutterTextView: NSTextView {
    struct Entry {
        let style: DiffSpanStyle
        let range: NSRange
        let lineNumber: Int
    }

    var entries: [Entry] = []
    var selectionAnchorLine: Int?
    var isDraggingLineSelection = false
    var pendingMenuActions: [(() -> Void)?] = []
    var menuProvider: ((DiffGutterSelection) -> [DiffGutterMenuItem])?
    var onSelectionChanged: ((DiffGutterSelection) -> Void)?
    var groupRangeProvider: ((Int) -> ClosedRange<Int>?)?
    var activateGroup: ((ClosedRange<Int>) -> Void)?
    var groupHitWidth: CGFloat = 0
    var toggleLineCheckbox: ((Int) -> Void)?
    var checkboxHitStart: CGFloat = 0
    var checkboxHitWidth: CGFloat = 0
    /// Changed line number (1-based) → its change-group index. Click hit-test only.
    var groupIndexAtLineNumber: [Int: UInt32] = [:]
    var toggleReviewCheckbox: ((UInt32) -> Void)?
    var externalSelection: ClosedRange<Int>? {
        didSet { applyExternalSelection() }
    }

    override public func mouseDown(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        if isInGroupColumn(point),
           let entryIndex = entryIndex(at: point),
           let entry = entries[safe: entryIndex],
           entry.style.isChanged
        {
            let lineNumber = entry.lineNumber
            // Review toggle takes priority over the legacy select-change-group click.
            if let toggleReviewCheckbox,
               let groupIdx = groupIndexAtLineNumber[lineNumber]
            {
                toggleReviewCheckbox(groupIdx)
                return
            }
            let range = groupRangeProvider?(lineNumber) ?? lineNumber ... lineNumber
            selectionAnchorLine = range.lowerBound
            selectLines(range)
            activateGroup?(range)
            return
        }

        if isInCheckboxColumn(point),
           let entryIndex = entryIndex(at: point),
           let entry = entries[safe: entryIndex],
           entry.style.isChanged
        {
            toggleLineCheckbox?(entry.lineNumber)
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

    override public func mouseDragged(with event: NSEvent) {
        guard isDraggingLineSelection,
              let anchor = selectionAnchorLine,
              let lineNumber = lineNumber(for: event)
        else {
            super.mouseDragged(with: event)
            return
        }

        selectLines(min(anchor, lineNumber) ... max(anchor, lineNumber))
    }

    override public func mouseUp(with event: NSEvent) {
        isDraggingLineSelection = false
        super.mouseUp(with: event)
    }

    override public func menu(for event: NSEvent) -> NSMenu? {
        let point = convert(event.locationInWindow, from: nil)
        if isInGroupColumn(point),
           let entryIndex = entryIndex(at: point),
           let entry = entries[safe: entryIndex],
           entry.style.isChanged
        {
            let lineNumber = entry.lineNumber
            let range = groupRangeProvider?(lineNumber) ?? lineNumber ... lineNumber
            selectionAnchorLine = range.lowerBound
            selectLines(range)
        } else if let lineNumber = lineNumber(for: event) {
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

    private func entryIndex(for event: NSEvent) -> Int? {
        let point = convert(event.locationInWindow, from: nil)
        return entryIndex(at: point)
    }

    private func lineNumber(for event: NSEvent) -> Int? {
        guard let entryIndex = entryIndex(for: event),
              let entry = entries[safe: entryIndex]
        else { return nil }
        return entry.lineNumber
    }

    private func isInGroupColumn(_ point: NSPoint) -> Bool {
        guard groupHitWidth > 0 else { return false }
        let x = point.x - textContainerInset.width
        return x >= 0 && x <= groupHitWidth
    }

    private func isInCheckboxColumn(_ point: NSPoint) -> Bool {
        guard checkboxHitWidth > 0 else { return false }
        let x = point.x - textContainerInset.width
        return x >= checkboxHitStart && x <= checkboxHitStart + checkboxHitWidth
    }

    private func entryIndex(at point: NSPoint) -> Int? {
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
        guard let lower = entries.first(where: { $0.lineNumber == range.lowerBound }),
              let upper = entries.last(where: { $0.lineNumber == range.upperBound })
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
        let selectedEntries = entries.filter { NSIntersectionRange($0.range, selected).length > 0 }
        guard let lower = selectedEntries.map(\.lineNumber).min(),
              let upper = selectedEntries.map(\.lineNumber).max()
        else { return nil }
        return lower ... upper
    }

    private var currentSelection: DiffGutterSelection? {
        guard let lineRange = selectedLineRange else { return nil }
        let changedLines = entries.reduce(into: Set<Int>()) { lines, entry in
            if lineRange.contains(entry.lineNumber),
               entry.style == .added || entry.style == .removed
            {
                lines.insert(entry.lineNumber)
            }
        }
        return DiffGutterSelection(lineRange: lineRange, changedLineCount: changedLines.count)
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
