import Foundation
import JayJayCore
#if canImport(FoundationModels)
    import FoundationModels
#endif

@Observable
final class RepoViewModel: ChangeActions, DAGActions, BookmarkActions {
    let repoPath: String
    private(set) var graphEntries: [GraphEntry] = []
    var changes: [ChangeInfo] {
        graphEntries.map(\.change)
    }

    var selectedChange: ChangeDetail?
    var selectedChangeId: String?
    /// When set, the detail panel shows an interdiff (from → to).
    var compareFromId: String?
    private(set) var bookmarks: [BookmarkInfo] = []
    private(set) var workingCopyDescription: String = ""
    var opLogEntries: [OpLogEntry] = []
    var error: String?
    var info: String?
    private(set) var workspaces: [WorkspaceInfo] = []
    private(set) var isLoading = false
    let reviewStore = ReviewStore()

    var revset: String = defaultRevset()
    /// Number of ancestors to show in default revset. Increases on "Load More".
    var ancestorLimit: Int = 20
    /// Whether a custom (non-default) revset is active.
    var isCustomRevset: Bool {
        revset != Self.buildDefaultRevset(limit: ancestorLimit)
    }

    /// False when load-more returned no new entries (reached oldest commit).
    private(set) var hasMoreToLoad = true

    let repo: JayJayRepo

    var aiProvider: String = ""
    var hasWorkingCopyChanges = false
    private var fsWatcher: RepoFSWatcher?
    private var refreshTask: Task<Void, Never>?

    init(path: String) throws {
        repoPath = path
        repo = try JayJayRepo.open(path: path)
        reviewStore.setRepoPath(path)
        aiProvider = Self.detectAIProvider()
        fsWatcher = RepoFSWatcher(
            repoPath: path,
            onChange: { [weak self] in self?.refresh() },
            onWorkingCopyChange: { [weak self] in self?.hasWorkingCopyChanges = true }
        )
    }

    private static func detectAIProvider() -> String {
        let cli = detectAiProvider() // from Rust via uniffi
        if !cli.isEmpty { return cli }
        #if canImport(FoundationModels)
            if #available(macOS 26.0, *) { return "Apple Intelligence" }
        #endif
        return ""
    }

    static func buildDefaultRevset(limit: Int) -> String {
        "@ | ancestors(@, \(limit)) | @-+"
    }

    func applyRevset(_ newRevset: String) {
        revset = newRevset
        hasMoreToLoad = true
        ancestorLimit = 20
        refresh(selecting: "@")
    }

    func loadMore() {
        let previousCount = graphEntries.count
        ancestorLimit += 20
        revset = Self.buildDefaultRevset(limit: ancestorLimit)
        Task.detached { [repo, revset] in
            guard let graph = try? repo.logGraph(revset: revset) else { return }
            await MainActor.run { [weak self] in
                self?.graphEntries = graph
                if graph.count <= previousCount {
                    self?.hasMoreToLoad = false
                }
            }
        }
    }

    // MARK: - Perform helper

    /// Runs a repo action off the main thread, then refreshes on success or shows an error.
    func perform(selecting rev: String? = "@", _ action: @escaping (JayJayRepo) throws -> Void) {
        Task.detached { [repo] in
            do {
                try action(repo)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: rev)
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func refresh(selecting preferredRev: String? = nil) {
        refreshTask?.cancel()
        isLoading = graphEntries.isEmpty
        hasWorkingCopyChanges = false
        error = nil
        let currentSelection = selectedChangeId
        refreshTask = Task.detached { [repo, revset] in
            do {
                try repo.refreshWorkingCopy()
                guard !Task.isCancelled else { return }

                // Try the revset — if it fails, show empty list (not an error alert)
                let graph: [GraphEntry]
                do {
                    graph = try repo.logGraph(revset: revset)
                } catch {
                    guard !Task.isCancelled else { return }
                    await MainActor.run { [weak self] in
                        self?.graphEntries = []
                        self?.selectedChange = nil
                        self?.selectedChangeId = nil
                        self?.isLoading = false
                    }
                    return
                }

                guard !Task.isCancelled else { return }

                let log = graph.map(\.change)
                let marks = try repo.listBookmarks()
                let wsList = (try? repo.workspaceList()) ?? []
                let detail = try Self.loadSelectedDetail(
                    repo: repo,
                    log: log,
                    preferredRev: preferredRev ?? currentSelection
                )
                let wcDesc = log.first(where: { $0.isWorkingCopy })?.description ?? ""
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.graphEntries = graph
                    self?.bookmarks = marks
                    self?.workspaces = wsList
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail?.info.changeId
                    self?.workingCopyDescription = wcDesc
                    self?.isLoading = false
                    self?.hasWorkingCopyChanges = false
                }
            } catch {
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                    self?.isLoading = false
                }
            }
        }
    }

    func select(changeId: String?) {
        compareFromId = nil
        selectedChangeId = changeId
        guard let changeId else {
            selectedChange = nil
            return
        }

        Task.detached { [repo] in
            do {
                let detail = try Self.loadSummaryWithConflicts(repo: repo, rev: changeId)
                await MainActor.run { [weak self] in
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail.info.changeId
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func compareWith(from: String, to: String) {
        compareFromId = from
        selectedChangeId = to
        Task.detached { [repo] in
            do {
                let detail = try repo.interdiffSummary(fromRev: from, toRev: to)
                await MainActor.run { [weak self] in
                    self?.selectedChange = detail
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                    self?.compareFromId = nil
                }
            }
        }
    }

    func clearCompare() {
        compareFromId = nil
        if let selectedChangeId {
            select(changeId: selectedChangeId)
        }
    }

    /// Load summary and merge any conflicted files that don't appear in the normal diff.
    static func loadSummaryWithConflicts(repo: JayJayRepo, rev: String) throws -> ChangeDetail {
        var detail = try repo.showSummary(rev: rev)
        if detail.info.hasConflict {
            let conflictPaths = (try? repo.resolveList(rev: rev)) ?? []
            let existingPaths = Set(detail.diff.map(\.path))
            let missing = conflictPaths.filter { !existingPaths.contains($0) }
            if !missing.isEmpty {
                var hunks = detail.diff
                for path in missing {
                    hunks.append(DiffHunk(
                        path: path, oldPath: nil,
                        oldContent: nil, newContent: nil,
                        hunkType: .modified
                    ))
                }
                detail = ChangeDetail(info: detail.info, diff: hunks)
            }
        }
        return detail
    }

    static func loadSelectedDetail(
        repo: JayJayRepo,
        log: [ChangeInfo],
        preferredRev: String?
    ) throws -> ChangeDetail? {
        var candidates = [String]()
        if let preferredRev, !preferredRev.isEmpty {
            candidates.append(preferredRev)
        }
        if let firstChange = log.first?.changeId, !candidates.contains(firstChange) {
            candidates.append(firstChange)
        }

        for candidate in candidates {
            guard let detail = try? loadSummaryWithConflicts(repo: repo, rev: candidate) else { continue }
            if log.contains(where: { $0.changeId == detail.info.changeId }) {
                return detail
            }
        }

        return nil
    }
}
