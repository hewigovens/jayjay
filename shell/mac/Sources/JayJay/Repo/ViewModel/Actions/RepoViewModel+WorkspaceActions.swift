import JayJayCore

extension RepoViewModel {
    func workspaceList() -> [WorkspaceInfo] {
        (try? repo.workspaceList()) ?? []
    }

    func workspaceAdd(dest: String, name: String, rev: String = "", onSuccess: @escaping () -> Void = {}) {
        performResult(onSuccess: { _, _ in onSuccess() }) {
            try $0.workspaceAdd(dest: dest, name: name, rev: rev)
        }
    }

    func workspaceForget(name: String) {
        perform { try $0.workspaceForget(name: name) }
    }

    /// Instant list highlight. Does not move DAG selection, open the other path, or snapshot.
    func adoptWorkspaceAppearance(_ workspace: WorkspaceInfo) {
        workspaceSwitchGeneration += 1
        refreshTask?.cancel()
        isRefreshingInFlight = false
        workspaces = WorkspaceSidebarPolicy.markingCurrent(workspaces, name: workspace.name)
    }

    /// Bind mutations and the watcher to another workspace path. Caller must not snapshot.
    @MainActor
    func attachWorkspace(
        path: String,
        repo: JayJayRepo,
        workingCopyIsLarge: Bool,
        configWarning: String?,
        selecting revision: String
    ) {
        refreshTask?.cancel()
        isRefreshingInFlight = false
        fsWatcher = nil
        repoPath = path
        self.repo = repo
        self.workingCopyIsLarge = workingCopyIsLarge
        self.configWarning = configWarning
        fsWatcher = RepoFSWatcher(
            repoPath: path,
            onChange: { [weak self] in self?.handleWorkingCopyChange() },
            onWorkingCopyChange: { [weak self] in self?.handleWorkingCopyChange() },
            isRelevantWorkingCopyChange: { [repo] paths in
                (try? repo.hasUnignoredWorkingCopyPaths(paths: paths)) ?? true
            }
        )
        refresh(
            selecting: revision,
            snapshotWorkingCopy: false,
            switchGeneration: workspaceSwitchGeneration
        )
    }

    /// Inspect another workspace's committed `@` versus its parent without `jj edit`.
    func showWorkspaceChanges(_ workspace: WorkspaceInfo) {
        let to = workspace.wcCommitId
        guard !to.isEmpty else { return }
        let from = workspace.parentCommitId
        guard !from.isEmpty else {
            select(changeId: to)
            return
        }
        compareWith(
            from: from,
            to: to,
            display: CompareDisplay(title: "Workspace \(workspace.name)", from: "parent", to: workspace.name)
        )
    }

    /// Interdiff of two workspaces' working-copy commits. Does not `jj edit`.
    func compareWorkspace(_ workspace: WorkspaceInfo, against baseline: WorkspaceInfo) {
        let to = workspace.wcCommitId
        let from = baseline.wcCommitId
        guard !to.isEmpty, !from.isEmpty else { return }
        compareWith(
            from: from,
            to: to,
            display: CompareDisplay(
                title: "\(workspace.name) vs \(baseline.name)",
                from: baseline.name,
                to: workspace.name
            )
        )
    }
}
