import JayJayCore

extension RepoViewModel {
    func workspaceList() -> [WorkspaceInfo] {
        (try? repo.workspaceList()) ?? []
    }

    func workspaceAdd(dest: String, name: String, rev: String = "") {
        performMessaging { try $0.workspaceAdd(dest: dest, name: name, rev: rev) }
    }

    func workspaceForget(name: String) {
        perform { try $0.workspaceForget(name: name) }
    }
}
