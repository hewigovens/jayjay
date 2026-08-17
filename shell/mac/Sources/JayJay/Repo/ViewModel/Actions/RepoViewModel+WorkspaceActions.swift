import JayJayCore

extension RepoViewModel {
    func workspaceList() -> [WorkspaceInfo] {
        (try? repo.workspaceList()) ?? []
    }

    func workspaceAdd(
        dest: String,
        name: String,
        rev: String = "",
        onSuccess: @escaping @MainActor () -> Void = {},
        onFailure: @escaping @MainActor () -> Void = {}
    ) {
        performResult(
            gatedBy: RepoActionGate(
                state: \.isAddingWorkspace,
                busyMessage: "A workspace is already being created"
            ),
            beforeRefresh: { _ in onSuccess() },
            onSuccess: { viewModel, message in viewModel.info = message },
            onFailure: { viewModel, error in
                viewModel.present(error: error)
                onFailure()
            },
            { try $0.workspaceAdd(dest: dest, name: name, rev: rev) }
        )
    }

    func workspaceForget(name: String) {
        perform { try $0.workspaceForget(name: name) }
    }
}
