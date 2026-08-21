import Foundation
import JayJayCore

extension RepoViewModel {
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

    func refreshWorkspaces() {
        runRepoTask { try $0.workspaceList() } onSuccess: { viewModel, workspaces in
            viewModel.workspaces = workspaces
        }
    }

    @MainActor
    func forgetWorkspace(_ workspace: WorkspaceInfo, deleteFromDisk: Bool) async -> Bool {
        lastInternalMutationAt = Date()
        do {
            let warning = try await awaitRepoTask {
                if deleteFromDisk {
                    return try $0.workspaceForgetAndDelete(
                        name: workspace.name,
                        expectedRoot: workspace.path
                    )
                }
                try $0.workspaceForget(
                    name: workspace.name,
                    expectedRoot: workspace.isPathResolved ? workspace.path : nil
                )
                return nil
            }
            if let warning {
                error = warning
            }
        } catch {
            present(error: error)
            return false
        }
        successActionSignal += 1
        refresh()
        return true
    }
}
