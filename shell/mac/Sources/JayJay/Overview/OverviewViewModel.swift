import Foundation
import JayJayCore

/// Owns the Overview window's repo handle: one snapshot per load, refreshed when the repository's operation log changes.
@MainActor
@Observable
final class OverviewViewModel {
    let repoPath: String
    private var repo: JayJayRepo?
    private(set) var snapshot: OverviewSnapshot?
    /// Why the snapshot could not load; cleared by the next successful load.
    private(set) var error: String?
    /// Why the last rebase, abandon, or forget failed; stays until the next action so a reload cannot hide it.
    private(set) var actionError: String?
    /// One of `laneIds`: it survives rewrites and reorders, which a lane index would not.
    var selectedLaneId: String?
    var selectedChangeId: String?
    private(set) var selectedChangeFiles: [FileDiffStats]?
    var filter = "" {
        didSet { keepSelectionVisible() }
    }

    var pendingAbandon: OverviewAbandonRequest?
    var pendingWorkspaceDelete: WorkspaceInfo?
    /// A repo window on this checkout snapshots working-copy edits itself; only without one does the overview do it.
    var hasRepoWindow: () -> Bool = { false }

    @ObservationIgnored private var fsWatcher: RepoFSWatcher?
    @ObservationIgnored private var loadTask: Task<Void, Never>?
    @ObservationIgnored private var generation = 0
    @ObservationIgnored private var lastInternalMutationAt: Date?

    init(repoPath: String) {
        self.repoPath = repoPath
    }

    var lanes: [OverviewLane] {
        snapshot?.overview.lanes ?? []
    }

    /// Head change id per lane, which stays put across rewrites; divergent heads share one, so those carry the commit too.
    var laneIds: [String] {
        Self.laneIds(for: lanes)
    }

    private static func laneIds(for lanes: [OverviewLane]) -> [String] {
        var seen: [String: Int] = [:]
        for lane in lanes {
            seen[lane.head.changeId.id, default: 0] += 1
        }
        return lanes.map { lane in
            let changeId = lane.head.changeId.id
            return seen[changeId] == 1 ? changeId : "\(changeId)/\(lane.head.commitId.id)"
        }
    }

    /// Groups with the filter applied; a lane that matches nothing disappears rather than dims, and empty groups go with it.
    var visibleGroups: [OverviewGroup] {
        guard let snapshot else { return [] }
        return snapshot.groups.compactMap { group in
            let lanes = group.lanes.filter { snapshot.overview.lanes[Int($0)].matches(filter: filter) }
            return lanes.isEmpty ? nil : OverviewGroup(base: group.base, lanes: lanes)
        }
    }

    /// Only an unambiguous name: with several bookmarks on trunk, the rebase target is still `trunk()`, so say so.
    var trunkName: String {
        let bookmarks = snapshot?.groups.first { $0.base.kind == .trunk }?.base.bookmarks ?? []
        return bookmarks.count == 1 ? bookmarks[0] : "trunk"
    }

    var selectedLane: OverviewLane? {
        guard let selectedLaneId, let index = laneIds.firstIndex(of: selectedLaneId) else { return nil }
        return lanes[index]
    }

    var selectedChange: OverviewChange? {
        guard let selectedChangeId else { return nil }
        return selectedLane?.changes.first { $0.commitId.id == selectedChangeId }
    }

    var repositoryStorePath: String? {
        repo?.repositoryStorePath()
    }

    func workspaceInfo(named name: String) -> WorkspaceInfo? {
        snapshot?.workspaces.first { $0.name == name }
    }

    /// The window a lane should open in: its own workspace when it has one on disk, else the Overview's repository.
    func targetRepoPath(for lane: OverviewLane) -> String {
        if lane.currentWorkspace != nil {
            return repoPath
        }
        let paths = lane.workspaces.compactMap { workspaceInfo(named: $0.name) }.filter(\.isPathResolved)
        return paths.first?.path ?? repoPath
    }

    func open() async {
        guard repo == nil else { return }
        let path = repoPath
        let expected = generation
        let result = await Task.detached {
            Result { try JayJayRepo.open(path: path) }
        }.value
        guard expected == generation else { return }
        switch result {
            case let .success(opened):
                repo = opened
                fsWatcher = RepoFSWatcher(
                    repoPath: path,
                    onChange: { [weak self] in Task { @MainActor in self?.load() } },
                    onWorkingCopyChange: { [weak self] in Task { @MainActor in self?.snapshotWorkingCopyIfUnwatched() } },
                    isRelevantWorkingCopyChange: { [repo = opened] paths in
                        (try? repo.hasUnignoredWorkingCopyPaths(paths: paths)) ?? true
                    }
                )
                load()
            case let .failure(error):
                self.error = error.friendlyDescription
        }
    }

    /// A closed AppKit window can linger with its SwiftUI state, so the watcher and any open or load stop here rather than at deinit.
    func close() {
        fsWatcher = nil
        loadTask?.cancel()
        generation += 1
        repo = nil
    }

    func load() {
        guard let repo else { return }
        generation += 1
        let expected = generation
        loadTask?.cancel()
        loadTask = Task { [repo] in
            let result = await Task.detached {
                Result { try repo.overviewSnapshot() }
            }.value
            guard !Task.isCancelled, expected == generation else { return }
            switch result {
                case let .success(loaded):
                    snapshot = loaded
                    error = nil
                    reconcileSelection(with: loaded)
                case let .failure(failure):
                    error = failure.friendlyDescription
            }
        }
    }

    /// A reload can drop or rewrite what the user pointed at; selection follows identity and a stale confirmation closes.
    private func reconcileSelection(with loaded: OverviewSnapshot) {
        let lanes = loaded.overview.lanes
        let ids = Self.laneIds(for: lanes)
        if !ids.contains(where: { $0 == selectedLaneId }) {
            selectedLaneId = loaded.groups.first?.lanes.first.map { ids[Int($0)] }
            selectedChangeId = nil
        }
        if let pendingAbandon, !pendingAbandon.isCurrent(in: lanes, ids: ids) {
            self.pendingAbandon = nil
        }
        if let pendingWorkspaceDelete, !loaded.workspaces.contains(where: { $0.name == pendingWorkspaceDelete.name }) {
            self.pendingWorkspaceDelete = nil
        }
        keepSelectionVisible()
    }

    /// A filtered-out selection would keep the panel open on nothing visible; move it to the first lane still shown.
    private func keepSelectionVisible() {
        let visible = visibleGroups.flatMap(\.lanes).map { laneIds[Int($0)] }
        if let selectedLaneId, visible.contains(selectedLaneId) {
            return
        }
        selectedLaneId = visible.first
        selectedChangeId = nil
    }

    /// Two handles snapshotting one working copy can fork `@`, so this yields to an open repo window on the same checkout,
    /// and skips the echo of its own writes.
    private func snapshotWorkingCopyIfUnwatched() {
        guard !hasRepoWindow() else { return }
        if let last = lastInternalMutationAt, Date().timeIntervalSince(last) < 5 {
            return
        }
        mutate { try $0.refreshWorkingCopy() }
    }

    func loadSelectedChangeFiles() async {
        selectedChangeFiles = nil
        guard let repo, let selectedChangeId else { return }
        let files = await Task.detached {
            (try? repo.diffFileStats(rev: selectedChangeId, ignoreWhitespace: false)) ?? []
        }.value
        guard selectedChangeId == self.selectedChangeId else { return }
        selectedChangeFiles = files
    }

    /// Left and right move through the lanes as drawn, group by group.
    func selectNeighbor(_ delta: Int) {
        let order = visibleGroups.flatMap(\.lanes).map { laneIds[Int($0)] }
        guard !order.isEmpty else { return }
        let current = order.firstIndex { $0 == selectedLaneId } ?? 0
        selectedLaneId = order[(current + delta).clamped(to: 0 ... (order.count - 1))]
        selectedChangeId = nil
    }

    /// Up and down move through the selected lane's changes, head first; past either end clears the change.
    func selectChangeNeighbor(_ delta: Int) {
        guard let lane = selectedLane else { return }
        let ids = lane.changes.map(\.commitId.id)
        guard let selectedChangeId, let current = ids.firstIndex(of: selectedChangeId) else {
            selectedChangeId = delta > 0 ? ids.first : ids.last
            return
        }
        let next = current + delta
        self.selectedChangeId = ids.indices.contains(next) ? ids[next] : nil
    }

    /// Moves the whole lane: descendants follow the rebased root, and a checkout in the lane moves with it.
    func rebaseLaneOntoTrunk(_ lane: OverviewLane) {
        guard let root = lane.changes.last else { return }
        let rev = root.commitId.id
        mutate { try $0.rebase(rev: rev, dest: "trunk()", mode: .source) }
    }

    func abandon(_ request: OverviewAbandonRequest) {
        let revs = request.commitIds
        if revs.count == 1 {
            mutate { try $0.abandon(rev: revs[0]) }
        } else {
            mutate { try $0.abandonMany(revs: revs) }
        }
    }

    /// Returns whether the workspace was forgotten; the caller quiesces and closes its window around this.
    func forgetWorkspace(_ workspace: WorkspaceInfo, deleteFromDisk: Bool) async -> Bool {
        guard let repo else { return false }
        actionError = nil
        lastInternalMutationAt = Date()
        let result = await Task.detached { () -> Result<String?, Error> in
            Result {
                if deleteFromDisk {
                    return try repo.workspaceForgetAndDelete(name: workspace.name, expectedRoot: workspace.path)
                }
                try repo.workspaceForget(name: workspace.name, expectedRoot: workspace.isPathResolved ? workspace.path : nil)
                return nil
            }
        }.value
        load()
        switch result {
            case let .success(warning):
                actionError = warning
                return true
            case let .failure(failure):
                actionError = failure.friendlyDescription
                return false
        }
    }

    private func mutate(_ work: @escaping @Sendable (JayJayRepo) throws -> Void) {
        guard let repo else { return }
        actionError = nil
        lastInternalMutationAt = Date()
        Task { [repo] in
            let result = await Task.detached { Result { try work(repo) } }.value
            if case let .failure(failure) = result {
                actionError = failure.friendlyDescription
            }
            load()
        }
    }
}

private extension Int {
    func clamped(to range: ClosedRange<Int>) -> Int {
        Swift.min(Swift.max(self, range.lowerBound), range.upperBound)
    }
}

extension OverviewLane {
    var head: OverviewChange {
        changes[0]
    }

    var title: String {
        head.description.isEmpty ? "(no description)" : head.description
    }

    var currentWorkspace: OverviewWorkspace? {
        workspaces.first(where: \.isCurrent)
    }

    var isBehindTrunk: Bool {
        base.kind == .olderTrunk
    }

    /// One sentence about the base, worded for a card.
    func baseSentence(trunkName: String) -> String {
        switch base.kind {
            case .trunk:
                "On \(trunkName)"
            case .olderTrunk:
                "\(base.behindTrunk) commit\(base.behindTrunk == 1 ? "" : "s") behind \(trunkName), based on \(Date.relativeLabel(millis: base.timestampMillis))"
            case .mutable:
                "On \(base.description.isEmpty ? base.changeId.prefix : base.description)"
            case .other:
                "On \(base.bookmarks.first ?? base.changeId.prefix), off trunk"
        }
    }

    func matches(filter: String) -> Bool {
        let needle = filter.trimmingCharacters(in: .whitespaces)
        guard !needle.isEmpty else { return true }
        let haystacks = changes.flatMap { [$0.description, $0.changeId.id] + $0.bookmarks + $0.workspaces }
        return haystacks.contains { $0.localizedCaseInsensitiveContains(needle) }
    }
}

extension OverviewChange {
    var title: String {
        description.isEmpty ? (isEmpty ? "(empty)" : "(no description)") : description
    }
}

extension ShortId {
    var prefix: String {
        String(id.prefix(Int(shortLen)))
    }
}

/// What an Abandon confirmation is about: one change, or every change of a lane.
struct OverviewAbandonRequest: Identifiable {
    let title: String
    let commitIds: [String]
    /// Set for a whole-lane request, so a lane that gained or lost changes since invalidates it.
    let laneId: String?

    var id: String {
        commitIds.joined(separator: ",")
    }

    static func lane(_ lane: OverviewLane, id: String) -> Self {
        Self(title: lane.title, commitIds: lane.changes.map(\.commitId.id), laneId: id)
    }

    static func change(_ change: OverviewChange) -> Self {
        Self(
            title: change.description.isEmpty ? change.changeId.prefix : change.description,
            commitIds: [change.commitId.id],
            laneId: nil
        )
    }

    var message: String {
        commitIds.count == 1
            ? "Abandon this change? Its descendants are rebased onto its parent, and a workspace checked out here moves to a new empty change."
            : "Abandon all \(commitIds.count) changes of this lane? A workspace checked out in it moves to a new empty change on the base."
    }

    /// Whether the confirmation still describes what is on screen after a reload.
    func isCurrent(in lanes: [OverviewLane], ids: [String]) -> Bool {
        if let laneId {
            return ids.firstIndex(of: laneId).map { lanes[$0].changes.map(\.commitId.id) } == commitIds
        }
        return lanes.contains { lane in lane.changes.contains { commitIds.contains($0.commitId.id) } }
    }
}
