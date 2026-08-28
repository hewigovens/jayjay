import AppKit
import Foundation
import JayJayCore

extension RepoViewModel {
    func gitFetch() {
        performPull { repo, sync in
            try repo.gitFetch(remote: "origin", sync: sync)
        }
    }

    func gitPullBookmark(name: String) {
        performPull { repo, sync in
            try repo.gitPullBookmark(bookmark: name, sync: sync)
        }
    }

    func gitPush(bookmark: String = "") {
        _ = gitPushIfIdle(bookmark: bookmark)
    }

    @discardableResult
    func gitPushIfIdle(bookmark: String) -> Bool {
        let sync = repo.syncToken()
        let started = performResult(
            gatedBy: RepoActionGate(
                state: \.isPushingInFlight,
                busyMessage: "Push already in progress"
            ),
            onSuccess: { viewModel, message in
                viewModel.pushSync = nil
                viewModel.info = message
            },
            onFailure: { viewModel, error in
                viewModel.pushSync = nil
                viewModel.presentSyncFailure(error, canceledMessage: "Push canceled")
            },
            { try $0.gitPush(bookmark: bookmark, sync: sync) }
        )
        if started {
            pushSync = sync
        }
        return started
    }

    func cancelPull() {
        pullSync?.cancel()
    }

    func cancelPush() {
        pushSync?.cancel()
    }

    func forgetStaleBookmarks() {
        performMessaging { repo in
            let count = try repo.forgetStaleBookmarks()
            return count > 0 ? "Forgot \(count) stale bookmark\(count == 1 ? "" : "s")" : "No stale bookmarks found"
        }
    }

    func openPR(bookmark: String) {
        guard !bookmark.isEmpty else { return }
        runRepoTask {
            try $0.pullRequestOpenUrl(bookmark: bookmark)
        } onSuccess: { viewModel, urlString in
            if let url = URL(string: urlString) {
                NSWorkspace.shared.open(url)
            } else {
                viewModel.info = urlString
            }
        }
    }

    private func handleFetchResult(_ result: FetchResult) {
        var msg = result.message
        if !result.abandonedBookmarks.isEmpty {
            let names = result.abandonedBookmarks.joined(separator: ", ")
            msg += "\nAbandoned merged: \(names)"
        }
        if !result.suggestAbandonBookmarks.isEmpty {
            let names = result.suggestAbandonBookmarks.joined(separator: ", ")
            msg += "\nConflicting (may be merged): \(names) — consider abandoning"
        }
        info = msg
    }

    private func performPull(_ operation: @escaping @Sendable (JayJayRepo, JayJaySyncToken) throws -> FetchResult) {
        let sync = repo.syncToken()
        let started = performResult(
            gatedBy: RepoActionGate(
                state: \.isPullingInFlight,
                busyMessage: "Pull already in progress"
            ),
            onSuccess: { viewModel, result in
                viewModel.pullSync = nil
                viewModel.handleFetchResult(result)
            },
            onFailure: { viewModel, error in
                viewModel.pullSync = nil
                viewModel.presentSyncFailure(error, canceledMessage: "Pull canceled")
            },
            { repo in try operation(repo, sync) }
        )
        if started {
            pullSync = sync
        }
    }

    @MainActor
    private func presentSyncFailure(_ error: any Error, canceledMessage: String) {
        if let jjError = error as? JayJayError, case .Canceled = jjError {
            info = canceledMessage
            // The remote phase may have landed before the cancel took effect.
            refresh()
        } else {
            present(error: error)
        }
    }
}
