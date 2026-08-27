import Foundation
import JayJayCore

struct RepoActionGate {
    let state: ReferenceWritableKeyPath<RepoViewModel, Bool>
    let busyMessage: String
}

extension RepoViewModel {
    typealias RepoOperation<Result> = @Sendable (JayJayRepo) throws -> Result

    @discardableResult
    func startRepoTask(_ operation: @escaping @Sendable () async -> Void) -> Task<Void, Never> {
        guard !isShuttingDown else {
            return Task {}
        }
        let taskId = UUID()
        let task = Task.detached { [weak self] in
            await operation()
            await self?.finishRepoTask(taskId)
        }
        repoTasks[taskId] = task
        return task
    }

    @MainActor
    func present(error: any Error) {
        self.error = error.friendlyDescription
    }

    /// A vanished workspace closes the window; an undecided presence keeps the real error.
    func handleRefreshFailure(
        _ error: any Error,
        workspacePresence: @Sendable () -> WorkspacePresence
    ) async {
        guard !Task.isCancelled else { return }
        let presence = workspacePresence()
        guard !Task.isCancelled else { return }
        await MainActor.run {
            applyRefreshFailure(error, presence: presence)
        }
    }

    @MainActor
    func applyRefreshFailure(_ error: any Error, presence: WorkspacePresence) {
        guard !Task.isCancelled, !isShuttingDown else { return }
        isLoading = false
        isRefreshingInFlight = false
        if presence == .gone {
            workspaceVanished = true
        } else {
            present(error: error)
        }
    }

    func perform(
        selecting rev: String? = "@",
        beforeRefresh: @escaping @MainActor (RepoViewModel) -> Void = { _ in },
        _ action: @escaping RepoOperation<Void>
    ) {
        performResult(
            selecting: rev,
            beforeRefresh: beforeRefresh,
            onSuccess: { _, _ in },
            action
        )
    }

    func performMessaging(
        selecting rev: String? = "@",
        _ action: @escaping RepoOperation<String>
    ) {
        performResult(
            selecting: rev,
            onSuccess: { viewModel, message in viewModel.info = message },
            action
        )
    }

    @discardableResult
    func performResult<Result>(
        selecting rev: String? = "@",
        gatedBy gate: RepoActionGate? = nil,
        beforeRefresh: @escaping @MainActor (RepoViewModel) -> Void = { _ in },
        onSuccess: @escaping @MainActor (RepoViewModel, Result) -> Void,
        onFailure: @escaping @MainActor (RepoViewModel, any Error) -> Void = { viewModel, error in
            viewModel.present(error: error)
        },
        _ action: @escaping RepoOperation<Result>
    ) -> Bool {
        if let gate {
            guard !self[keyPath: gate.state] else {
                info = gate.busyMessage
                return false
            }
            self[keyPath: gate.state] = true
        }
        lastInternalMutationAt = Date()
        runRepoTask(action) { viewModel, result in
            if let gate {
                viewModel[keyPath: gate.state] = false
            }
            viewModel.successActionSignal += 1
            beforeRefresh(viewModel)
            onSuccess(viewModel, result)
            viewModel.refresh(selecting: rev)
        } onFailure: { viewModel, error in
            if let gate {
                viewModel[keyPath: gate.state] = false
            }
            onFailure(viewModel, error)
        }
        return true
    }

    func runRepoTask<Result>(
        _ operation: @escaping RepoOperation<Result>,
        onSuccess: @escaping @MainActor (RepoViewModel, Result) -> Void,
        onFailure: @escaping @MainActor (RepoViewModel, any Error) -> Void = { viewModel, error in
            viewModel.present(error: error)
        }
    ) {
        startRepoTask { [weak self, repo] in
            let outcome = Swift.Result { try operation(repo) }
            await self?.applyRepoTaskOutcome(
                outcome,
                onSuccess: onSuccess,
                onFailure: onFailure
            )
        }
    }

    @MainActor
    private func finishRepoTask(_ taskId: UUID) {
        repoTasks[taskId] = nil
    }

    @MainActor
    private func applyRepoTaskOutcome<Result>(
        _ outcome: Swift.Result<Result, any Error>,
        onSuccess: @escaping @MainActor (RepoViewModel, Result) -> Void,
        onFailure: @escaping @MainActor (RepoViewModel, any Error) -> Void
    ) {
        guard !isShuttingDown, !Task.isCancelled else { return }
        switch outcome {
            case let .success(result): onSuccess(self, result)
            case let .failure(error): onFailure(self, error)
        }
    }

    @MainActor
    func awaitRepoTask<Result>(_ operation: @escaping RepoOperation<Result>) async throws -> Result {
        guard !isShuttingDown else { throw CancellationError() }
        return try await withCheckedThrowingContinuation { continuation in
            startRepoTask { [repo] in
                continuation.resume(with: Swift.Result { try operation(repo) })
            }
        }
    }

    @MainActor
    func runJjCommand(_ command: String) async throws -> JjCommandResult {
        try await awaitRepoTask { repo in
            try repo.runJjCommand(command: command)
        }
    }
}
