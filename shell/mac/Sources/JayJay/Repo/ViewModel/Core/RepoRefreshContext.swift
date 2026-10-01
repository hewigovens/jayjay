import JayJayCore

struct RepoRefreshContext {
    let bookmarks: [BookmarkInfo]
    let revsetVocabulary: RevsetVocabulary
    let workspaces: [WorkspaceInfo]?
    let prHostName: String?
    let fixUnavailableReason: String?
    let statusBar: StatusBarSnapshot

    init(repo: JayJayRepo) throws {
        bookmarks = try repo.listBookmarks()
        revsetVocabulary = repo.revsetVocabulary(bookmarks: bookmarks)
        workspaces = try? repo.workspaceList()
        prHostName = repo.prHostName()
        fixUnavailableReason = repo.fixUnavailableReason()
        statusBar = StatusBarSnapshot.load(from: repo)
    }
}
