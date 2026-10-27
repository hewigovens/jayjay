import JayJayCore

struct RepoRefreshContext {
    let bookmarks: [BookmarkInfo]
    let tags: [TagInfo]
    let revsetVocabulary: RevsetVocabulary
    let workspaces: [WorkspaceInfo]?
    let trunkName: String
    let prHostName: String?
    let fixUnavailableReason: String?
    let bisectState: BisectState?
    let statusBar: StatusBarSnapshot

    init(repo: JayJayRepo) throws {
        let context = try repo.repositoryContext()
        bisectState = context.bisectState
        bookmarks = context.bookmarks
        tags = context.tags
        revsetVocabulary = context.vocabulary
        workspaces = context.workspaces
        trunkName = context.trunkName
        prHostName = context.prHostName
        fixUnavailableReason = context.fixUnavailableReason
        statusBar = StatusBarSnapshot(workingCopyStats: context.workingCopyStats, currentOperationDescription: context.operation)
    }
}

@MainActor
extension RepoViewModel {
    func apply(_ context: RepoRefreshContext) {
        bisect.state = context.bisectState
        bookmarks = context.bookmarks
        tags = context.tags
        revsetVocabulary = context.revsetVocabulary
        if let workspaces = context.workspaces {
            self.workspaces = workspaces
        }
        trunkName = context.trunkName
        prHostName = context.prHostName
        fixUnavailableReason = context.fixUnavailableReason
        apply(context.statusBar)
    }
}
