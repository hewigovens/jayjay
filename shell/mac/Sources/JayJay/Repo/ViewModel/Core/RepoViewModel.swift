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

    var selectedChange: ChangeDetail?
    var selectedChangeId: String?
    /// When set, the detail panel shows an interdiff (from → to).
    var compareFromId: String?
    var bookmarks: [BookmarkInfo] = []
    var workingCopyDescription: String = ""
    var commitDraft: String = ""
    var opLogEntries: [OpLogEntry] = []
    var submoduleAttentionItems: [GitSubmoduleStatus] = []
    var pendingCommitMessage: String?
    var error: String?
    var info: String?
    var workspaces: [WorkspaceInfo] = []
    var isLoading = false
    var canLoadMore = true
    let reviewStore = ReviewStore()
    let diffStore = DiffStore()

    var revset: String = defaultRevset()

    let repo: JayJayRepo

    var aiProvider: String = ""
    var hasWorkingCopyChanges = false
    var successActionSignal = 0
    var configWarning: String?
    private var fsWatcher: RepoFSWatcher?
    var refreshTask: Task<Void, Never>?
    /// Stamp set by `perform()` so handleWorkingCopyChange can suppress its own FS echo.
    var lastInternalMutationAt: Date?
    /// True while a refresh task is running — gates FS-triggered re-entry.
    var isRefreshingInFlight: Bool = false
    var prInfo: PrInfo?
    var prFetchTask: Task<Void, Never>?
    var evologEntries: [EvologEntry]?
    var evologRev: String?

    init(path: String) throws {
        repoPath = path
        repo = try JayJayRepo.open(path: path)
        reviewStore.setRepoPath(path)
        aiProvider = Self.detectAIProvider()
        configWarning = repo.checkUserConfig()
        fsWatcher = RepoFSWatcher(
            repoPath: path,
            onChange: { [weak self] in self?.handleWorkingCopyChange() },
            onWorkingCopyChange: { [weak self] in self?.handleWorkingCopyChange() },
            isRelevantWorkingCopyChange: { [repo] paths in
                (try? repo.hasUnignoredWorkingCopyPaths(paths: paths)) ?? true
            }
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
