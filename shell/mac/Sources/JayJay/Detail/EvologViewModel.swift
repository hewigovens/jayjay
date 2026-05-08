import AppKit
import JayJayCore
import SwiftUI

extension EvologVisibleRow {
    var primary: EvologEntry {
        entries[0]
    }
}

@Observable
final class EvologViewModel {
    let entries: [EvologEntry]
    let changeId: String
    let repo: JayJayRepo?
    let diffStore: DiffStore

    var selectedIndex: Int?
    var interdiffDetail: ChangeDetail?
    var interdiffLoading = false
    var selectedPath: String?
    var selectedHunk: DiffHunk?
    var hideSnapshots = false
    var collapseSnapshotRuns = true

    /// Most recent commit_id (entries are newest-first); we diff older versions against this.
    var headCommitId: String? {
        entries.first?.commitId
    }

    var selectedFromCommitId: String? {
        selectedIndex.flatMap { entries.indices.contains($0) ? entries[$0].commitId : nil }
    }

    var visibleRows: [EvologVisibleRow] {
        evologVisibleRows(
            entries: entries,
            hideSnapshots: hideSnapshots,
            collapseSnapshotRuns: collapseSnapshotRuns
        )
    }

    var hiddenSnapshotCount: Int {
        hideSnapshots ? entries.filter { EvologDisplay.isSnapshot($0.operation) }.count : 0
    }

    init(
        entries: [EvologEntry],
        changeId: String,
        repo: JayJayRepo?,
        diffStore: DiffStore,
        hideSnapshots: Bool = false,
        collapseSnapshotRuns: Bool = true
    ) {
        self.entries = entries
        self.changeId = changeId
        self.repo = repo
        self.diffStore = diffStore
        self.hideSnapshots = hideSnapshots
        self.collapseSnapshotRuns = collapseSnapshotRuns
    }

    func setHideSnapshots(_ value: Bool) {
        hideSnapshots = value
        normalizeSelection()
    }

    func setCollapseSnapshotRuns(_ value: Bool) {
        collapseSnapshotRuns = value
        normalizeSelection()
    }

    func normalizeSelection() {
        guard let selectedIndex else { return }
        guard let selectedRowIndex = UInt32(exactly: selectedIndex),
              let row = visibleRows.first(where: { $0.indices.contains(selectedRowIndex) })
        else {
            self.selectedIndex = nil
            loadInterdiff(for: nil)
            return
        }
        let primaryIndex = Int(row.primaryIndex)
        if primaryIndex != selectedIndex {
            self.selectedIndex = primaryIndex
            loadInterdiff(for: primaryIndex)
        }
    }

    func loadInterdiff(for index: Int?) {
        selectedHunk = nil
        selectedPath = nil
        interdiffDetail = nil
        guard let index, entries.indices.contains(index),
              let repo, let to = headCommitId
        else { return }
        let from = entries[index].commitId
        if from == to {
            interdiffDetail = ChangeDetail(info: entries[index].asPlaceholderInfo(), diff: [])
            return
        }
        interdiffLoading = true
        Task.detached { [weak self] in
            let detail = try? repo.interdiffSummary(fromRev: from, toRev: to)
            await MainActor.run { [weak self] in
                guard let self, selectedIndex == index else { return }
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
              let from = selectedFromCommitId, let to = headCommitId
        else { return }
        Task.detached { [weak self] in
            let hunk = try? repo.interdiffFile(fromRev: from, toRev: to, path: path)
            await MainActor.run { [weak self] in
                guard let self, selectedPath == path else { return }
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
