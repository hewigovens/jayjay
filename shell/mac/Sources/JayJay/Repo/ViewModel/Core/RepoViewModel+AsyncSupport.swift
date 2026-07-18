import Foundation
import JayJayCore

struct RepoActionGate {
    let state: ReferenceWritableKeyPath<RepoViewModel, Bool>
    let busyMessage: String
}

extension RepoViewModel {
    typealias RepoOperation<Result> = @Sendable (JayJayRepo) throws -> Result

    @MainActor
    func present(error: any Error) {
        self.error = error.friendlyDescription
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
            if let gate { viewModel[keyPath: gate.state] = false }
            viewModel.successActionSignal += 1
            beforeRefresh(viewModel)
            onSuccess(viewModel, result)
            viewModel.refresh(selecting: rev)
        } onFailure: { viewModel, error in
            if let gate { viewModel[keyPath: gate.state] = false }
            viewModel.present(error: error)
        }
        return true
    }

    func load<Result>(
        _ operation: @escaping RepoOperation<Result>,
        onSuccess: @escaping @MainActor (RepoViewModel, Result) -> Void,
        onFailure: @escaping @MainActor (RepoViewModel, any Error) -> Void = { viewModel, error in
            viewModel.present(error: error)
        }
    ) {
        runRepoTask(operation, onSuccess: onSuccess, onFailure: onFailure)
    }

    func runRepoTask<Result>(
        _ operation: @escaping RepoOperation<Result>,
        onSuccess: @escaping @MainActor (RepoViewModel, Result) -> Void,
        onFailure: @escaping @MainActor (RepoViewModel, any Error) -> Void = { viewModel, error in
            viewModel.present(error: error)
        }
    ) {
        Task.detached { [repo] in
            do {
                let result = try operation(repo)
                await MainActor.run { [weak self] in
                    guard let self else { return }
                    onSuccess(self, result)
                }
            } catch {
                await MainActor.run { [weak self] in
                    guard let self else { return }
                    onFailure(self, error)
                }
            }
        }
    }
}
