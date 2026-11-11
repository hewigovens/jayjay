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

    @MainActor
    func applyRefreshFailure(_ error: any Error, presence: WorkspacePresence, context: RepoRefreshContext? = nil) {
        guard !Task.isCancelled, !isShuttingDown else { return }
        if let context {
            apply(context)
        }
        isLoading = false
        isRefreshingInFlight = false
        hasFinishedFirstLoad = true
        if presence == .gone {
            workspaceVanished = true
        } else {
            present(error: error)
        }
        // A queued background refresh must not be stranded by a failed one.
        resumePendingBackgroundRefresh(afterFailure: true)
    }

    func perform(
        selecting rev: String? = "@",
        undoable message: String? = nil,
        beforeRefresh: @escaping @MainActor (RepoViewModel) -> Void = { _ in },
        _ action: @escaping RepoOperation<Void>
    ) {
        performResult(selecting: rev, undoable: message.map { message in { _ in message } }, beforeRefresh: beforeRefresh, action)
    }

    @discardableResult
    func performResult<Result>(
        selecting rev: String? = "@",
        selectingResult: ((Result) -> String)? = nil,
        gatedBy gate: RepoActionGate? = nil,
        undoable: ((Result) -> String?)? = nil,
        // Undo and redo step through history rather than act, so they never count toward the rating prompt.
        signalsSuccess: Bool = true,
        beforeRefresh: @escaping @MainActor (RepoViewModel) -> Void = { _ in },
        onSuccess: @escaping @MainActor (RepoViewModel, Result) -> Void = { _, _ in },
        // A manual refresh clears `error`, so anything that sets it belongs here rather than in `onSuccess`.
        afterRefresh: @escaping @MainActor (RepoViewModel, Result) -> Void = { _, _ in },
        onFailure: @escaping @MainActor (RepoViewModel, any Error) -> Void = { viewModel, error in
            viewModel.present(error: error)
        },
        _ action: @escaping RepoOperation<Result>
    ) -> Bool {
        guard repo.sshState() == .connected else { return false }
        if let gate {
            guard !self[keyPath: gate.state] else {
                info = gate.busyMessage
                return false
            }
            self[keyPath: gate.state] = true
        }
        lastInternalMutationAt = Date()
        pendingMutations += 1
        let tracksOperation = undoable != nil
        runRepoTask { repo -> (result: Result, operationId: String?) in
            guard tracksOperation else { return try (action(repo), nil) }
            return try Self.trackingOperation(repo: repo, action)
        } onSuccess: { viewModel, tracked in
            viewModel.pendingMutations -= 1
            if let gate {
                viewModel[keyPath: gate.state] = false
            }
            if signalsSuccess {
                viewModel.successActionSignal += 1
            }
            beforeRefresh(viewModel)
            onSuccess(viewModel, tracked.result)
            if let message = undoable?(tracked.result) {
                viewModel.actionNotice = RepoActionNotice(message: message, undoOperationId: tracked.operationId)
            }
            viewModel.refresh(selecting: selectingResult.map { $0(tracked.result) } ?? rev)
            afterRefresh(viewModel, tracked.result)
        } onFailure: { viewModel, error in
            viewModel.pendingMutations -= 1
            if let gate {
                viewModel[keyPath: gate.state] = false
            }
            onFailure(viewModel, error)
        }
        return true
    }

    static func trackingOperation<Result>(
        repo: JayJayRepo,
        _ action: RepoOperation<Result>
    ) throws -> (result: Result, operationId: String?) {
        _ = takeMutationOperationId()
        let result = try action(repo)
        return (result, takeMutationOperationId())
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
