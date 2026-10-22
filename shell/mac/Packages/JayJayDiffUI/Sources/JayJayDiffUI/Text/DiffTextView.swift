import AppKit

final class DiffTextView: NSTextView {
    weak var findPartner: DiffTextView?
    var copyRows: [DiffCopyRow] = []

    override func writeSelection(to pboard: NSPasteboard, types: [NSPasteboard.PasteboardType]) -> Bool {
        let written = types.filter { [.string, .rtf, .rtfd].contains($0) }
        guard !copyRows.isEmpty, !written.isEmpty, let storage = textStorage else {
            return super.writeSelection(to: pboard, types: types)
        }
        let text = NSMutableAttributedString()
        for (ix, range) in selectedRanges.enumerated() {
            if ix > 0 {
                text.append(NSAttributedString(string: "\n"))
            }
            text.append(storage.copyText(in: range.rangeValue, rows: copyRows))
        }
        let whole = NSRange(location: 0, length: text.length)
        pboard.declareTypes(written, owner: nil)
        return written.allSatisfy { type in
            switch type {
                case .rtf: text.rtf(from: whole).map { pboard.setData($0, forType: type) } ?? false
                case .rtfd: text.rtfd(from: whole).map { pboard.setData($0, forType: type) } ?? false
                default: pboard.setString(text.string, forType: type)
            }
        }
    }

    override func setFrameSize(_ newSize: NSSize) {
        pinningClipOrigin { super.setFrameSize(newSize) }
    }

    var showsFindHighlights = false {
        didSet {
            if !showsFindHighlights {
                hasCurrentFindSelection = false
                activeFindQuery = nil
                hasObservedFindBarVisible = false
                stopFindPasteboardMonitoring()
            }
            applySelectionAttributes()
            invalidateFindHighlights()
        }
    }

    var activeFindQuery: String? {
        didSet {
            if activeFindQuery != oldValue {
                syncFindSelectionState(scrollToCurrent: false)
            }
        }
    }

    var hasCurrentFindSelection = false {
        didSet {
            applySelectionAttributes()
            invalidateFindHighlights()
        }
    }

    var findPasteboardChangeCount = NSPasteboard(name: .find).changeCount
    var findPasteboardTimer: Timer?
    var pendingFindSelection: DispatchWorkItem?
    var hasObservedFindBarVisible = false
    var findSelectionBackgroundColor = NSColor.selectedTextBackgroundColor
    var findSelectionAttributes: [NSAttributedString.Key: Any] = [
        .backgroundColor: NSColor.selectedTextBackgroundColor,
        .foregroundColor: NSColor.selectedTextColor
    ]
    let defaultSelectionAttributes: [NSAttributedString.Key: Any] = [
        .backgroundColor: NSColor.selectedTextBackgroundColor,
        .foregroundColor: NSColor.selectedTextColor
    ]

    func configureFindSelectionColors(_ theme: DiffColors) {
        findSelectionBackgroundColor = theme.findCurrentMatchBg
        findSelectionAttributes = [
            .backgroundColor: theme.findCurrentMatchBg,
            .foregroundColor: theme.findCurrentMatchText
        ]
        applySelectionAttributes()
    }

    var selectionHighlightBackgroundColor: NSColor {
        hasCurrentFindSelection ? findSelectionBackgroundColor : .selectedTextBackgroundColor
    }

    override func performFindPanelAction(_ sender: Any?) {
        let action = findAction(sender)
        if action == .next || action == .previous {
            activateFindSession(propagate: true)
            super.performFindPanelAction(sender)
            syncActiveFindQueryFromPasteboard(propagate: true)
            syncFindSelectionState(scrollToCurrent: true)
            return
        }

        let shouldTrackFind = action?.usesDiffFindHighlights ?? true
        if shouldTrackFind {
            activateFindSession(propagate: true)
        }
        super.performFindPanelAction(sender)
        if shouldTrackFind {
            syncActiveFindQueryFromPasteboard(propagate: true)
            scheduleDebouncedFindSelection()
            DispatchQueue.main.async { [weak self] in
                self?.syncActiveFindQueryFromPasteboard(propagate: true)
                self?.scheduleDebouncedFindSelection()
            }
        } else {
            invalidateFindHighlights()
        }
    }

    override func cancelOperation(_ sender: Any?) {
        endFindSession(propagate: true)
        super.cancelOperation(sender)
    }

    override func setSelectedRange(_ charRange: NSRange) {
        super.setSelectedRange(charRange)
        if showsFindHighlights {
            syncFindSelectionState(scrollToCurrent: false)
        }
    }

    override func setSelectedRange(
        _ charRange: NSRange,
        affinity: NSSelectionAffinity,
        stillSelecting stillSelectingFlag: Bool
    ) {
        super.setSelectedRange(charRange, affinity: affinity, stillSelecting: stillSelectingFlag)
        if showsFindHighlights, !stillSelectingFlag {
            syncFindSelectionState(scrollToCurrent: false)
        }
    }

    override func setSelectedRanges(
        _ ranges: [NSValue],
        affinity: NSSelectionAffinity,
        stillSelecting stillSelectingFlag: Bool
    ) {
        super.setSelectedRanges(ranges, affinity: affinity, stillSelecting: stillSelectingFlag)
        if showsFindHighlights, !stillSelectingFlag {
            syncFindSelectionState(scrollToCurrent: false)
        }
    }

    deinit {
        pendingFindSelection?.cancel()
        stopFindPasteboardMonitoring()
    }
}

extension NSTextView {
    func applyFindSelectionColors(_ theme: DiffColors) {
        if let diffTextView = self as? DiffTextView {
            diffTextView.configureFindSelectionColors(theme)
        }
    }
}
