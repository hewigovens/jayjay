import Foundation
import JayJayCore

struct RepoActionGate {
    let state: ReferenceWritableKeyPath<RepoViewModel, Bool>
    let busyMessage: String
}

extension RepoViewModel {
    typealias RepoOperation<Result> = @Sendable (JayJayRepo) throws -> Result

    /// Retains lifecycle work independently of the latest-task handles, so superseded synchronous FFI remains visible to the workspace-removal barrier.
    @discardableResult
    func startLifecycleRepoTask(
        _ operation: @escaping @Sendable () async -> Void
    ) -> Task<Void, Never> {
        guard !isShuttingDown else {
            return Task {}
        }
        let taskId = UUID()
        let task = Task.detached { [self] in
            await operation()
            await MainActor.run {
                lifecycleRepoTasks[taskId] = nil
            }
        }
        lifecycleRepoTasks[taskId] = task
        return task
    }

    @MainActor
    func present(error: any Error) {
        self.error = error.friendlyDescription
    }

    /// Refresh failures from a workspace that vanished underneath the window (forgotten or deleted elsewhere) close the window instead of alerting; an undecided presence means the repo was merely unreadable, so the real error still belongs on screen. The presence probe is passed in because it repeats the filesystem access that just failed and must not run on the main actor.
    @MainActor
    func presentRefreshFailure(_ error: any Error, presence: WorkspacePresence) {
        guard presence == .gone else {
            present(error: error)
            return
        }
        workspaceVanished = true
    }

    /// The presence probe repeats the filesystem access that just failed and can block for a volume timeout, so cancellation must be checked on both sides of it and after the main-actor hop.
    func handleRefreshFailure(
        _ error: any Error,
        workspacePresence: @Sendable () -> WorkspacePresence
    ) async {
        guard !Task.isCancelled else { return }
        let presence = workspacePresence()
        guard !Task.isCancelled else { return }
        await MainActor.run {
            guard !Task.isCancelled, !isShuttingDown else { return }
            isLoading = false
            isRefreshingInFlight = false
            presentRefreshFailure(error, presence: presence)
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
        guard !isShuttingDown else { return }
        let taskId = UUID()
        inFlightRepoTasks[taskId] = Task.detached { [self, repo] in
            do {
                let result = try operation(repo)
                await MainActor.run {
                    onSuccess(self, result)
                    inFlightRepoTasks[taskId] = nil
                }
            } catch {
                await MainActor.run {
                    onFailure(self, error)
                    inFlightRepoTasks[taskId] = nil
                }
            }
        }
    }

    /// Runs an awaited repo operation through the same registry as callback-based actions, so workspace removal cannot outrun direct async reads or mutations.
    @MainActor
    func awaitRepoTask<Result>(_ operation: @escaping RepoOperation<Result>) async throws -> Result {
        guard !isShuttingDown else { throw CancellationError() }
        let taskId = UUID()
        return try await withCheckedThrowingContinuation { continuation in
            inFlightRepoTasks[taskId] = Task.detached { [self, repo] in
                let outcome = Swift.Result { try operation(repo) }
                await MainActor.run {
                    inFlightRepoTasks[taskId] = nil
                    continuation.resume(with: outcome)
                }
            }
        }
    }

    @MainActor
    func runJjCommand(_ command: String) async throws -> JjCommandResult {
        let path = repoPath
        return try await awaitRepoTask { _ in
            try runJjCommandInRepoPath(repoPath: path, command: command)
        }
    }
}
