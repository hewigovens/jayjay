import AppKit

extension DiffTextView {
    func findAction(_ sender: Any?) -> NSFindPanelAction? {
        if let item = sender as? NSMenuItem {
            return NSFindPanelAction(rawValue: UInt(item.tag))
        }
        if let control = sender as? NSControl {
            return NSFindPanelAction(rawValue: UInt(control.tag))
        }
        if let cell = sender as? NSCell {
            return NSFindPanelAction(rawValue: UInt(cell.tag))
        }
        return nil
    }

    func syncFindSelectionState(scrollToCurrent: Bool) {
        let hasFindSelection = currentFindSelectionRange() != nil
        hasCurrentFindSelection = showsFindHighlights && hasFindSelection
        invalidateFindHighlights()
        if scrollToCurrent, hasFindSelection {
            scrollCurrentFindSelectionToVisible()
        }
    }

    @discardableResult
    func syncActiveFindQueryFromPasteboard(propagate: Bool) -> Bool {
        let query = currentFindPasteboardString()
        guard activeFindQuery != query else {
            syncFindSelectionState(scrollToCurrent: false)
            return false
        }
        activeFindQuery = query
        if propagate {
            findPartner?.receiveFindQuery(query, active: showsFindHighlights)
        }
        return true
    }

    func receiveFindQuery(_ query: String?, active: Bool) {
        showsFindHighlights = active
        activeFindQuery = query
        if active {
            startFindPasteboardMonitoring()
        }
        invalidateFindHighlights()
    }

    func activateFindSession(propagate: Bool) {
        showsFindHighlights = true
        hasObservedFindBarVisible = hasObservedFindBarVisible || enclosingScrollView?.isFindBarVisible == true
        startFindPasteboardMonitoring()
        syncActiveFindQueryFromPasteboard(propagate: propagate)
        if propagate {
            findPartner?.receiveFindQuery(activeFindQuery, active: true)
        }
    }

    func endFindSession(propagate: Bool) {
        pendingFindSelection?.cancel()
        pendingFindSelection = nil
        showsFindHighlights = false
        if propagate {
            findPartner?.endFindSession(propagate: false)
        }
    }

    func startFindPasteboardMonitoring() {
        guard findPasteboardTimer == nil else { return }
        findPasteboardChangeCount = NSPasteboard(name: .find).changeCount
        let timer = Timer(timeInterval: 0.1, repeats: true) { [weak self] _ in
            self?.pollFindPasteboard()
        }
        RunLoop.main.add(timer, forMode: .common)
        findPasteboardTimer = timer
    }

    func stopFindPasteboardMonitoring() {
        findPasteboardTimer?.invalidate()
        findPasteboardTimer = nil
    }

    func pollFindPasteboard() {
        guard showsFindHighlights else {
            stopFindPasteboardMonitoring()
            return
        }
        guard syncFindBarVisibility() else { return }
        let pasteboard = NSPasteboard(name: .find)
        let queryChanged = pasteboard.changeCount != findPasteboardChangeCount ||
            currentFindPasteboardString() != activeFindQuery
        guard queryChanged else { return }

        findPasteboardChangeCount = pasteboard.changeCount
        if syncActiveFindQueryFromPasteboard(propagate: true) {
            scheduleDebouncedFindSelection()
        }
    }

    @discardableResult
    func syncFindBarVisibility() -> Bool {
        guard showsFindHighlights else { return false }
        guard let scrollView = enclosingScrollView else { return true }

        if scrollView.isFindBarVisible {
            hasObservedFindBarVisible = true
            return true
        }

        if hasObservedFindBarVisible {
            endFindSession(propagate: true)
            return false
        }
        return true
    }

    func scheduleDebouncedFindSelection() {
        pendingFindSelection?.cancel()
        guard showsFindHighlights, activeFindQuery != nil else { return }

        let work = DispatchWorkItem { [weak self] in
            self?.selectFindMatchAfterTyping()
        }
        pendingFindSelection = work
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.18, execute: work)
    }

    func selectFindMatchAfterTyping() {
        pendingFindSelection = nil
        guard showsFindHighlights, activeFindQuery != nil else { return }
        if currentFindSelectionRange() != nil {
            scrollCurrentFindSelectionToVisible()
            return
        }
        _ = selectFindMatch(direction: .forward, includePartner: true)
    }

    @discardableResult
    func selectFindMatch(direction: DiffFindDirection, includePartner: Bool) -> Bool {
        syncActiveFindQueryFromPasteboard(propagate: true)
        guard let query = activeFindQuery else { return false }

        if let range = findRange(query, direction: direction, wrapping: false) {
            selectFindRange(range, scroll: true)
            return true
        }

        if includePartner,
           let partner = findPartner,
           let range = partner.edgeFindRange(query, direction: direction)
        {
            partner.receiveFindQuery(query, active: true)
            partner.selectFindRange(range, scroll: true)
            window?.makeFirstResponder(partner)
            return true
        }

        if let range = findRange(query, direction: direction, wrapping: true) {
            selectFindRange(range, scroll: true)
            return true
        }
        return false
    }

    func selectFindRange(_ range: NSRange, scroll: Bool) {
        setSelectedRange(range)
        hasCurrentFindSelection = true
        if scroll {
            scrollCurrentFindSelectionToVisible()
        }
    }

    func findRange(_ query: String, direction: DiffFindDirection, wrapping: Bool) -> NSRange? {
        let text = string as NSString
        let queryLength = (query as NSString).length
        guard queryLength > 0, text.length >= queryLength else { return nil }

        let selection = selectedRanges.first?.rangeValue ?? NSRange(location: 0, length: 0)
        let currentSelection = currentFindSelectionRange()
        switch direction {
            case .forward:
                let start = currentSelection.map(NSMaxRange) ?? selection.location
                if let range = text.firstRange(of: query, from: start) {
                    return range
                }
                return wrapping ? text.firstRange(of: query, from: 0, to: start) : nil
            case .backward:
                let end = currentSelection?.location ?? selection.location
                if let range = text.lastRange(of: query, upTo: end) {
                    return range
                }
                return wrapping ? text.lastRange(of: query, from: end) : nil
        }
    }

    func edgeFindRange(_ query: String, direction: DiffFindDirection) -> NSRange? {
        let text = string as NSString
        switch direction {
            case .forward:
                return text.firstRange(of: query, from: 0)
            case .backward:
                return text.lastRange(of: query, upTo: text.length)
        }
    }

    func scrollCurrentFindSelectionToVisible() {
        guard let range = currentFindSelectionRange() else { return }

        guard let layoutManager, let textContainer else {
            scrollRangeToVisible(range)
            return
        }

        layoutManager.ensureLayout(forCharacterRange: range)
        scrollRangeToVisible(range)

        let glyphRange = layoutManager.glyphRange(forCharacterRange: range, actualCharacterRange: nil)
        guard glyphRange.length > 0 else { return }

        let matchRect = layoutManager
            .boundingRect(forGlyphRange: glyphRange, in: textContainer)
            .offsetBy(dx: textContainerOrigin.x, dy: textContainerOrigin.y)
        guard let scrollView = enclosingScrollView else { return }

        let clipView = scrollView.contentView
        let visibleRect = clipView.bounds
        let margin = min(CGFloat(24), visibleRect.height / 3)
        if visibleRect.insetBy(dx: 0, dy: margin).contains(matchRect) {
            return
        }

        let maxY = max(CGFloat(0), bounds.height - visibleRect.height)
        let targetY = min(max(CGFloat(0), matchRect.midY - visibleRect.height / 2), maxY)
        clipView.scroll(to: NSPoint(x: visibleRect.origin.x, y: targetY))
        scrollView.reflectScrolledClipView(clipView)
    }

    func currentFindSelectionRange() -> NSRange? {
        guard let findString = activeFindQuery ?? currentFindPasteboardString() else { return nil }
        let findLength = (findString as NSString).length
        guard findLength > 0 else { return nil }

        let text = string as NSString
        for value in selectedRanges {
            let range = value.rangeValue
            guard range.location != NSNotFound,
                  range.length == findLength,
                  NSMaxRange(range) <= text.length,
                  text.compare(findString, options: [.caseInsensitive], range: range) == .orderedSame
            else { continue }
            return range
        }
        return nil
    }

    func invalidateFindHighlights() {
        let range = NSRange(location: 0, length: (string as NSString).length)
        layoutManager?.invalidateDisplay(forCharacterRange: range)
        needsDisplay = true
    }

    func applySelectionAttributes() {
        selectedTextAttributes = hasCurrentFindSelection ? findSelectionAttributes : defaultSelectionAttributes
    }
}

enum DiffFindDirection {
    case forward
    case backward
}

extension NSFindPanelAction {
    var usesDiffFindHighlights: Bool {
        switch self {
            case .showFindPanel, .next, .previous, .setFindString, .selectAll, .selectAllInSelection:
                true
            default:
                false
        }
    }
}

private func currentFindPasteboardString() -> String? {
    guard let findString = NSPasteboard(name: .find).string(forType: .string) else {
        return nil
    }
    return findString.isEmpty ? nil : findString
}
