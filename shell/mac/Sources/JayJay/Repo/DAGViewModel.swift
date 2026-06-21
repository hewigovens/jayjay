import JayJayCore
import SwiftUI

struct DAGViewModel {
    private static let downArrowKeyCode: UInt16 = 125
    private static let upArrowKeyCode: UInt16 = 126

    let entries: [GraphEntry]
    let selectedId: String?
    let compareFromId: String?
    let contextTargetId: String?
    let rebaseDrag: DAGRebaseDragState?
    let bookmarkDrag: BookmarkDragState?
    let colorScheme: ColorScheme
    let layout: DAGLayout

    var isEmpty: Bool {
        entries.isEmpty
    }

    func rowViewModel(
        for entry: GraphEntry,
        index: Int,
        rebasePreviewText: String?,
        bookmarkPreviewText: String?
    ) -> DAGRowViewModel {
        DAGRowViewModel(
            entry: entry,
            layout: layout,
            index: index,
            selectedId: selectedId,
            compareFromId: compareFromId,
            contextTargetId: contextTargetId,
            rebaseDrag: rebaseDrag,
            rebasePreviewText: rebasePreviewText,
            bookmarkDrag: bookmarkDrag,
            bookmarkPreviewText: bookmarkPreviewText,
            colorScheme: colorScheme
        )
    }

    func nextContextTargetId(hovering: Bool, entry: GraphEntry) -> String? {
        let rowId = entry.change.selectionRevision
        if hovering, let selectedId, selectedId != rowId {
            return rowId
        }
        if !hovering, contextTargetId == rowId {
            return nil
        }
        return contextTargetId
    }

    func shouldCancelRebaseDrag(for hoveredCommitId: String?) -> Bool {
        guard let hoveredCommitId else { return false }
        return !entries.contains(where: { $0.change.commitId.id == hoveredCommitId })
    }

    func selectedChangeId(afterMovingBy delta: Int) -> String? {
        guard !entries.isEmpty else { return nil }
        let currentIdx: Int = if let selectedId,
                                 let idx = entries.firstIndex(where: { $0.change.selectionRevision == selectedId })
        {
            idx
        } else {
            delta > 0 ? -1 : entries.count
        }
        let newIdx = max(0, min(entries.count - 1, currentIdx + delta))
        guard newIdx != currentIdx else { return nil }
        return entries[newIdx].change.selectionRevision
    }

    func selectedRevision(for changeId: String) -> String {
        guard let selectedEntry = entries.first(where: { $0.change.matchesRevision(changeId) }) else {
            return changeId
        }
        return selectedEntry.change.selectionRevision
    }

    func bookmarkDiffRequest(from selectedId: String, to target: ChangeInfo) -> BookmarkDiffRequest? {
        guard let selectedEntry = entries.first(where: { $0.change.matchesRevision(selectedId) }),
              let base = RevsetExpressions.primaryBaseBookmarkEndpoint(for: selectedEntry.change),
              let head = RevsetExpressions.primaryHeadBookmarkEndpoint(for: target),
              base.label != head.label
        else {
            return nil
        }
        return BookmarkDiffRequest(base: base, head: head)
    }

    func scrollId(for rev: String) -> String {
        entries.first(where: { $0.change.matchesRevision(rev) })?.change.selectionRevision ?? rev
    }

    static func selectionDelta(
        keyCode: UInt16,
        charactersIgnoringModifiers: String?,
        controlPressed: Bool
    ) -> Int? {
        switch keyCode {
            case downArrowKeyCode:
                return 1
            case upArrowKeyCode:
                return -1
            default:
                break
        }

        switch charactersIgnoringModifiers {
            case "j":
                return 1
            case "k":
                return -1
            case "n" where controlPressed:
                return 1
            case "p" where controlPressed:
                return -1
            default:
                return nil
        }
    }
}
