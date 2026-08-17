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
        let expectedRoot = deleteFromDisk ? workspace.path : nil
        do {
            try await awaitRepoTask {
                try $0.workspaceForget(name: workspace.name, expectedRoot: expectedRoot)
            }
        } catch {
            present(error: error)
            return false
        }
        successActionSignal += 1
        refresh()
        guard deleteFromDisk else { return true }
        do {
            try await Task.detached {
                try FileManager.default.removeItem(atPath: workspace.path)
            }.value
        } catch {
            self.error = "Workspace \(workspace.name) was forgotten, but its directory could not be deleted:\n\(workspace.path)\n\(error.localizedDescription)"
        }
        return true
    }
}
