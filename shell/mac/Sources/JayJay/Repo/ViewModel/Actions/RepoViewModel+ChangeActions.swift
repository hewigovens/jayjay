import Foundation
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

        // Capture before the commit rewrites the graph; the committed change keeps this id and only its marks should clear.
        let committedChangeId = changes.first(where: \.isWorkingCopy)?.changeId.id
        perform(selecting: "@", beforeRefresh: { viewModel in
            if let committedChangeId {
                viewModel.reviewStore.clearChange(changeId: committedChangeId)
            }
            viewModel.submoduleAttentionItems = []
            viewModel.pendingCommitMessage = nil
            viewModel.commitSummaryDraft = ""
            viewModel.commitDescriptionDraft = ""
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
        let committedChangeId = changes.first(where: \.isWorkingCopy)?.changeId.id

        isLoading = true
        do {
            let infoMessage = try await Task.detached { [repo] in
                try repo.commitSafeSubmoduleUpdates(
                    message: "\(message) (submodule)",
                    paths: safePaths
                )
            }.value

            if let committedChangeId {
                reviewStore.clearChange(changeId: committedChangeId)
            }
            submoduleAttentionItems = []
            pendingCommitMessage = nil
            commitSummaryDraft = ""
            commitDescriptionDraft = ""
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
        // A new change starts empty, so clear the commit-box draft instead of
        // carrying the previous change's message into it.
        perform(beforeRefresh: { viewModel in
            viewModel.commitSummaryDraft = ""
            viewModel.commitDescriptionDraft = ""
        }, {
            try $0.newChange(parent: parent, message: message)
        })
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

    func revertChange(rev: String) {
        perform { try $0.revertChange(rev: rev) }
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

    /// Move a bookmark to an arbitrary revision (drag-to-move). Backward moves are
    /// allowed silently, matching jj's default. Undoable via the op log. If the
    /// bookmark tracks a remote, offer a one-click push follow-up — but only once
    /// the move has actually succeeded (so the banner can't push the old target).
    func moveBookmark(name: String, toRev: String) {
        let wasTracking = bookmarks.first(where: { $0.name == name })?.isTrackingRemote == true
        perform(
            selecting: nil,
            beforeRefresh: { viewModel in
                if wasTracking { viewModel.pendingPushBookmark = name }
            },
            { try $0.moveBookmark(name: name, toRev: toRev) }
        )
    }

    func confirmPendingPush() {
        guard let name = pendingPushBookmark else { return }
        pendingPushBookmark = nil
        gitPush(bookmark: name)
    }

    func dismissPendingPush() {
        pendingPushBookmark = nil
    }

    func deleteBookmark(name: String) {
        perform(selecting: nil) { try $0.deleteBookmark(name: name) }
    }

    func forgetBookmark(name: String) {
        perform(selecting: nil) { try $0.forgetBookmark(name: name) }
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
        // The remainder keeps @'s content but gets a fresh change id; the typed draft still describes it.
        perform(beforeRefresh: { viewModel in
            viewModel.keepCommitDraftOnNextWorkingCopyChange = true
        }, {
            try $0.split(rev: rev, paths: paths, message: message, parallel: parallel)
        })
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
        // Abandoning lines from a leaf @ rewrites only that commit, so the cheap in-place row patch is safe; any other rev (or @ mid-stack) rebases descendants and needs the full refresh.
        if destination == .removeFromSource, Self.canPatchWorkingCopyRowInPlace(rev: rev, changes: changes) {
            abandonWorkingCopySelection(rev: rev, selections: selections, ignoreWhitespace: ignoreWhitespace)
            return
        }
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

    /// True when rev is the working copy and @ has no children in the loaded graph, so a removeFromSource rewrite cannot move any other row.
    static func canPatchWorkingCopyRowInPlace(rev: String, changes: [ChangeInfo]) -> Bool {
        guard let workingCopy = changes.first(where: \.isWorkingCopy) else { return false }
        let revIsWorkingCopy = rev == "@" || rev == workingCopy.changeId.id || rev == workingCopy.commitId.id
        let isLeaf = !changes.contains { $0.parents.contains(workingCopy.commitId.id) }
        return revIsWorkingCopy && isLeaf
    }

    private func abandonWorkingCopySelection(
        rev: String,
        selections: [DiffEditFileSelection],
        ignoreWhitespace: Bool
    ) {
        lastInternalMutationAt = Date()
        let includeSubmoduleStatuses = includeSubmoduleStatuses
        load {
            try $0.applyDiffSelection(
                rev: rev,
                destination: .removeFromSource,
                selections: selections,
                message: "",
                ignoreWhitespace: ignoreWhitespace
            )
            // The mutation already updated @; reload only this change's detail.
            let detail = try Self.loadSummaryWithConflicts(
                repo: $0,
                rev: rev,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            )
            return (detail, StatusBarSnapshot.load(from: $0))
        } onSuccess: { viewModel, result in
            let (detail, statusBar) = result
            viewModel.successActionSignal += 1
            viewModel.selectedChange = detail
            viewModel.selectedChangeId = detail.info.selectionRevision
            viewModel.apply(statusBar)
            // Patch the @ row in place (no descendants → edges unchanged) instead of a full log rebuild.
            if let index = viewModel.graphEntries.firstIndex(where: { $0.change.isWorkingCopy }) {
                viewModel.graphEntries[index] = GraphEntry(
                    change: detail.info,
                    edges: viewModel.graphEntries[index].edges
                )
            }
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
