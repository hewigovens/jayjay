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
    private(set) var selection = OrderedSelection<Int>()
    var interdiffDetail: ChangeDetail?
    var interdiffLoading = false
    var selectedPath: String?
    var selectedHunk: DiffHunk?

    var headCommitId: String? {
        entries.first?.commitId.id
    }

    var displayedRows: [EvologRow] {
        evologRows(entries: entries, hideSnapshots: hideSnapshots, expandedRuns: Array(expandedSnapshotRuns))
    }

    var selectedIndex: Int? {
        selection.primaryID
    }

    var selectedFromCommitId: String? {
        guard let index = orderedSelectedIndices.last else { return nil }
        return entries[index].commitId.id
    }

    var selectedToCommitId: String? {
        guard let index = orderedSelectedIndices.first else { return nil }
        return selection.count > 1 ? entries[index].commitId.id : headCommitId
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
        let retarget: (Int?) -> Int? = { index in
            guard let index else { return nil }
            return rows.first(where: { $0.range.contains(index) })?.actionIndex
        }
        selection = OrderedSelection(
            selectedIDs: Set(previousSelection.selectedIDs.compactMap { retarget($0) }),
            primaryID: retarget(previousSelection.primaryID),
            anchorID: retarget(previousSelection.anchorID)
        )
        if previousFrom != selectedFromCommitId || previousTo != selectedToCommitId {
            loadInterdiff()
        }
    }

    func select(_ row: EvologRow, click: OrderedSelectionClick) {
        let orderedIndices = displayedRows.map(\.actionIndex)
        selection.apply(click, to: row.actionIndex, orderedIDs: orderedIndices)
        if row.isCollapsedRun {
            expandedSnapshotRuns.insert(row.start)
        }
        loadInterdiff()
    }

    private var orderedSelectedIndices: [Int] {
        selection.orderedIDs(in: Array(entries.indices))
    }

    private func loadInterdiff() {
        selectedHunk = nil
        selectedPath = nil
        interdiffDetail = nil
        interdiffLoading = false
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
            let detail = try? repo.interdiffSummary(fromRev: from, toRev: to)
            await MainActor.run { [weak self] in
                guard let self,
                      selectedFromCommitId == from,
                      selectedToCommitId == to
                else { return }
                interdiffLoading = false
                interdiffDetail = detail
                if let firstPath = detail?.diff.first?.path {
                    selectedPath = firstPath
                    // file-list `onChange` doesn't fire until that view mounts; trigger the load here.
                    loadFile(path: firstPath)
                }
            }
        }
    }

    func loadFile(path: String?) {
        selectedHunk = nil
        guard let path, let repo,
              let from = selectedFromCommitId, let to = selectedToCommitId
        else { return }
        Task.detached { [weak self] in
            let hunk = try? repo.interdiffFile(fromRev: from, toRev: to, path: path)
            await MainActor.run { [weak self] in
                guard let self,
                      selectedPath == path,
                      selectedFromCommitId == from,
                      selectedToCommitId == to
                else { return }
                selectedHunk = hunk
            }
        }
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
