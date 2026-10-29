import JayJayCore

struct RepoRefreshContext {
    let bookmarks: [BookmarkInfo]
    let tags: [TagInfo]
    let revsetVocabulary: RevsetVocabulary
    let workspaces: [WorkspaceInfo]?
    let trunkName: String
    let prHostName: String?
    let fixUnavailableReason: String?
    let statusBar: StatusBarSnapshot

    init(repo: JayJayRepo) throws {
        bookmarks = try repo.listBookmarks()
        tags = try repo.listTags()
        revsetVocabulary = repo.revsetVocabulary(bookmarks: bookmarks)
        workspaces = try? repo.workspaceList()
        trunkName = repo.trunkName()
        prHostName = repo.prHostName()
        fixUnavailableReason = repo.fixUnavailableReason()
        statusBar = StatusBarSnapshot.load(from: repo)
    }
}

extension RepoViewModel {
    @MainActor
    func apply(_ context: RepoRefreshContext) {
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
