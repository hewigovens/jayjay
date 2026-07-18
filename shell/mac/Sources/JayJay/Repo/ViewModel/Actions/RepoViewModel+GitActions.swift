import AppKit
import Foundation
import JayJayCore

extension RepoViewModel {
    func gitFetch() {
        performPull { repo in
            try repo.gitFetch(remote: "origin")
        }
    }

    func gitPullBookmark(name: String) {
        performPull { repo in
            try repo.gitPullBookmark(bookmark: name)
        }
    }

    func gitPush(bookmark: String = "") {
        _ = gitPushIfIdle(bookmark: bookmark)
    }

    @discardableResult
    func gitPushIfIdle(bookmark: String) -> Bool {
        performResult(
            gatedBy: RepoActionGate(
                state: \.isPushingInFlight,
                busyMessage: "Push already in progress"
            ),
            onSuccess: { viewModel, message in viewModel.info = message },
            { try $0.gitPush(bookmark: bookmark) }
        )
    }

    func forgetStaleBookmarks() {
        performMessaging { repo in
            let count = try repo.forgetStaleBookmarks()
            return count > 0 ? "Forgot \(count) stale bookmark\(count == 1 ? "" : "s")" : "No stale bookmarks found"
        }
    }

    func openPR(bookmark: String) {
        guard !bookmark.isEmpty else { return }
        Task.detached { [repo] in
            let url = repo.pullRequestOpenUrl(bookmark: bookmark).flatMap(URL.init(string:))
            await MainActor.run { [weak self] in
                guard let self else { return }
                if let url {
                    NSWorkspace.shared.open(url)
                } else {
                    info = "Couldn't determine a pull request URL — no GitHub, GitLab, or Codeberg \"origin\" remote found."
                }
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

    private func performPull(_ operation: @escaping RepoOperation<FetchResult>) {
        performResult(
            gatedBy: RepoActionGate(
                state: \.isPullingInFlight,
                busyMessage: "Pull already in progress"
            ),
            onSuccess: { viewModel, result in viewModel.handleFetchResult(result) },
            operation
        )
    }
}
