import Foundation
import JayJayCore

@Observable
final class RepoViewModel: ChangeActions, DAGActions, BookmarkActions {
    static let defaultLogPageSize = Int(logPageSize())

    let repoPath: String
    var graphEntries: [GraphEntry] = []
    var dagLayout = DAGLayout(entries: [])
    var changes: [ChangeInfo] {
        graphEntries.map(\.change)
    }

    func change(for rev: String) -> ChangeInfo? {
        changes.first(where: { $0.matchesRevision(rev) })
    }

    var selectedChange: ChangeDetail?
    var selectedChangeId: String?
    var selectedChangeIds: [String] = []
    @ObservationIgnored var selectedChangeAnchorId: String?
    @ObservationIgnored var selectionLoadTask: Task<Void, Never>?
    @ObservationIgnored var lastKeyboardSelection: ContinuousClock.Instant?
    @ObservationIgnored var comparisonRequestId: UInt64 = 0
    /// When set, the detail panel shows an interdiff (from → to).
    var compareFromId: String?
    var compareToId: String?
    var compareDisplay: CompareDisplay?
    var bookmarks: [BookmarkInfo] = []
    var conflictedBookmarkNames: Set<String> {
        Set(bookmarks.filter(\.isConflicted).map(\.name))
    }

    var workingCopyDescription: String = ""
    /// Change ID the commit-box draft belongs to; when @ moves to a described change, the draft is reseeded from that description.
    var workingCopyChangeId: String = ""
    var workingCopyStats: DiffStats?
    var currentOperationDescription: String = ""
    var commitSummaryDraft: String = ""
    var commitDescriptionDraft: String = ""
    var opLogEntries: [OpLogEntry] = []
    var submoduleAttentionItems: [GitSubmoduleStatus] = []
    var pendingCommitMessage: String?
    var error: String?
    var workspaceVanished = false
    var info: String?
    /// A tracked bookmark just moved by drag, awaiting an optional one-click push.
    var pendingPushBookmark: String?
    var workspaces: [WorkspaceInfo] = []
    var isLoading = false
    var canLoadMore = true
    let reviewStore = ReviewStore()
    let diffStore = DiffStore()

    var revset: String = RepoViewModel.buildDefaultRevset()
    /// True when `revset` was never explicitly typed/selected — the current load must pass `nil` for
    /// `logGraphPage(revset:)` (not `revset`'s text) so it gets the `revsets.log` resolution and
    /// sparse-context widening that only the Rust `LogQuery::Default` path triggers.
    var isDefaultRevset = true
    var appliedLimit: Int = RepoViewModel.defaultLogPageSize

    /// The `revset` argument the next `logGraphPage` load should pass: `nil` selects
    /// `LogQuery::Default` on the Rust side when `revset` was never explicitly typed/selected.
    var currentLogQueryRevset: String? {
        isDefaultRevset ? nil : revset
    }

    let repo: JayJayRepo

    /// Huge checkouts (e.g. chromium) skip the working-copy snapshot on open; small repos refresh eagerly.
    let workingCopyIsLarge: Bool

    var aiProvider: String = ""
    var successActionSignal = 0
    var configWarning: String?
    private var fsWatcher: RepoFSWatcher?
    var refreshTask: Task<Void, Never>?
    /// A superseded refresh stays registered: cancellation cannot interrupt synchronous FFI.
    var repoTasks: [UUID: Task<Void, Never>] = [:]
    var isShuttingDown = false
    /// Stamp set by `perform()` so handleWorkingCopyChange can suppress its own FS echo.
    var lastInternalMutationAt: Date?
    /// FS-triggered refreshes wait while a sheet or editor owns transient user input.
    var isBackgroundRefreshSuspended = false
    var hasPendingBackgroundRefresh = false
    /// True while a refresh task is running — gates FS-triggered re-entry.
    var isRefreshingInFlight: Bool = false
    var isPullingInFlight = false
    var isPushingInFlight = false
    var pullSync: JayJaySyncToken?
    var pushSync: JayJaySyncToken?
    var isAddingWorkspace = false
    var includeSubmoduleStatuses: Bool
    var prInfo: PrInfo?
    var prFetchTask: Task<Void, Never>?
    var prHostName: String?
    var evologEntries: [EvologEntry]?
    var evologRev: String?

    convenience init(path: String, includeSubmoduleStatuses: Bool = false) throws {
        let repo = try JayJayRepo.open(path: path)
        self.init(
            path: path,
            repo: repo,
            workingCopyIsLarge: repo.workingCopyIsLarge(),
            configWarning: repo.checkUserConfig(),
            includeSubmoduleStatuses: includeSubmoduleStatuses
        )
    }

    /// Designated init taking values `openRepo()` precomputes off the main thread (blocking FFI)
    /// so window open never stalls. `prHostName` stays nil here; the first refresh populates it.
    init(
        path: String,
        repo: JayJayRepo,
        workingCopyIsLarge: Bool,
        configWarning: String?,
        includeSubmoduleStatuses: Bool = false
    ) {
        repoPath = path
        self.includeSubmoduleStatuses = includeSubmoduleStatuses
        self.repo = repo
        self.workingCopyIsLarge = workingCopyIsLarge
        aiProvider = Self.detectAIProvider()
        self.configWarning = configWarning
        fsWatcher = RepoFSWatcher(
            repoPath: path,
            onChange: { [weak self] in self?.handleWorkingCopyChange() },
            onWorkingCopyChange: { [weak self] in self?.handleWorkingCopyChange() },
            isRelevantWorkingCopyChange: { [repo] paths in
                (try? repo.hasUnignoredWorkingCopyPaths(paths: paths)) ?? true
            }
        )
    }

    @MainActor
    func beginShutdown() {
        guard !isShuttingDown else { return }
        isShuttingDown = true
        repoTasks.values.forEach { $0.cancel() }
        refreshTask = nil
        prFetchTask = nil
    }

    @MainActor
    func prepareForTermination() {
        beginShutdown()
        repo.cancelRunningJjProcesses()
    }

    @MainActor
    func prepareForRemoval() async {
        beginShutdown()
        while let task = repoTasks.values.first {
            await task.value
        }
    }

    @MainActor
    func resumeAfterFailedRemoval() {
        guard isShuttingDown else { return }
        isShuttingDown = false
        refresh()
    }

    private static func detectAIProvider() -> String {
        let cli = detectAiProvider() // from Rust via uniffi
        if !cli.isEmpty {
            return cli
        }
        #if canImport(FoundationModels)
            return "Apple Intelligence"
        #else
            return ""
        #endif
    }

    static func buildDefaultRevset() -> String {
        buildDefaultRevset(depth: Int(defaultLogContextDepth()))
    }

    static func buildDefaultRevset(depth: Int) -> String {
        defaultRevsetWithDepth(depth: UInt32(depth))
    }
}
