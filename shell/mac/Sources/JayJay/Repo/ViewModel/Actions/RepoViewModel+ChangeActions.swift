import JayJayCore

extension RepoViewModel {
    func describeChange(rev: String, message: String) {
        describe(rev: rev, message: message)
    }

    func describe(rev: String, message: String) {
        perform(selecting: rev) { try $0.describe(rev: rev, message: message) }
    }

    func describeWorkingCopy(message: String) {
        perform { try $0.describe(rev: "@", message: message) }
    }

    @MainActor
    @discardableResult
    func commit(message: String, manageSubmodules: Bool) async -> Bool {
        if manageSubmodules {
            do {
                let blockedSubmodules = try await Task.detached { [repo] in
                    try repo.submoduleStatuses()
                }.value
                if !blockedSubmodules.isEmpty {
                    pendingCommitMessage = message
                    submoduleAttentionItems = blockedSubmodules
                    return false
                }
            } catch {
                self.error = error.friendlyDescription
                return false
            }
        }

        perform(selecting: "@", beforeRefresh: { viewModel in
            viewModel.reviewStore.clearAll()
            viewModel.submoduleAttentionItems = []
            viewModel.pendingCommitMessage = nil
        }, {
            try $0.jjCommit(message: message)
        })
        return true
    }

    @MainActor
    @discardableResult
    func commitWithSafeSubmoduleUpdates() async -> Bool {
        guard let message = pendingCommitMessage else { return false }

        let safePaths = submoduleAttentionItems
            .filter { $0.hasNewCommits && !$0.hasModifiedContent && !$0.hasUntrackedContent }
            .map(\.path)
        guard !safePaths.isEmpty else { return false }

        isLoading = true
        do {
            let infoMessage = try await Task.detached { [repo] in
                try repo.commitSafeSubmoduleUpdates(
                    message: "\(message) (submodule)",
                    paths: safePaths
                )
            }.value

            reviewStore.clearAll()
            submoduleAttentionItems = []
            pendingCommitMessage = nil
            info = infoMessage
            refresh(selecting: "@")
            isLoading = false
            return true
        } catch {
            self.error = error.friendlyDescription
            isLoading = false
            return false
        }
    }

    func newChange(parent: String, message: String = "") {
        perform { try $0.newChange(parent: parent, message: message) }
    }

    func abandon(rev: String) {
        perform(selecting: "@", beforeRefresh: { viewModel in
            viewModel.selectedChangeId = nil
            viewModel.selectedChange = nil
        }, {
            try $0.abandon(rev: rev)
        })
    }

    func squash(rev: String) {
        perform { try $0.squash(rev: rev, intoRev: nil) }
    }

    func squash(rev: String, into destination: String) {
        perform(selecting: destination) { try $0.squash(rev: rev, intoRev: destination) }
    }

    func edit(rev: String) {
        perform(selecting: rev) { try $0.edit(rev: rev) }
    }

    func graft(rev: String) {
        perform { try $0.graft(rev: rev) }
    }

    func absorb(rev: String) {
        perform { try $0.absorb(rev: rev) }
    }

    func backout(rev: String) {
        perform { try $0.backout(rev: rev) }
    }

    func rebase(rev: String, dest: String) {
        perform { try $0.rebase(rev: rev, dest: dest) }
    }

    func merge(parents: [String]) {
        perform { try $0.merge(parentRevs: parents) }
    }

    func duplicate(rev: String) {
        perform { try $0.duplicate(rev: rev) }
    }

    func createBookmark(name: String, rev: String = "@") {
        perform(selecting: nil) { try $0.createBookmark(name: name, rev: rev) }
    }

    func moveBookmarkForward(name: String) {
        perform(selecting: nil) { try $0.moveBookmark(name: name, toRev: "@-") }
    }

    func deleteBookmark(name: String) {
        perform(selecting: nil) { try $0.deleteBookmark(name: name) }
    }

    func renameBookmark(oldName: String, newName: String) {
        perform(selecting: nil) { try $0.renameBookmark(oldName: oldName, newName: newName) }
    }

    func trackBookmark(name: String, remote: String) {
        perform(selecting: nil) { try $0.trackBookmark(name: name, remote: remote) }
    }

    func restoreFiles(rev: String, paths: [String]) {
        perform(selecting: rev) { try $0.restoreFiles(rev: rev, paths: paths) }
    }

    func deleteFiles(paths: [String]) {
        perform { try $0.deleteFiles(paths: paths) }
    }

    func ignoreAndUntrack(paths: [String]) {
        perform(selecting: nil) { try $0.ignoreAndUntrack(paths: paths) }
    }

    func split(rev: String, paths: [String], message: String = "", parallel: Bool = false) {
        perform { try $0.split(rev: rev, paths: paths, message: message, parallel: parallel) }
    }

    func moveToWorkingCopy(rev: String, paths: [String]) {
        perform { try $0.moveToWorkingCopy(rev: rev, paths: paths) }
    }

    func applyDiffSelection(
        rev: String,
        destination: DiffEditDestination,
        selections: [DiffEditFileSelection],
        message: String,
        ignoreWhitespace: Bool
    ) {
        perform(selecting: rev) {
            try $0.applyDiffSelection(
                rev: rev,
                destination: destination,
                selections: selections,
                message: message,
                ignoreWhitespace: ignoreWhitespace
            )
        }
    }

    func opLog() {
        load {
            try $0.opLog()
        } onSuccess: { viewModel, entries in
            viewModel.opLogEntries = entries
        }
    }

    func opRestore(opId: String) {
        perform { try $0.opRestore(opId: opId) }
    }

    func resolveUseOurs(rev: String, path: String) {
        perform(selecting: rev) { try $0.resolveUseOurs(rev: rev, path: path) }
    }

    func resolveInEditor(rev: String, path: String, tool: String) {
        perform(selecting: rev) { try $0.resolveWithTool(rev: rev, path: path, tool: tool) }
    }

    func resolveUseTheirs(rev: String, path: String) {
        perform(selecting: rev) { try $0.resolveUseTheirs(rev: rev, path: path) }
    }
}
