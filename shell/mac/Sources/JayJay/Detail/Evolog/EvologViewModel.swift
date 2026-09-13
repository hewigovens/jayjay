import AppKit
import JayJayCore
import SwiftUI

@Observable
final class EvologViewModel {
    let entries: [EvologEntry]
    let changeId: String
    let repo: JayJayRepo?
    let diffStore: DiffStore

    private(set) var hideSnapshots = true
    var expandedSnapshotRuns: Set<UInt32> = []
    private(set) var selection = OrderedSelection(selected: [], primary: nil, anchor: nil)
    var interdiffDetail: ChangeDetail?
    var interdiffLoading = false
    var interdiffError: String?
    var selectedPath: String?
    var selectedHunk: DiffHunk?
    var fileLoading = false
    var fileError: String?
    private(set) var comparisonReversed = false

    var headCommitId: String? {
        entries.first?.commitId.id
    }

    var displayedRows: [EvologRow] {
        evologRows(entries: entries, hideSnapshots: hideSnapshots, expandedRuns: Array(expandedSnapshotRuns))
    }

    var selectedIndex: Int? {
        selection.primary.flatMap(Int.init)
    }

    func isSelected(_ row: EvologRow) -> Bool {
        selection.selected.contains(String(row.actionIndex))
    }

    var selectedFromCommitId: String? {
        selectedEndpoints?.from
    }

    var selectedToCommitId: String? {
        selectedEndpoints?.to
    }

    var canReverseComparison: Bool {
        guard let endpoints = chronologicalEndpoints else { return false }
        return endpoints.from != endpoints.to
    }

    init(entries: [EvologEntry], changeId: String, repo: JayJayRepo?, diffStore: DiffStore) {
        self.entries = entries
        self.changeId = changeId
        self.repo = repo
        self.diffStore = diffStore
    }

    func setHideSnapshots(_ hide: Bool) {
        guard hideSnapshots != hide else { return }
        hideSnapshots = hide
        guard hide else { return }

        let previousFrom = selectedFromCommitId
        let previousTo = selectedToCommitId
        let previousSelection = selection
        expandedSnapshotRuns.removeAll()
        let rows = displayedRows
        let retarget: (String?) -> String? = { id in
            guard let index = id.flatMap(Int.init) else { return nil }
            return rows.first(where: { $0.range.contains(index) }).map { String($0.actionIndex) }
        }
        selection = orderedSelection(
            selected: previousSelection.selected.compactMap { retarget($0) },
            primary: retarget(previousSelection.primary),
            anchor: retarget(previousSelection.anchor)
        )
        if previousFrom != selectedFromCommitId || previousTo != selectedToCommitId {
            loadInterdiff()
        }
    }

    func select(_ row: EvologRow, click: SelectionClick) {
        selection = applyPairSelectionClick(
            selection: selection,
            click: click,
            id: String(row.actionIndex),
            order: displayedRows.map { String($0.actionIndex) }
        )
        comparisonReversed = false
        if row.isCollapsedRun {
            expandedSnapshotRuns.insert(row.start)
        }
        loadInterdiff()
    }

    func reverseComparison() {
        guard canReverseComparison else { return }
        comparisonReversed.toggle()
        loadInterdiff()
    }

    private var orderedSelectedIndices: [Int] {
        orderedSelectionIds(selection: selection, order: entries.indices.map(String.init))
            .compactMap(Int.init)
    }

    private var chronologicalEndpoints: (from: String, to: String)? {
        guard let newest = orderedSelectedIndices.first,
              let oldest = orderedSelectedIndices.last
        else { return nil }
        return (
            entries[oldest].commitId.id,
            selection.selected.count > 1 ? entries[newest].commitId.id : headCommitId ?? entries[newest].commitId.id
        )
    }

    private var selectedEndpoints: (from: String, to: String)? {
        guard let endpoints = chronologicalEndpoints else { return nil }
        return comparisonReversed ? (from: endpoints.to, to: endpoints.from) : endpoints
    }

    private func loadInterdiff() {
        selectedHunk = nil
        selectedPath = nil
        interdiffDetail = nil
        interdiffLoading = false
        interdiffError = nil
        fileLoading = false
        fileError = nil
        guard let repo,
              let from = selectedFromCommitId,
              let to = selectedToCommitId
        else { return }
        if from == to {
            guard let index = orderedSelectedIndices.first else { return }
            interdiffDetail = ChangeDetail(info: entries[index].asPlaceholderInfo(), diff: [])
            return
        }
        interdiffLoading = true
        Task.detached { [weak self] in
            let result: (detail: ChangeDetail?, error: String?)
            do {
                result = try (repo.interdiffSummary(fromRev: from, toRev: to), nil)
            } catch {
                result = (nil, error.friendlyDescription)
            }
            await MainActor.run { [weak self] in
                guard let self,
                      selectedFromCommitId == from,
                      selectedToCommitId == to
                else { return }
                interdiffLoading = false
                interdiffDetail = result.detail
                interdiffError = result.error
                if let firstPath = result.detail?.diff.first?.path {
                    selectedPath = firstPath
                    // file-list `onChange` doesn't fire until that view mounts; trigger the load here.
                    loadFile(path: firstPath)
                }
            }
        }
    }

    func loadFile(path: String?) {
        selectedHunk = nil
        fileLoading = false
        fileError = nil
        guard let path, let repo,
              let from = selectedFromCommitId, let to = selectedToCommitId
        else { return }
        fileLoading = true
        Task.detached { [weak self] in
            let result: (hunk: DiffHunk?, error: String?)
            do {
                result = try (repo.interdiffFile(fromRev: from, toRev: to, path: path), nil)
            } catch {
                result = (nil, error.friendlyDescription)
            }
            await MainActor.run { [weak self] in
                guard let self,
                      selectedPath == path,
                      selectedFromCommitId == from,
                      selectedToCommitId == to
                else { return }
                fileLoading = false
                selectedHunk = result.hunk
                fileError = result.error
            }
        }
    }

    func retryInterdiff() {
        loadInterdiff()
    }

    func copyCommitId(_ commitId: String) {
        copyToPasteboard(commitId)
    }

    func copyRestoreCommand(_ commitId: String) {
        copyToPasteboard("jj restore --from \(commitId) --into @")
    }

    private func copyToPasteboard(_ value: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(value, forType: .string)
    }
}

extension EvologRow {
    var isCollapsedRun: Bool {
        count > 1
    }

    var range: Range<Int> {
        Int(start) ..< Int(start + count)
    }

    var actionIndex: Int {
        Int(start)
    }
}
