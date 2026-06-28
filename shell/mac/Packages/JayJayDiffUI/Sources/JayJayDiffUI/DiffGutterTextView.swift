import AppKit
import JayJayCore

struct DiffGutterSelection {
    let lineRange: ClosedRange<Int>
    let menuLineNumber: Int
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

    var entries: [Entry] = [] {
        // Hover fires per mouse move; precompute the row lookup so it never scans all (wrapped) rows. Embedded note rows (negative ids) stay out so they never hover or select.
        didSet {
            rowsByLineNumber = Dictionary(
                grouping: entries.indices.filter { entries[$0].lineNumber > 0 }
            ) { entries[$0].lineNumber }
        }
    }

    private var rowsByLineNumber: [Int: [Int]] = [:]
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
    var groupIndexAtLineNumber: [Int: UInt32] = [:]
    var toggleReviewCheckbox: ((UInt32) -> Void)?
    var notedLines: Set<Int> = []
    var onNoteClicked: ((Int, NSRect) -> Void)?
    var noteHitStart: CGFloat = 0
    var noteHitWidth: CGFloat = 0
    var activeNotePopover: NSPopover?
    var externalSelection: ClosedRange<Int>? {
        didSet { applyExternalSelection() }
    }

    override public func setFrameSize(_ newSize: NSSize) {
        pinningClipOrigin { super.setFrameSize(newSize) }
    }

    private var pendingMenuLineNumber: Int?
    private var hoveredLineNumber: Int? {
        didSet {
            guard hoveredLineNumber != oldValue else { return }
            applyHover()
        }
    }

    override public func updateTrackingAreas() {
        super.updateTrackingAreas()
        for area in trackingAreas where area.owner === self {
            removeTrackingArea(area)
        }
        addTrackingArea(NSTrackingArea(
            rect: .zero,
            options: [.mouseMoved, .mouseEnteredAndExited, .activeInKeyWindow, .inVisibleRect],
            owner: self,
            userInfo: nil
        ))
    }

    override public func mouseMoved(with event: NSEvent) {
        hoveredLineNumber = hoverLineNumber(at: convert(event.locationInWindow, from: nil))
    }

    override public func mouseExited(with event: NSEvent) {
        hoveredLineNumber = nil
        super.mouseExited(with: event)
    }

    private func hoverLineNumber(at point: NSPoint) -> Int? {
        guard let layoutManager, let textContainer else { return nil }
        let containerPoint = NSPoint(
            x: point.x - textContainerInset.width,
            y: point.y - textContainerInset.height
        )
        let glyphIndex = layoutManager.glyphIndex(for: containerPoint, in: textContainer)
        // glyphIndex clamps to the last line; reject points below the laid-out text so empty space doesn't hover the last row.
        let fragmentRect = layoutManager.lineFragmentRect(forGlyphAt: glyphIndex, effectiveRange: nil)
        guard containerPoint.y <= fragmentRect.maxY else { return nil }
        let charIndex = layoutManager.characterIndexForGlyph(at: glyphIndex)
        return entry(atCharIndex: charIndex)?.lineNumber
    }

    /// Binary search over `entries` (built in ascending range order).
    private func entry(atCharIndex charIndex: Int) -> Entry? {
        var low = 0
        var high = entries.count - 1
        while low <= high {
            let mid = (low + high) / 2
            let candidate = entries[mid]
            if charIndex < candidate.range.location {
                high = mid - 1
            } else if charIndex >= NSMaxRange(candidate.range) {
                low = mid + 1
            } else {
                return candidate
            }
        }
        return nil
    }

    private func applyHover() {
        guard let manager = layoutManager as? DiffLayoutManager else { return }
        manager.hoveredRowIndices = hoveredLineNumber
            .flatMap { rowsByLineNumber[$0] }
            .map(Set.init) ?? []
        needsDisplay = true
    }

    override public func mouseDown(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        if isInGroupColumn(point),
           let entryIndex = entryIndex(at: point),
           let entry = entries[safe: entryIndex],
           entry.style.isChanged
        {
            let lineNumber = entry.lineNumber
            // Review toggle takes priority over the select-change-group click.
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

        if isInNoteColumn(point),
           let onNoteClicked,
           let entryIndex = entryIndex(at: point),
           let entry = entries[safe: entryIndex],
           notedLines.contains(entry.lineNumber)
        {
            onNoteClicked(entry.lineNumber, noteAnchorRect(for: entry))
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
            pendingMenuLineNumber = lineNumber
            selectionAnchorLine = range.lowerBound
            selectLines(range)
        } else if let lineNumber = lineNumber(for: event) {
            pendingMenuLineNumber = lineNumber
            let current = selectedLineRange
            if current == nil || !(current!.contains(lineNumber)) {
                selectionAnchorLine = lineNumber
                selectLines(lineNumber ... lineNumber)
            }
        }

        guard let selection = currentSelection,
              let menuProvider
        else { return nil }
        pendingMenuLineNumber = nil

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
              let entry = entries[safe: entryIndex],
              entry.lineNumber > 0
        else { return nil }
        return entry.lineNumber
    }

    /// Anchor at the gutter's trailing edge for the line's row, so the popover arrow points at the start of the code line instead of the marker dot.
    private func noteAnchorRect(for entry: Entry) -> NSRect {
        guard let layoutManager, let textContainer else { return .zero }
        let glyphRange = layoutManager.glyphRange(forCharacterRange: entry.range, actualCharacterRange: nil)
        var rect = layoutManager.boundingRect(forGlyphRange: glyphRange, in: textContainer)
        rect.origin.y += textContainerInset.height
        rect.size.width = 4
        rect.origin.x = bounds.maxX - rect.width
        return rect
    }

    private func isInNoteColumn(_ point: NSPoint) -> Bool {
        guard noteHitWidth > 0 else { return false }
        let x = point.x - textContainerInset.width
        return x >= noteHitStart && x <= noteHitStart + noteHitWidth
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
        let selectedEntries = entries.filter { $0.lineNumber > 0 && NSIntersectionRange($0.range, selected).length > 0 }
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
        return DiffGutterSelection(
            lineRange: lineRange,
            menuLineNumber: pendingMenuLineNumber ?? lineRange.lowerBound,
            changedLineCount: changedLines.count
        )
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
