extension RepoViewModel {
    func gitFetch() {
        performMessaging { try $0.gitFetch(remote: "origin") }
    }

    func gitPullBookmark(name: String) {
        performMessaging { try $0.gitPullBookmark(bookmark: name) }
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
}
