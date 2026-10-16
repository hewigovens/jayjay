extension RepoWindowManager {
    func openFileHistory(_ request: FileHistoryRequest) {
        openWindowAction?(AppWindows.fileHistory, request.windowValue)
    }

    func revealInGraph(repoPath: String, revision: String) {
        guard let viewModel = registeredViewModel(at: repoPath), activateRepo(repoPath) else { return }
        viewModel.revealInGraph(revision)
    }
}
