import JayJayCore

struct RepoRefreshContext {
    let bookmarks: [BookmarkInfo]
    let workspaces: [WorkspaceInfo]?
    let prHostName: String?
    let statusBar: StatusBarSnapshot

    init(repo: JayJayRepo) throws {
        bookmarks = try repo.listBookmarks()
        workspaces = try? repo.workspaceList()
        prHostName = repo.prHostName()
        statusBar = StatusBarSnapshot.load(from: repo)
    }
}
