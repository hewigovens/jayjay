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

    func updateStaleWorkspace() {
        perform(selecting: nil) { try $0.updateStaleWorkspace() }
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

/// What the Refresh button and ⌘R do: a stale working copy cannot be snapshotted, so they update it instead.
enum RefreshMode {
    case refresh
    case updateWorkspace

    var help: String {
        switch self {
            case .refresh: "Refresh (\(AppShortcut.refresh.symbol))"
            case .updateWorkspace: "Update Workspace — the working copy is stale (\(AppShortcut.refresh.symbol))"
        }
    }

    var showsBadge: Bool {
        self == .updateWorkspace
    }
}

extension RepoViewModel {
    var refreshMode: RefreshMode {
        isWorkingCopyStale ? .updateWorkspace : .refresh
    }

    func runRefresh() {
        switch refreshMode {
            case .refresh: refresh()
            case .updateWorkspace: updateStaleWorkspace()
        }
    }
}
