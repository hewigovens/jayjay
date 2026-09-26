import Foundation
import JayJayCore

@MainActor
@Observable
final class OverviewViewModel {
    let repoPath: String
    private var repo: JayJayRepo?
    private(set) var snapshot: OverviewSnapshot?
    private(set) var error: String?
    private(set) var actionError: String?
    var selectedLaneId: String?
    var selectedChangeId: String?
    var isLanePanelShown = false
    private(set) var selectedChangeFiles: [FileDiffStats]?
    var filter = "" {
        didSet { keepSelectionVisible() }
    }

    var pendingAbandon: OverviewAbandonRequest?
    var pendingWorkspaceDelete: WorkspaceInfo?
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

    /// Divergent heads share a change id, so those lanes carry the commit id too.
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

    var visibleGroups: [OverviewGroup] {
        guard let snapshot else { return [] }
        return snapshot.groups.compactMap { group in
            let lanes = group.lanes.filter { snapshot.overview.lanes[Int($0)].matches(filter: filter) }
            return lanes.isEmpty ? nil : OverviewGroup(base: group.base, lanes: lanes)
        }
    }

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

    private func keepSelectionVisible() {
        let visible = visibleGroups.flatMap(\.lanes).map { laneIds[Int($0)] }
        if let selectedLaneId, visible.contains(selectedLaneId) {
            return
        }
        selectedLaneId = visible.first
        selectedChangeId = nil
    }

    /// Two handles snapshotting one working copy can fork `@`, so an open repo window on this checkout wins.
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

    func selectNeighbor(_ delta: Int) {
        let order = visibleGroups.flatMap(\.lanes).map { laneIds[Int($0)] }
        guard !order.isEmpty else { return }
        let current = order.firstIndex { $0 == selectedLaneId } ?? 0
        selectedLaneId = order[(current + delta).clamped(to: 0 ... (order.count - 1))]
        selectedChangeId = nil
    }

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

    func baseSentence(trunkName: String) -> String {
        switch base.kind {
            case .trunk:
                "On \(trunkName)"
            case .olderTrunk:
                "\(base.behindTrunk) commit\(base.behindTrunk == 1 ? "" : "s") behind \(trunkName)"
            case .mutable:
                "Forked from \(base.description.isEmpty ? base.changeId.prefix : base.description)"
            case .other:
                "On \(base.bookmarks.first ?? base.changeId.prefix), off \(trunkName)"
        }
    }

    var footer: String {
        attention.first ?? "\(changes.count) change\(changes.count == 1 ? "" : "s") · \(Date.relativeLabel(millis: latestTimestampMillis))"
    }

    func matches(filter: String) -> Bool {
        let needle = filter.trimmingCharacters(in: .whitespaces)
        guard !needle.isEmpty else { return true }
        let haystacks = changes.flatMap { [$0.description, $0.changeId.id] + $0.bookmarks + $0.workspaces }
        return haystacks.contains { $0.localizedCaseInsensitiveContains(needle) }
    }
}

extension OverviewWorkspace {
    var label: String {
        isCurrent ? "@ \(name)" : "\(name)@"
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

struct OverviewAbandonRequest: Identifiable {
    let title: String
    let commitIds: [String]
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

    func isCurrent(in lanes: [OverviewLane], ids: [String]) -> Bool {
        if let laneId {
            return ids.firstIndex(of: laneId).map { lanes[$0].changes.map(\.commitId.id) } == commitIds
        }
        return lanes.contains { lane in lane.changes.contains { commitIds.contains($0.commitId.id) } }
    }
}
