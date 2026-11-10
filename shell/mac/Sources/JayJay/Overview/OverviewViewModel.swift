import Foundation
import JayJayCore

@MainActor
@Observable
final class OverviewViewModel {
    let repoPath: String
    private var repo: JayJayRepo?
    private(set) var snapshot: OverviewSnapshot? {
        didSet {
            laneIds = snapshot.map { overviewLaneIds(snapshot: $0) } ?? []
            laneIndexById = Dictionary(uniqueKeysWithValues: laneIds.enumerated().map { ($1, $0) })
            trunkName = snapshot.map { overviewTrunkName(snapshot: $0) } ?? "trunk"
            laneFilter = snapshot.map { OverviewFilter(snapshot: $0) }
            rebuildLayout()
        }
    }

    private(set) var error: String?
    private(set) var actionError: String?
    var selectedLaneId: String?
    var selectedChangeId: String?
    var isLanePanelShown = false
    private(set) var selectedChangeFiles: Result<[FileDiffStats], any Error>?
    var filter = "" {
        didSet {
            rebuildLayout()
            keepSelectionVisible()
        }
    }

    /// Cached because the canvas reads them on every update.
    private(set) var laneIds: [String] = []
    private(set) var trunkName = "trunk"
    private(set) var visibleGroups: [OverviewGroup] = []
    private(set) var placement = OverviewPlacement(lanes: [], groups: [])
    @ObservationIgnored private var laneIndexById: [String: Int] = [:]
    @ObservationIgnored private var laneFilter: OverviewFilter?

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

    private func rebuildLayout() {
        visibleGroups = laneFilter?.visibleGroups(filter: filter) ?? []
        placement = OverviewPlacement(lanes: lanes, groups: visibleGroups)
    }

    var selectedLane: OverviewLane? {
        guard let selectedLaneId, let index = laneIndexById[selectedLaneId] else { return nil }
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
        if selectedLaneId.flatMap({ laneIndexById[$0] }) == nil {
            selectedLaneId = loaded.groups.first?.lanes.first.map { laneIds[Int($0)] }
            selectedChangeId = nil
        }
        if let pendingAbandon, !pendingAbandon.isCurrent(in: lanes, ids: laneIds) {
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
            Result { try repo.diffFileStats(rev: selectedChangeId, ignoreWhitespace: false) }
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
        mutate { _ = try $0.rebase(rev: rev, dest: "trunk()", mode: .source) }
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
        head.description.isEmpty ? Localized.Repo.noDescriptionPlaceholder : head.description
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
                Localized.Overview.onTrunk(trunk: trunkName)
            case .olderTrunk:
                Localized.Overview.commitsBehind(count: Int(base.behindTrunk), trunk: trunkName)
            case .mutable:
                Localized.Overview.forkedFrom(base: base.description.isEmpty ? base.changeId.prefix : base.description)
            case .other:
                Localized.Overview.onOffTrunk(base: base.bookmarks.first ?? base.changeId.prefix, trunk: trunkName)
        }
    }

    var footer: String {
        attention.first ?? Localized.Overview.laneSummary(changes: Localized.Repo.changeCount(count: changes.count), age: Date.relativeLabel(millis: latestTimestampMillis))
    }
}

extension OverviewWorkspace {
    var label: String {
        isCurrent ? "@ \(name)" : "\(name)@"
    }
}

extension OverviewChange {
    var title: String {
        description.isEmpty ? (isEmpty ? Localized.Repo.emptyPlaceholder : Localized.Repo.noDescriptionPlaceholder) : description
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
            ? Localized.Overview.abandonThisChangeItsDescendantsAre
            : Localized.Overview.abandonLaneMessage(count: commitIds.count)
    }

    func isCurrent(in lanes: [OverviewLane], ids: [String]) -> Bool {
        if let laneId {
            return ids.firstIndex(of: laneId).map { lanes[$0].changes.map(\.commitId.id) } == commitIds
        }
        return lanes.contains { lane in lane.changes.contains { commitIds.contains($0.commitId.id) } }
    }
}
