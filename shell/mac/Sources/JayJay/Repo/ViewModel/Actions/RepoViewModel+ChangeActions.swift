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
                let blockedSubmodules = try await awaitRepoTask { try $0.submoduleStatuses() }
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
            let infoMessage = try await awaitRepoTask {
                try $0.commitSafeSubmoduleUpdates(
                    message: "\(message) (submodule)",
                    paths: safePaths
                )
            }

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
        performClearingDrafts { try $0.newChange(parent: parent, message: message) }
    }

    func insertChange(rev: String, position: InsertPosition) {
        performClearingDrafts { try $0.newChangeInserted(rev: rev, position: position, message: "") }
    }

    private func performClearingDrafts(_ action: @escaping RepoOperation<Void>) {
        perform(beforeRefresh: { viewModel in
            viewModel.commitSummaryDraft = ""
            viewModel.commitDescriptionDraft = ""
        }, action)
    }

    func abandon(rev: String) {
        perform(selecting: "@", beforeRefresh: { viewModel in
            viewModel.applySingleSelectedChange(nil)
        }, {
            try $0.abandon(rev: rev)
        })
    }

    func abandon(revs: [String]) {
        perform(selecting: "@", beforeRefresh: { viewModel in
            viewModel.applySingleSelectedChange(nil)
        }, {
            try $0.abandonMany(revs: revs)
        })
    }

    func squash(rev: String) {
        perform { try $0.squash(rev: rev, intoRev: nil) }
    }

    func squash(rev: String, into destination: String) {
        perform(selecting: destination) { try $0.squash(rev: rev, intoRev: destination) }
    }

    func squash(revs: [String]) {
        performResult(
            selectingResult: { $0 },
            onSuccess: { _, _ in },
            { try $0.squashMany(revs: revs) }
        )
    }

    func edit(rev: String) {
        perform(selecting: rev, beforeRefresh: { viewModel in
            guard viewModel.change(for: rev)?.isWorkingCopy != true else { return }
            // Explicit Edit changes which revision @ points to, so discard any draft for the old working copy before the new description is loaded.
            viewModel.commitSummaryDraft = commitSummary(message: viewModel.workingCopyDescription)
            viewModel.commitDescriptionDraft = commitBody(message: viewModel.workingCopyDescription)
        }, {
            try $0.edit(rev: rev)
        })
    }

    func absorb(rev: String) {
        performResult(
            onSuccess: { viewModel, outcome in
                switch outcome {
                    case .changed:
                        break
                    case .unchanged:
                        viewModel.info = "Nothing to absorb. Use Squash into parent instead."
                }
            },
            { try $0.absorb(rev: rev) }
        )
    }

    func revertChange(rev: String) {
        perform { try $0.revertChange(rev: rev) }
    }

    func rebase(rev: String, dest: String) {
        perform { try $0.rebase(rev: rev, dest: dest) }
    }

    func rebase(revs: [String], dest: String) {
        perform { try $0.rebaseMany(revs: revs, dest: dest) }
    }

    func merge(parents: [String]) {
        perform { try $0.merge(parentRevs: parents) }
    }

    func duplicate(rev: String) {
        perform { try $0.duplicate(rev: rev) }
    }

    func opLog() {
        runRepoTask {
            try $0.opLog()
        } onSuccess: { viewModel, entries in
            viewModel.opLogEntries = entries
        }
    }

    func opRestore(opId: String) {
        perform { try $0.opRestore(opId: opId) }
    }
}
