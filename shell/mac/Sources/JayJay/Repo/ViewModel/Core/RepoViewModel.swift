import Foundation
import JayJayCore

@Observable
final class RepoViewModel: ChangeActions, DAGActions, BookmarkActions {
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
    var opLogEntries: [OpLogEntry] = []
    var error: String?
    var info: String?
    var workspaces: [WorkspaceInfo] = []
    var isLoading = false
    let reviewStore = ReviewStore()

    var revset: String = defaultRevset()
    /// Number of ancestors to show in default revset. Increases on "Load More".
    var ancestorLimit: Int = 20
    /// Whether a custom (non-default) revset is active.
    var isCustomRevset: Bool {
        revset != Self.buildDefaultRevset(limit: ancestorLimit)
    }

    /// False when load-more returned no new entries (reached oldest commit).
    var hasMoreToLoad = true

    let repo: JayJayRepo

    var aiProvider: String = ""
    var hasWorkingCopyChanges = false
    private var fsWatcher: RepoFSWatcher?
    var refreshTask: Task<Void, Never>?

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
}
