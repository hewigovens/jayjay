import Foundation
import JayJayCore

@Observable
final class RepoViewModel: ChangeActions, DAGActions, BookmarkActions {
    static let defaultRevsetPageSize = 20

    let repoPath: String
    var graphEntries: [GraphEntry] = []
    var changes: [ChangeInfo] {
        graphEntries.map(\.change)
    }

    func change(for rev: String) -> ChangeInfo? {
        changes.first(where: { $0.matchesRevision(rev) })
    }

    var selectedChange: ChangeDetail?
    var selectedChangeId: String?
    @ObservationIgnored var selectionLoadTask: Task<Void, Never>?
    @ObservationIgnored var lastKeyboardSelection: ContinuousClock.Instant?
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

    var revset: String = defaultRevset()

    let repo: JayJayRepo

    /// Huge checkouts (e.g. chromium) skip the working-copy snapshot on open; small repos refresh eagerly.
    let workingCopyIsLarge: Bool

    var aiProvider: String = ""
    var hasWorkingCopyChanges = false
    var successActionSignal = 0
    var configWarning: String?
    private var fsWatcher: RepoFSWatcher?
    var refreshTask: Task<Void, Never>?
    /// A superseded refresh stays registered: cancellation cannot interrupt synchronous FFI.
    var repoTasks: [UUID: Task<Void, Never>] = [:]
    var isShuttingDown = false
    /// Stamp set by `perform()` so handleWorkingCopyChange can suppress its own FS echo.
    var lastInternalMutationAt: Date?
    /// True while a refresh task is running — gates FS-triggered re-entry.
    var isRefreshingInFlight: Bool = false
    var isPullingInFlight = false
    var isPushingInFlight = false
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
        buildDefaultRevset(depth: defaultRevsetPageSize)
    }

    static func buildDefaultRevset(depth: Int) -> String {
        defaultRevsetWithDepth(depth: UInt32(depth))
    }

    static func defaultRevsetDepth(for revset: String) -> Int? {
        let prefix = "present(@) | ancestors(immutable_heads().., "
        let suffix = ") | trunk()"
        guard revset.hasPrefix(prefix), revset.hasSuffix(suffix) else { return nil }
        let start = revset.index(revset.startIndex, offsetBy: prefix.count)
        let end = revset.index(revset.endIndex, offsetBy: -suffix.count)
        return Int(revset[start ..< end])
    }

    static func canLoadMore(revset: String, loadedCount: Int) -> Bool {
        guard let depth = defaultRevsetDepth(for: revset) else { return false }
        return loadedCount >= depth
    }
}
