import AppKit
import Foundation
import JayJayCore

extension RepoViewModel {
    func gitFetch() {
        lastInternalMutationAt = Date()
        runRepoTask { repo in
            try repo.gitFetch(remote: "origin")
        } onSuccess: { viewModel, result in
            viewModel.successActionSignal += 1
            viewModel.handleFetchResult(result)
            viewModel.refresh(selecting: "@")
        }
    }

    func gitPullBookmark(name: String) {
        lastInternalMutationAt = Date()
        runRepoTask { repo in
            try repo.gitPullBookmark(bookmark: name)
        } onSuccess: { viewModel, result in
            viewModel.successActionSignal += 1
            viewModel.handleFetchResult(result)
            viewModel.refresh(selecting: "@")
        }
    }

    func gitPush(bookmark: String = "") {
        performMessaging { try $0.gitPush(bookmark: bookmark) }
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
                    info = "Couldn't determine a pull request URL — push the bookmark to a GitHub, GitLab, or Codeberg remote first."
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
}
