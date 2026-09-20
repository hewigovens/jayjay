import Foundation
import JayJayCore

struct PullRequestImportRequest {
    let url: String
    let previewedHeadCommitId: String
    let workspaceName: String
    let workspaceDest: String
}

extension RepoViewModel {
    func resolvePullRequestImport(
        url: String,
        onSuccess: @escaping @MainActor (PullRequestImportPreview) -> Void,
        onFailure: @escaping @MainActor (any Error) -> Void
    ) {
        guard !isResolvingPullRequest else { return }
        isResolvingPullRequest = true
        let sync = repo.syncToken()
        pullRequestImportSync = sync
        runRepoTask { try $0.pullRequestImportPreview(url: url, sync: sync) } onSuccess: { viewModel, preview in
            viewModel.isResolvingPullRequest = false
            viewModel.pullRequestImportSync = nil
            onSuccess(preview)
        } onFailure: { viewModel, error in
            viewModel.isResolvingPullRequest = false
            viewModel.pullRequestImportSync = nil
            onFailure(error)
        }
    }

    func importPullRequest(
        _ request: PullRequestImportRequest,
        onSuccess: @escaping @MainActor (String) -> Void,
        onFailure: @escaping @MainActor (any Error) -> Void
    ) {
        let sync = repo.syncToken()
        let started = performResult(
            selecting: nil,
            gatedBy: RepoActionGate(
                state: \.isImportingPullRequest,
                busyMessage: "A pull request import is already in progress"
            ),
            onSuccess: { viewModel, createdPath in
                viewModel.pullRequestImportSync = nil
                onSuccess(createdPath)
            },
            onFailure: { viewModel, error in
                viewModel.pullRequestImportSync = nil
                if let jjError = error as? JayJayError, case .Canceled = jjError {
                    // The remote add or the fetch may have landed before the cancel took effect.
                    viewModel.refresh()
                }
                onFailure(error)
            },
            {
                try $0.pullRequestImport(
                    url: request.url, previewedHeadCommitId: request.previewedHeadCommitId,
                    workspaceName: request.workspaceName, workspaceDest: request.workspaceDest, sync: sync
                )
            }
        )
        if started {
            pullRequestImportSync = sync
        }
    }

    func cancelPullRequestImport() {
        pullRequestImportSync?.cancel()
    }
}
