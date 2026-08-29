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
    var selectedIndex: Int?
    var interdiffDetail: ChangeDetail?
    var interdiffLoading = false
    var selectedPath: String?
    var selectedHunk: DiffHunk?

    /// Most recent commit_id (entries are newest-first); we diff older versions against this.
    var headCommitId: String? {
        entries.first?.commitId.id
    }

    var displayedRows: [EvologRow] {
        evologRows(entries: entries, hideSnapshots: hideSnapshots, expandedRuns: Array(expandedSnapshotRuns))
    }

    var selectedFromCommitId: String? {
        selectedIndex.flatMap { entries.indices.contains($0) ? entries[$0].commitId.id : nil }
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
        expandedSnapshotRuns.removeAll()
        if let selectedIndex, let row = displayedRows.first(where: { $0.range.contains(selectedIndex) }) {
            self.selectedIndex = row.actionIndex
        }
    }

    func select(_ row: EvologRow?) {
        guard let row else {
            selectedIndex = nil
            return
        }
        if row.isCollapsedRun {
            expandedSnapshotRuns.insert(row.start)
        }
        selectedIndex = row.actionIndex
    }

    func loadInterdiff(for index: Int?) {
        selectedHunk = nil
        selectedPath = nil
        interdiffDetail = nil
        guard let index, entries.indices.contains(index),
              let repo, let to = headCommitId
        else { return }
        let from = entries[index].commitId.id
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

extension EvologRow: Identifiable {
    public var id: Self {
        self
    }

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
