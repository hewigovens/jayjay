import Foundation
import JayJayCore

extension RepoViewModel {
    func workspaceList() -> [WorkspaceInfo] {
        workspaces
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

    @MainActor
    func workspaceRemovalGuard(
        name: String,
        expectedRoot: String,
        expectedOperation: String
    ) async throws -> String {
        try await awaitRepoTask {
            try $0.workspaceRemovalGuard(
                name: name,
                expectedRoot: expectedRoot,
                expectedOperation: expectedOperation
            )
        }
    }

    @MainActor
    func workspaceForget(
        name: String,
        expectedRoot: String,
        expectedOperation: String
    ) async throws -> String? {
        lastInternalMutationAt = Date()
        let warning = try await awaitRepoTask {
            try $0.workspaceForget(
                name: name,
                expectedRoot: expectedRoot,
                expectedOperation: expectedOperation
            )
        }
        successActionSignal += 1
        refresh()
        return warning
    }

    @MainActor
    func workspaceForgetUnresolved(
        name: String,
        expectedOperation: String
    ) async throws -> String? {
        lastInternalMutationAt = Date()
        let warning = try await awaitRepoTask {
            try $0.workspaceForgetUnresolved(
                name: name,
                expectedOperation: expectedOperation
            )
        }
        successActionSignal += 1
        refresh()
        return warning
    }
}
