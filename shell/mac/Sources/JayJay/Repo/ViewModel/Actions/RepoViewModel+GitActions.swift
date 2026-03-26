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
}
