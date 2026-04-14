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
    let colorScheme: ColorScheme
    let layout: DAGLayout

    init(
        entries: [GraphEntry],
        selectedId: String?,
        compareFromId: String?,
        contextTargetId: String?,
        rebaseDrag: DAGRebaseDragState?,
        colorScheme: ColorScheme,
        layout: DAGLayout
    ) {
        self.entries = entries
        self.selectedId = selectedId
        self.compareFromId = compareFromId
        self.contextTargetId = contextTargetId
        self.rebaseDrag = rebaseDrag
        self.colorScheme = colorScheme
        self.layout = layout
    }

    var isEmpty: Bool {
        entries.isEmpty
    }

    func rowViewModel(for entry: GraphEntry, index: Int, previewText: String?) -> DAGRowViewModel {
        DAGRowViewModel(
            entry: entry,
            layout: layout,
            index: index,
            selectedId: selectedId,
            compareFromId: compareFromId,
            contextTargetId: contextTargetId,
            rebaseDrag: rebaseDrag,
            rebasePreviewText: previewText,
            colorScheme: colorScheme
        )
    }

    func nextContextTargetId(hovering: Bool, entry: GraphEntry) -> String? {
        if hovering, let selectedId, selectedId != entry.change.changeId {
            return entry.change.changeId
        }
        if !hovering, contextTargetId == entry.change.changeId {
            return nil
        }
        return contextTargetId
    }

    func shouldCancelRebaseDrag(for hoveredCommitId: String?) -> Bool {
        guard let hoveredCommitId else { return false }
        return !entries.contains(where: { $0.change.commitId == hoveredCommitId })
    }

    func selectedChangeId(afterMovingBy delta: Int) -> String? {
        guard !entries.isEmpty else { return nil }
        let currentIdx: Int
        if let selectedId,
           let idx = entries.firstIndex(where: { $0.change.changeId == selectedId })
        {
            currentIdx = idx
        } else {
            currentIdx = delta > 0 ? -1 : entries.count
        }
        let newIdx = max(0, min(entries.count - 1, currentIdx + delta))
        guard newIdx != currentIdx else { return nil }
        return entries[newIdx].change.changeId
    }

    func selectedRevision(for changeId: String) -> String {
        guard let selectedEntry = entries.first(where: { $0.change.changeId == changeId }) else {
            return changeId
        }
        return selectedEntry.change.isDivergent ? selectedEntry.change.commitId : changeId
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
