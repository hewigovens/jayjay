import JayJayCore
import SwiftUI

@MainActor
struct DAGViewModel {
    nonisolated private static let downArrowKeyCode: UInt16 = 125
    nonisolated private static let upArrowKeyCode: UInt16 = 126

    let entries: [GraphEntry]
    let selectedId: String?
    let selectedIds: [String]
    let compareFromId: String?
    let contextTargetId: String?
    let rebaseDrag: DAGRebaseDragState?
    let bookmarkDrag: BookmarkDragState?
    let colorScheme: ColorScheme
    let layout: DAGLayout
    private let cache = Cache()

    var isEmpty: Bool {
        entries.isEmpty
    }

    var hasMultipleSelection: Bool {
        selectedIds.count > 1
    }

    var selectedRevisions: [String] {
        selectedChanges.map(\.selectionRevision)
    }

    var canAbandonSelection: Bool {
        hasMutableSelection
    }

    var canDiffSelection: Bool {
        isContiguousLinearSelection && Self.rangeHasSingleParentBase(selectedChanges)
    }

    var canSquashSelection: Bool {
        hasMutableSelection && isContiguousLinearSelection
    }

    private var isContiguousLinearSelection: Bool {
        let selectedEntries = entries.enumerated().filter { isSelected($0.element.change) }
        guard let first = selectedEntries.first?.offset,
              let last = selectedEntries.last?.offset,
              selectedEntries.count == last - first + 1
        else {
            return false
        }
        return Self.formsConsecutiveLinearRange(selectedChanges)
    }

    func canRebaseSelection(onto target: ChangeInfo) -> Bool {
        guard hasMutableSelection, !isSelected(target) else { return false }
        return !descendantCommitIds.contains(target.commitId.id)
    }

    var canMergeSelection: Bool {
        selectedCommitIds.count > 1 && selectedCommitIds.isDisjoint(with: ancestorCommitIds)
    }

    func canMergeSelectedChange(with target: ChangeInfo) -> Bool {
        guard !selectedCommitIds.isEmpty else { return false }
        if selectedCommitIds.contains(target.commitId.id) {
            return canMergeSelection
        }
        return selectedCommitIds.isDisjoint(with: ancestorCommitIds)
            && !ancestorCommitIds.contains(target.commitId.id)
            && !descendantCommitIds.contains(target.commitId.id)
    }

    nonisolated static func formsConsecutiveLinearRange(_ changes: [ChangeInfo]) -> Bool {
        changes.count > 1 && zip(changes, changes.dropFirst()).allSatisfy { newer, older in
            newer.parents == [older.commitId.id]
        }
    }

    /// The combined diff bases on the oldest change's single parent; squashing the same range into a merge commit is still legal.
    nonisolated static func rangeHasSingleParentBase(_ changes: [ChangeInfo]) -> Bool {
        changes.last?.parents.count == 1
    }

    private var selectedChanges: [ChangeInfo] {
        if let changes = cache.selectedChanges {
            return changes
        }
        let changes = entries.compactMap { isSelected($0.change) ? $0.change : nil }
        cache.selectedChanges = changes
        return changes
    }

    private var selectedCommitIds: Set<String> {
        Set(selectedChanges.map(\.commitId.id))
    }

    private var hasMutableSelection: Bool {
        selectedChanges.count == selectedIds.count
            && selectedChanges.count > 1
            && selectedChanges.allSatisfy { !$0.isImmutable }
    }

    private var parentIdsByCommitId: [String: [String]] {
        if let parents = cache.parentIdsByCommitId {
            return parents
        }
        let parents = Dictionary(
            uniqueKeysWithValues: entries.map { entry in
                (
                    entry.change.commitId.id,
                    entry.edges.filter { $0.edgeType != .missing }.map(\.target)
                )
            }
        )
        cache.parentIdsByCommitId = parents
        return parents
    }

    private var ancestorCommitIds: Set<String> {
        if let ancestors = cache.ancestorCommitIds {
            return ancestors
        }
        let ancestors = reachableCommitIds(links: parentIdsByCommitId)
        cache.ancestorCommitIds = ancestors
        return ancestors
    }

    private var descendantCommitIds: Set<String> {
        if let descendants = cache.descendantCommitIds {
            return descendants
        }
        var children: [String: [String]] = [:]
        for (commitId, parents) in parentIdsByCommitId {
            for parent in parents {
                children[parent, default: []].append(commitId)
            }
        }
        let descendants = reachableCommitIds(links: children)
        cache.descendantCommitIds = descendants
        return descendants
    }

    private func reachableCommitIds(links: [String: [String]]) -> Set<String> {
        var pending = selectedCommitIds.flatMap { links[$0, default: []] }
        var visited: Set<String> = []
        while let commitId = pending.popLast() {
            if visited.insert(commitId).inserted {
                pending.append(contentsOf: links[commitId, default: []])
            }
        }
        return visited
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
            selectedIds: selectedIds,
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
        if hovering, !isSelected(entry.change) {
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

    func isSelected(_ change: ChangeInfo) -> Bool {
        let revision = change.selectionRevision
        return selectedIds.contains(revision) || (selectedIds.isEmpty && selectedId == revision)
    }

    func selectedRevision(for changeId: String) -> String {
        change(for: changeId)?.selectionRevision ?? changeId
    }

    func change(for changeId: String) -> ChangeInfo? {
        if cache.changesByRevision == nil {
            var changes: [String: ChangeInfo] = [:]
            for entry in entries {
                for revision in [entry.change.changeId.id, entry.change.commitId.id] where changes[revision] == nil {
                    changes[revision] = entry.change
                }
            }
            cache.changesByRevision = changes
        }
        return cache.changesByRevision?[changeId]
    }

    func canSquashIntoParent(_ target: ChangeInfo) -> Bool {
        guard let parentId = target.parents.first else { return false }
        return change(for: parentId).map { !$0.isImmutable } ?? true
    }

    func bookmarkDiffRequest(from selectedId: String, to target: ChangeInfo) -> BookmarkDiffRequest? {
        guard let selectedChange = change(for: selectedId),
              let base = RevsetExpressions.primaryBaseBookmarkEndpoint(for: selectedChange),
              let head = RevsetExpressions.primaryHeadBookmarkEndpoint(for: target),
              base.label != head.label
        else {
            return nil
        }
        return BookmarkDiffRequest(base: base, head: head)
    }

    func scrollId(for rev: String) -> String {
        change(for: rev)?.selectionRevision ?? rev
    }

    /// Other visible commits that share this change's id — the siblings of a divergent change. Empty unless `change` is divergent. Used to offer an interdiff between two versions of the same change so the user can see which is safer to abandon.
    func divergentSiblings(of change: ChangeInfo) -> [ChangeInfo] {
        guard change.isDivergent else { return [] }
        return entries
            .map(\.change)
            .filter { $0.changeId.id == change.changeId.id && $0.commitId.id != change.commitId.id }
    }

    nonisolated static func selectionDelta(
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

    /// All inputs are immutable, so row menus can share derived data for this view-model snapshot.
    private final class Cache {
        var selectedChanges: [ChangeInfo]?
        var parentIdsByCommitId: [String: [String]]?
        var changesByRevision: [String: ChangeInfo]?
        var ancestorCommitIds: Set<String>?
        var descendantCommitIds: Set<String>?
    }
}
