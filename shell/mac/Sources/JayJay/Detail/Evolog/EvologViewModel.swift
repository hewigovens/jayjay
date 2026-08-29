import AppKit
import JayJayCore
import SwiftUI

@Observable
final class EvologViewModel {
    /// One list row: a real evolog entry, or a collapsed run of consecutive snapshots.
    enum Row: Hashable, Identifiable {
        case entry(Int)
        case collapsedRun(Range<Int>)

        var id: Self {
            self
        }

        /// Entry used for Compare / Restore / copy-command. Collapsed runs use their newest snapshot (lowest index; evolog is newest-first).
        var actionIndex: Int {
            switch self {
                case let .entry(index): index
                case let .collapsedRun(range): range.lowerBound
            }
        }

        func contains(_ index: Int) -> Bool {
            switch self {
                case let .entry(entryIndex): entryIndex == index
                case let .collapsedRun(range): range.contains(index)
            }
        }
    }

    let entries: [EvologEntry]
    let changeId: String
    let repo: JayJayRepo?
    let diffStore: DiffStore

    private(set) var hideSnapshots = true
    var expandedSnapshotRuns: Set<Int> = []
    var selectedIndex: Int?
    var interdiffDetail: ChangeDetail?
    var interdiffLoading = false
    var selectedPath: String?
    var selectedHunk: DiffHunk?

    /// Most recent commit_id (entries are newest-first); we diff older versions against this.
    var headCommitId: String? {
        entries.first?.commitId.id
    }

    var displayedRows: [Row] {
        Self.displayedRows(
            entries: entries,
            hideSnapshots: hideSnapshots,
            expandedRuns: expandedSnapshotRuns
        )
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

    static func displayedRows(
        entries: [EvologEntry],
        hideSnapshots: Bool,
        expandedRuns: Set<Int> = []
    ) -> [Row] {
        guard hideSnapshots else {
            return entries.indices.map(Row.entry)
        }
        var rows: [Row] = []
        var index = 0
        while index < entries.count {
            // The newest entry stays a real row so the current state is always visible.
            if index == 0 || !EvologDisplay.isSnapshot(entries[index].operation) {
                rows.append(.entry(index))
                index += 1
                continue
            }
            let start = index
            index += 1
            while index < entries.count, EvologDisplay.isSnapshot(entries[index].operation) {
                index += 1
            }
            let range = start ..< index
            if range.count == 1 || expandedRuns.contains(start) {
                rows.append(contentsOf: range.map(Row.entry))
            } else {
                rows.append(.collapsedRun(range))
            }
        }
        return rows
    }

    func setHideSnapshots(_ hide: Bool) {
        guard hideSnapshots != hide else { return }
        hideSnapshots = hide
        guard hide else { return }
        expandedSnapshotRuns.removeAll()
        if let selectedIndex, let row = displayedRows.first(where: { $0.contains(selectedIndex) }) {
            self.selectedIndex = row.actionIndex
        }
    }

    func select(_ row: Row?) {
        guard let row else {
            selectedIndex = nil
            return
        }
        if case let .collapsedRun(range) = row {
            expandedSnapshotRuns.insert(range.lowerBound)
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
