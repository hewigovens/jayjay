import Foundation
import JayJayCore

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
        lastInternalMutationAt = Date()
        runRepoTask(action) { viewModel, _ in
            viewModel.successActionSignal += 1
            beforeRefresh(viewModel)
            viewModel.refresh(selecting: rev)
        }
    }

    func performMessaging(
        selecting rev: String? = "@",
        _ action: @escaping RepoOperation<String>
    ) {
        lastInternalMutationAt = Date()
        runRepoTask(action) { viewModel, message in
            viewModel.successActionSignal += 1
            viewModel.info = message
            viewModel.refresh(selecting: rev)
        }
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

    private func runRepoTask<Result>(
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
